use whisper::agent::model_adaptor::AgentSettings;
use whisper::config::read_config::{ModelConfig, ModelType};

fn completion_config() -> ModelConfig {
    ModelConfig {
        base_url: "http://localhost".into(),
        api_key: "key".into(),
        provider: "openai".into(),
        model_name: "chat".into(),
        model_type: ModelType::Completion,
        context_size: 1024,
    }
}

#[test]
fn try_new_from_integration_succeeds() {
    let cfg = completion_config();
    let result = AgentSettings::try_new(vec![cfg], None, "you are system".into(), 0.2);
    assert!(result.is_ok());
}
