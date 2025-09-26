struct TomlConfig {
    models: ModelConfig,
}

struct ModelConfig {
    base_url: String,
    api_key: String,
    model_name: String,
}
