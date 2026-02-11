pub struct AiVision {
    pub model_path: String,
    pub api_key: String,
}

impl AiVision {
    pub fn load_model(path: &str, key: &str) -> Self {
        println!("Loading AI model from {} with key {}", path, key);
        AiVision { model_path: path.to_string(), api_key: key.to_string() }
    }

    pub fn detect(&self) -> bool {
        println!("Running AI detection on model {}", self.model_path);
        true
    }
}
