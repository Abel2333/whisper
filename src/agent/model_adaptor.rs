use std::sync::Arc;

use rig::{
    client::{CompletionClient, EmbeddingsClient},
    completion::CompletionModelDyn,
    embeddings::embedding::EmbeddingModelDyn,
    providers::{deepseek, ollama, openai},
};

use crate::config::read_config::{AppConfig, ModelConfig, ModelType};

trait ModelFactory {
    fn build_model(
        model_config: &ModelConfig,
        embed_models: &mut Vec<Arc<dyn EmbeddingModelDyn>>,
        completion_models: &mut Vec<Arc<dyn CompletionModelDyn>>,
    ) -> anyhow::Result<()>;
}

impl ModelFactory for openai::Client {
    fn build_model(
        model_config: &ModelConfig,
        embed_models: &mut Vec<Arc<dyn EmbeddingModelDyn>>,
        completion_models: &mut Vec<Arc<dyn CompletionModelDyn>>,
    ) -> anyhow::Result<()> {
        let client = openai::Client::builder(&model_config.api_key)
            .base_url(&model_config.base_url)
            .build()?;

        match model_config.model_type {
            ModelType::Embedding => {
                let embed = client.embedding_model(&model_config.model_name);
                embed_models.push(Arc::new(embed));
            }
            ModelType::Completion => {
                let completion = client
                    .completion_model(&model_config.model_name)
                    .completions_api();
                completion_models.push(Arc::new(completion));
            }
            ModelType::Chat => {}
        }

        Ok(())
    }
}

impl ModelFactory for deepseek::Client {
    fn build_model(
        model_config: &ModelConfig,
        _embed_models: &mut Vec<Arc<dyn EmbeddingModelDyn>>,
        completion_models: &mut Vec<Arc<dyn CompletionModelDyn>>,
    ) -> anyhow::Result<()> {
        let client = deepseek::Client::builder(&model_config.api_key)
            .base_url(&model_config.base_url)
            .build()?;

        match model_config.model_type {
            ModelType::Embedding => {}
            ModelType::Completion => {
                let completion = client.completion_model(&model_config.model_name);
                completion_models.push(Arc::new(completion));
            }
            ModelType::Chat => {}
        }

        Ok(())
    }
}

impl ModelFactory for ollama::Client {
    fn build_model(
        model_config: &ModelConfig,
        embed_models: &mut Vec<Arc<dyn EmbeddingModelDyn>>,
        completion_models: &mut Vec<Arc<dyn CompletionModelDyn>>,
    ) -> anyhow::Result<()> {
        let client = ollama::Client::builder()
            .base_url(&model_config.base_url)
            .build()?;

        match model_config.model_type {
            ModelType::Embedding => {
                let embed = client.embedding_model(&model_config.model_name);
                embed_models.push(Arc::new(embed));
            }
            ModelType::Completion => {
                let completion = client.completion_model(&model_config.model_name);
                completion_models.push(Arc::new(completion));
            }
            ModelType::Chat => {}
        }

        Ok(())
    }
}

pub type CompletionModelVec = Vec<Arc<dyn CompletionModelDyn>>;
pub type EmbedModelVec = Vec<Arc<dyn EmbeddingModelDyn>>;

fn load_models_for<T: ModelFactory>(
    model_config: &ModelConfig,
    embed_models: &mut Vec<Arc<dyn EmbeddingModelDyn>>,
    completion_models: &mut Vec<Arc<dyn CompletionModelDyn>>,
) -> anyhow::Result<()> {
    T::build_model(model_config, embed_models, completion_models)
}

pub fn load_models(app_config: &AppConfig) -> anyhow::Result<(CompletionModelVec, EmbedModelVec)> {
    let mut completion_models: CompletionModelVec = Vec::new();
    let mut embed_models: EmbedModelVec = Vec::new();

    for model_config in &app_config.models {
        match model_config.provider.as_str() {
            "openai" => {
                load_models_for::<openai::Client>(
                    model_config,
                    &mut embed_models,
                    &mut completion_models,
                )?;
            }
            "deepseek" => {
                load_models_for::<deepseek::Client>(
                    model_config,
                    &mut embed_models,
                    &mut completion_models,
                )?;
            }
            "ollama" => {
                load_models_for::<ollama::Client>(
                    model_config,
                    &mut embed_models,
                    &mut completion_models,
                )?;
            }
            _ => {}
        }
    }

    Ok((completion_models, embed_models))
}
