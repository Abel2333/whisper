use std::sync::Arc;

use anyhow::{Result, anyhow, bail, ensure};
use rig::{
    agent::{Agent, AgentBuilder},
    client::{CompletionClient, EmbeddingsClient, completion::CompletionModelHandle},
    completion::CompletionModelDyn,
    embeddings::{Embed, EmbeddingsBuilder, embedding::EmbeddingModelDyn},
    providers::{deepseek, ollama, openai},
    tool::ToolSet,
    vector_store::in_memory_store::InMemoryVectorStore,
};
use serde::Serialize;

use crate::{
    agent::dyn_embedding_wrapper::DynEmbeddingModelWrapper,
    config::read_config::{ModelConfig, ModelType},
};

pub struct AgentSettings {
    model_configs: Vec<ModelConfig>,
    temperature: f32,
    tool_set: Option<ToolSet>,
    preamble: String,
}

type DynCompletionModel = Arc<dyn CompletionModelDyn>;
type DynEmbeddingModel = Arc<dyn EmbeddingModelDyn>;

type CompletionHandle = CompletionModelHandle<'static>;
type GeneralEmb = DynEmbeddingModelWrapper<2048>;

impl AgentSettings {
    pub fn try_new(
        model_configs: Vec<ModelConfig>,
        tool_set: Option<ToolSet>,
        preamble: String,
        temperature: f32,
    ) -> anyhow::Result<Self> {
        let settings = Self {
            model_configs,
            tool_set,
            temperature,
            preamble,
        };
        settings.validate()?;

        Ok(settings)
    }

    fn validate(&self) -> anyhow::Result<()> {
        let len = self.model_configs.len();
        ensure!(len > 0, "AgentSettings requires at least one model config");
        ensure!(
            len <= 2,
            "AgentSettings supports at most two model configs (completion + optional embedding)",
        );

        match len {
            1 => {
                ensure!(
                    self.model_configs[0].model_type == ModelType::Completion,
                    "Single-model setup must be a completion model",
                );
            }
            2 => {
                let embedding_count = self
                    .model_configs
                    .iter()
                    .filter(|m| m.model_type == ModelType::Embedding)
                    .count();
                let completion_count = self
                    .model_configs
                    .iter()
                    .filter(|m| m.model_type == ModelType::Completion)
                    .count();

                ensure!(
                    embedding_count == 1 && completion_count == 1,
                    "Two-model setup must contain exactly one embedding and one completion model",
                );
            }
            _ => bail!("unreachable"),
        }

        Ok(())
    }
}

pub async fn create_agent<T, D>(
    agent_settings: AgentSettings,
    docs: Option<D>,
) -> Result<Agent<CompletionModelHandle<'static>>>
where
    T: Embed + Serialize + Eq + Send + Sync + 'static,
    D: IntoIterator<Item = T>,
{
    agent_settings.validate()?;

    let AgentSettings {
        model_configs,
        tool_set,
        preamble,
        temperature,
    } = agent_settings;

    let mut tool_set = tool_set;

    let mut docs = docs;

    let mut completion_model: Option<DynCompletionModel> = None;
    let mut embedding_model: Option<GeneralEmb> = None;

    for config in &model_configs {
        match config.model_type {
            ModelType::Completion => completion_model = Some(create_completion_model(config)?),
            ModelType::Embedding => embedding_model = Some(create_embed_model(config)?),
            ModelType::Chat => return Err(anyhow!("Chat model type not supported yet")),
        }
    }

    let completion =
        completion_model.ok_or_else(|| anyhow!("AgentSettings must include a completion model"))?;

    let mut builder = AgentBuilder::new(CompletionHandle {
        inner: completion.clone(),
    })
    .preamble(&preamble)
    .temperature(temperature as f64);

    if let Some(model) = embedding_model.as_ref() {
        match docs.take() {
            Some(doc_iter) => {
                let embeddings = EmbeddingsBuilder::new(model.clone())
                    .documents(doc_iter)?
                    .build()
                    .await?;

                let vector_store = InMemoryVectorStore::from_documents(embeddings);
                let index = vector_store.index(model.clone());

                match tool_set.take() {
                    Some(tooling) => {
                        builder = builder.dynamic_tools(2, index, tooling);
                    }
                    None => {
                        log::warn!(
                            "Embedding model configured but no ToolSet supplied; skipping dynamic tools"
                        );
                    }
                }
            }
            None => {
                log::warn!(
                    "Embedding model configured but no documents supplied; skipping dynamic tools"
                );
            }
        }
    } else if docs.is_some() {
        log::warn!("Documents provided without an embedding model; ignoring document collection");
    }

    log::info!(
        "Building agent with {} completion model(s) and embedding: {}",
        1,
        embedding_model.is_some()
    );
    let agent = builder.build();

    Ok(agent)
}

fn create_completion_model(model_config: &ModelConfig) -> Result<DynCompletionModel> {
    if model_config.model_type == ModelType::Embedding {
        return Err(anyhow!("Invalid model type"));
    }

    log::debug!(
        "Initializing completion model '{}' via provider '{}'",
        model_config.model_name,
        model_config.provider
    );

    let model = match model_config.provider.as_str() {
        "openai" => {
            let client = openai::Client::builder(&model_config.api_key)
                .base_url(&model_config.base_url)
                .build()?;

            let chat_model = client
                .completion_model(&model_config.model_name)
                .completions_api();

            let model: DynCompletionModel = Arc::new(chat_model);
            model
        }
        "ollama" => {
            let client = ollama::Client::builder()
                .base_url(&model_config.base_url)
                .build()?;

            let chat_model = client.completion_model(&model_config.model_name);
            let model: DynCompletionModel = Arc::new(chat_model);
            model
        }
        "deepseek" => {
            let client = deepseek::Client::builder(&model_config.api_key)
                .base_url(&model_config.base_url)
                .build()?;

            let chat_model = client.completion_model(&model_config.model_name);
            let model: DynCompletionModel = Arc::new(chat_model);
            model
        }
        other => return Err(anyhow!("Unsupported provider: {}", other)),
    };

    Ok(model)
}

fn create_embed_model(model_config: &ModelConfig) -> Result<GeneralEmb> {
    if model_config.model_type == ModelType::Completion {
        return Err(anyhow!("Invalid model type"));
    }

    let max_docs = usize::try_from(model_config.context_size)
        .map_err(|_| anyhow!("context_size exceeds usize range"))?;
    ensure!(
        max_docs > 0,
        "context_size must be positive for embedding models"
    );

    log::debug!(
        "Initializing embedding model '{}' via provider '{}' (max batch {})",
        model_config.model_name,
        model_config.provider,
        max_docs
    );

    let base_model: DynEmbeddingModel = match model_config.provider.as_str() {
        "openai" => {
            let client = openai::Client::builder(&model_config.api_key)
                .base_url(&model_config.base_url)
                .build()?;

            let embed_model = client.embedding_model(&model_config.model_name);
            Arc::new(embed_model)
        }
        "ollama" => {
            let client = ollama::Client::builder()
                .base_url(&model_config.base_url)
                .build()?;

            let embed_model = client.embedding_model(&model_config.model_name);
            Arc::new(embed_model)
        }
        other => return Err(anyhow!("Unsupported provider: {}", other)),
    };

    let wrapped = GeneralEmb::new(base_model);

    Ok(wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_config(model_type: ModelType) -> ModelConfig {
        ModelConfig {
            base_url: "http://localhost".into(),
            api_key: "secret".into(),
            provider: "openai".into(),
            model_name: match model_type {
                ModelType::Completion => "gpt",
                ModelType::Embedding => "embed",
                ModelType::Chat => "chat",
            }
            .into(),
            model_type,
            context_size: 256,
        }
    }

    #[test]
    fn validate_accepts_single_completion() {
        let settings = AgentSettings {
            model_configs: vec![mk_config(ModelType::Completion)],
            temperature: 0.5,
            tool_set: None,
            preamble: "system".into(),
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_rejects_single_embedding_only() {
        let settings = AgentSettings {
            model_configs: vec![mk_config(ModelType::Embedding)],
            temperature: 0.5,
            tool_set: None,
            preamble: "system".into(),
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_accepts_completion_plus_embedding() {
        let settings = AgentSettings {
            model_configs: vec![
                mk_config(ModelType::Completion),
                mk_config(ModelType::Embedding),
            ],
            temperature: 0.5,
            tool_set: None,
            preamble: "system".into(),
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_rejects_more_than_two_models() {
        let settings = AgentSettings {
            model_configs: vec![
                mk_config(ModelType::Completion),
                mk_config(ModelType::Embedding),
                mk_config(ModelType::Completion),
            ],
            temperature: 0.5,
            tool_set: None,
            preamble: "system".into(),
        };
        assert!(settings.validate().is_err());
    }
}
