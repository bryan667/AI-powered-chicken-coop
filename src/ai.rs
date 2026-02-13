#[cfg(feature = "vision-local")]
use image::imageops::FilterType;
#[cfg(feature = "vision-local")]
use image::DynamicImage;
#[cfg(feature = "vision-local")]
use ndarray::Array4;
#[cfg(feature = "vision-local")]
use tract_onnx::prelude::*;

pub struct AiVision {
    pub model_path: String,
    pub api_key: String,
    #[cfg(feature = "vision-local")]
    model: RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    #[cfg(feature = "vision-local")]
    labels: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct VisionResult {
    pub label: String,
    pub confidence: f32,
    pub chicken_detected: bool,
    pub predator_detected: bool,
}

fn redact_key(key: &str) -> String {
    let shown: String = key.chars().take(4).collect();
    format!("{shown}***")
}

#[cfg(feature = "vision-local")]
fn is_chicken_label(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    l.contains("hen") || l.contains("cock") || l.contains("chicken") || l.contains("rooster")
}

#[cfg(feature = "vision-local")]
fn is_predator_label(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    l.contains("fox")
        || l.contains("wolf")
        || l.contains("coyote")
        || l.contains("dog")
        || l.contains("hawk")
        || l.contains("eagle")
        || l.contains("owl")
        || l.contains("snake")
}

#[cfg(feature = "vision-local")]
fn load_labels(path: Option<&str>) -> Vec<String> {
    if let Some(file_path) = path {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let labels: Vec<String> = content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
                .collect();
            if !labels.is_empty() {
                return labels;
            }
        }
    }
    Vec::new()
}

#[cfg(feature = "vision-local")]
fn preprocess_imagenet_224(img: DynamicImage) -> Result<Tensor, String> {
    let resized = img.resize_exact(224, 224, FilterType::Triangle).to_rgb8();
    let mut input = Array4::<f32>::zeros((1, 3, 224, 224));

    // ImageNet normalization
    let mean = [0.485_f32, 0.456_f32, 0.406_f32];
    let std = [0.229_f32, 0.224_f32, 0.225_f32];

    for (x, y, pixel) in resized.enumerate_pixels() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        let xi = x as usize;
        let yi = y as usize;
        input[[0, 0, yi, xi]] = (r - mean[0]) / std[0];
        input[[0, 1, yi, xi]] = (g - mean[1]) / std[1];
        input[[0, 2, yi, xi]] = (b - mean[2]) / std[2];
    }

    Ok(input.into_tensor())
}

#[cfg(feature = "vision-local")]
fn top_class(scores: &[f32]) -> Option<(usize, f32)> {
    let mut best_idx = None;
    let mut best_score = f32::MIN;
    for (idx, score) in scores.iter().enumerate() {
        if *score > best_score {
            best_idx = Some(idx);
            best_score = *score;
        }
    }
    best_idx.map(|idx| (idx, best_score))
}

impl AiVision {
    pub fn load_model(path: &str, key: &str) -> Self {
        println!(
            "Loading AI model from {} with key {}",
            path,
            redact_key(key)
        );

        #[cfg(feature = "vision-local")]
        let model = tract_onnx::onnx()
            .model_for_path(path)
            .unwrap_or_else(|e| panic!("failed to load model at {path}: {e}"))
            .into_optimized()
            .unwrap_or_else(|e| panic!("failed to optimize model: {e}"))
            .into_runnable()
            .unwrap_or_else(|e| panic!("failed to make model runnable: {e}"));

        #[cfg(feature = "vision-local")]
        let labels = load_labels(std::env::var("VISION_LABELS_PATH").ok().as_deref());

        AiVision {
            model_path: path.to_string(),
            api_key: key.to_string(),
            #[cfg(feature = "vision-local")]
            model,
            #[cfg(feature = "vision-local")]
            labels,
        }
    }

    #[cfg(feature = "vision-local")]
    pub fn classify_image(&self, image_path: &str) -> Result<VisionResult, String> {
        println!(
            "Running local image classification with model {} (auth key length: {})",
            self.model_path,
            self.api_key.len()
        );
        let image = image::open(image_path)
            .map_err(|e| format!("failed to open image `{image_path}`: {e}"))?;
        self.classify_dynamic_image(image)
    }

    #[cfg(feature = "vision-local")]
    pub fn classify_dynamic_image(&self, image: DynamicImage) -> Result<VisionResult, String> {
        let input = preprocess_imagenet_224(image)?;
        let outputs = self
            .model
            .run(tvec!(input.into()))
            .map_err(|e| format!("model inference failed: {e}"))?;
        let logits = outputs[0]
            .to_array_view::<f32>()
            .map_err(|e| format!("unexpected model output format: {e}"))?;
        let scores: Vec<f32> = logits.iter().copied().collect();
        let (idx, confidence) =
            top_class(&scores).ok_or_else(|| "model returned empty output".to_string())?;
        let label = self
            .labels
            .get(idx)
            .cloned()
            .unwrap_or_else(|| format!("class_{idx}"));

        Ok(VisionResult {
            chicken_detected: is_chicken_label(&label),
            predator_detected: is_predator_label(&label),
            label,
            confidence,
        })
    }

    #[cfg(not(feature = "vision-local"))]
    pub fn classify_image(&self, _image_path: &str) -> Result<VisionResult, String> {
        println!(
            "Vision disabled for model {} (auth key length: {})",
            self.model_path,
            self.api_key.len()
        );
        Err("local vision is disabled; run with cargo feature `vision-local`".to_string())
    }
}

#[cfg(all(test, feature = "vision-local"))]
mod tests {
    use super::{is_chicken_label, is_predator_label};

    #[test]
    fn chicken_labels_are_detected() {
        assert!(is_chicken_label("hen"));
        assert!(is_chicken_label("rooster"));
    }

    #[test]
    fn predator_labels_are_detected() {
        assert!(is_predator_label("red fox"));
        assert!(is_predator_label("hawk"));
    }
}
