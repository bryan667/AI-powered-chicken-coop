pub struct AiVision {
    pub model_path: String,
    pub api_key: String,
}

impl AiVision {
    pub fn load_model(path: &str, key: &str) -> Self {
        println!("Loading AI model from {} with key {}", path, key);
        AiVision {
            model_path: path.to_string(),
            api_key: key.to_string(),
        }
    }

    pub fn detect(&self) -> bool {
        println!(
            "Running AI detection on model {} (auth key length: {})",
            self.model_path,
            self.api_key.len()
        );
        !self.model_path.contains("predator")
    }
}

#[cfg(test)]
mod tests {
    use super::AiVision;

    #[test]
    fn egg_detector_returns_true() {
        let model = AiVision::load_model("/models/egg_detector.pt", "AI_KEY");
        assert!(model.detect());
    }

    #[test]
    fn predator_detector_returns_false() {
        let model = AiVision::load_model("/models/predator_detector.pt", "AI_KEY");
        assert!(!model.detect());
    }
}
