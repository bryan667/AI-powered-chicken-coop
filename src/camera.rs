#[cfg(feature = "camera")]
use image::DynamicImage;
#[cfg(feature = "camera")]
use image::ImageFormat;
#[cfg(feature = "camera")]
use nokhwa::pixel_format::RgbFormat;
#[cfg(feature = "camera")]
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
#[cfg(feature = "camera")]
use nokhwa::Camera;
#[cfg(feature = "camera")]
use std::fs;
#[cfg(feature = "camera")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "camera")]
pub fn capture_frame(camera_index: u32) -> Result<DynamicImage, String> {
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(CameraIndex::Index(camera_index), requested)
        .map_err(|e| format!("camera init failed: {e}"))?;
    camera
        .open_stream()
        .map_err(|e| format!("camera stream open failed: {e}"))?;

    let frame = camera
        .frame()
        .map_err(|e| format!("camera frame capture failed: {e}"))?;
    let decoded = frame
        .decode_image::<RgbFormat>()
        .map_err(|e| format!("camera frame decode failed: {e}"))?;
    Ok(DynamicImage::ImageRgb8(decoded))
}

#[cfg(feature = "camera")]
pub fn save_detection_frame(
    frame: &DynamicImage,
    label: &str,
    confidence: f32,
) -> Result<String, String> {
    fs::create_dir_all("captures").map_err(|e| format!("failed to create captures dir: {e}"))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock error: {e}"))?
        .as_millis();
    let safe_label: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let file_name = format!("captures/{ts}_{safe_label}_{:.3}.jpg", confidence);
    frame
        .save_with_format(&file_name, ImageFormat::Jpeg)
        .map_err(|e| format!("failed to save detection frame: {e}"))?;
    Ok(file_name)
}
