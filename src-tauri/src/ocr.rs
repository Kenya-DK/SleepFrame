//! Screen-region OCR using a bundled, pure-Rust engine (`ocrs` + `rten`)..
//! (Pure Rust, cross-platform, models embedded — no external installs.)

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex, OnceLock};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{GrayImage, ImageFormat, RgbImage, RgbaImage};
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InvertMode {
    Auto,
    On,
    Off,
}

impl InvertMode {
    pub fn parse(value: &str) -> Self {
        match value {
            "on" => InvertMode::On,
            "off" => InvertMode::Off,
            _ => InvertMode::Auto,
        }
    }
}

/// Options controlling how a captured region is prepared and read.
#[derive(Clone)]
pub struct PreprocessOptions {
    pub invert: InvertMode,
    pub scale: u32,
    pub threshold: bool,
    /// When set, only pixels near this colour are treated as text.
    pub color: Option<[u8; 3]>,
    pub tolerance: u32,
    /// When set, the recogniser only outputs these characters.
    pub allowed_chars: Option<String>,
    /// Optional local detection model path (`.onnx`/`.rten`); blank = bundled.
    pub detection_model: Option<String>,
    /// Optional local recognition model path (`.onnx`/`.rten`); blank = bundled.
    pub recognition_model: Option<String>,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            invert: InvertMode::Auto,
            scale: 1,
            threshold: false,
            color: None,
            tolerance: 60,
            allowed_chars: None,
            detection_model: None,
            recognition_model: None,
        }
    }
}

/// Parses `#rrggbb` (or `rrggbb`); returns `None` for blank/invalid input.
pub fn parse_color(value: &str) -> Option<[u8; 3]> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

const DETECTION_MODEL: &[u8] = include_bytes!("../models/text-detection.onnx");
const RECOGNITION_MODEL: &[u8] = include_bytes!("../models/text-recognition.onnx");

/// Engines are cached per allowed-character set (building one is expensive).
static ENGINES: OnceLock<Mutex<HashMap<String, Result<Arc<OcrEngine>, String>>>> = OnceLock::new();

fn load_model(path: &Option<String>, fallback: &'static [u8]) -> Result<rten::Model, String> {
    match path {
        Some(path) if !path.trim().is_empty() => {
            rten::Model::load_file(path.trim()).map_err(|error| format!("{path}: {error}"))
        }
        _ => rten::Model::load_static_slice(fallback).map_err(|error| error.to_string()),
    }
}

fn build_engine(options: &PreprocessOptions) -> Result<Arc<OcrEngine>, String> {
    let detection = load_model(&options.detection_model, DETECTION_MODEL)?;
    let recognition = load_model(&options.recognition_model, RECOGNITION_MODEL)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection),
        recognition_model: Some(recognition),
        allowed_chars: options.allowed_chars.clone(),
        ..Default::default()
    })
    .map_err(|error| error.to_string())?;

    Ok(Arc::new(engine))
}

fn engine(options: &PreprocessOptions) -> Result<Arc<OcrEngine>, String> {
    let key = format!(
        "{}|{}|{}",
        options.allowed_chars.as_deref().unwrap_or(""),
        options.detection_model.as_deref().unwrap_or(""),
        options.recognition_model.as_deref().unwrap_or(""),
    );

    let cache = ENGINES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cached = cache.lock().map_err(|error| error.to_string())?;

    if let Some(existing) = cached.get(&key) {
        return existing.clone();
    }

    let built = build_engine(options);
    cached.insert(key, built.clone());
    built
}

/// Captures a rectangle of the primary monitor as an RGB image.
pub fn capture_region(x: i32, y: i32, width: i32, height: i32) -> Result<RgbImage, String> {
    if width <= 0 || height <= 0 {
        return Err("Region has no size".to_string());
    }

    let mut monitors = xcap::Monitor::all().map_err(|error| error.to_string())?;
    if monitors.is_empty() {
        return Err("No monitor found".to_string());
    }
    let index = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);
    let monitor = monitors.swap_remove(index);

    let rgba = monitor
        .capture_region(x.max(0) as u32, y.max(0) as u32, width as u32, height as u32)
        .map_err(|error| error.to_string())?;

    Ok(rgba_to_rgb(&rgba))
}

/// Flattens an RGBA image onto a white background.
fn rgba_to_rgb(rgba: &RgbaImage) -> RgbImage {
    let (width, height) = rgba.dimensions();
    let mut output = RgbImage::new(width, height);
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel.0[3] as u32;
        let blend = |channel: u8| -> u8 { ((channel as u32 * alpha + 255 * (255 - alpha)) / 255) as u8 };
        output.put_pixel(
            x,
            y,
            image::Rgb([blend(pixel.0[0]), blend(pixel.0[1]), blend(pixel.0[2])]),
        );
    }
    output
}

fn resize_rgb(image: RgbImage, scale: u32) -> RgbImage {
    if scale <= 1 {
        return image;
    }
    let (width, height) = image.dimensions();
    image::imageops::resize(
        &image,
        width * scale,
        height * scale,
        image::imageops::FilterType::Triangle,
    )
}

/// Keeps only pixels close to `target` (as black text), everything else white.
fn color_mask(image: &RgbImage, target: [u8; 3], tolerance: u32) -> RgbImage {
    let (width, height) = image.dimensions();
    let mut output = RgbImage::new(width, height);
    let tolerance = tolerance as i32;

    for (x, y, pixel) in image.enumerate_pixels() {
        let distance = (pixel.0[0] as i32 - target[0] as i32)
            .abs()
            .max((pixel.0[1] as i32 - target[1] as i32).abs())
            .max((pixel.0[2] as i32 - target[2] as i32).abs());

        let value = if distance <= tolerance { 0u8 } else { 255u8 };
        output.put_pixel(x, y, image::Rgb([value, value, value]));
    }
    output
}

/// Otsu's method: picks a threshold that best separates the histogram.
fn otsu(data: &[u8]) -> u8 {
    let mut histogram = [0u32; 256];
    for &value in data {
        histogram[value as usize] += 1;
    }

    let total = data.len() as f64;
    let sum_all: f64 = (0..256).map(|i| i as f64 * histogram[i] as f64).sum();

    let mut sum_background = 0.0;
    let mut weight_background = 0.0;
    let mut max_variance = -1.0;
    let mut threshold = 127u8;

    for i in 0..256 {
        weight_background += histogram[i] as f64;
        if weight_background == 0.0 {
            continue;
        }
        let weight_foreground = total - weight_background;
        if weight_foreground == 0.0 {
            break;
        }
        sum_background += i as f64 * histogram[i] as f64;
        let mean_background = sum_background / weight_background;
        let mean_foreground = (sum_all - sum_background) / weight_foreground;
        let between = weight_background
            * weight_foreground
            * (mean_background - mean_foreground)
            * (mean_background - mean_foreground);
        if between > max_variance {
            max_variance = between;
            threshold = i as u8;
        }
    }

    threshold
}

/// Prepares a captured region for OCR.
pub fn preprocess(image: RgbImage, options: &PreprocessOptions) -> RgbImage {
    // Colour-keyed masking: only the chosen colour becomes text.
    if let Some(target) = options.color {
        let masked = color_mask(&image, target, options.tolerance);
        return resize_rgb(masked, options.scale);
    }

    let gray = image::DynamicImage::ImageRgb8(image).into_luma8();
    let (width, height) = gray.dimensions();
    if width == 0 || height == 0 {
        return RgbImage::new(0, 0);
    }

    // Light denoise: smooths fine background patterns.
    let gray = image::imageops::blur(&gray, 1.0);

    let count = (width * height) as f32;
    let mean: f32 = gray.pixels().map(|pixel| pixel.0[0] as f32).sum::<f32>() / count;

    let invert_now = match options.invert {
        InvertMode::Auto => mean < 128.0,
        InvertMode::On => true,
        InvertMode::Off => false,
    };

    let mut values: Vec<u8> = Vec::with_capacity((width * height) as usize);
    let mut min = 255u8;
    let mut max = 0u8;
    for pixel in gray.pixels() {
        let mut value = pixel.0[0];
        if invert_now {
            value = 255 - value;
        }
        min = min.min(value);
        max = max.max(value);
        values.push(value);
    }

    let span = (max as i32 - min as i32).max(1) as f32;
    for value in values.iter_mut() {
        let normalized =
            ((*value as i32 - min as i32) as f32 / span * 255.0).round().clamp(0.0, 255.0);
        *value = normalized as u8;
    }

    if options.threshold {
        let cutoff = otsu(&values);
        for value in values.iter_mut() {
            *value = if *value > cutoff { 255 } else { 0 };
        }
    }

    let luma =
        GrayImage::from_raw(width, height, values).unwrap_or_else(|| GrayImage::new(width, height));
    let luma = if options.scale > 1 {
        image::imageops::resize(
            &luma,
            width * options.scale,
            height * options.scale,
            image::imageops::FilterType::Triangle,
        )
    } else {
        luma
    };

    let (out_width, out_height) = luma.dimensions();
    let mut output = RgbImage::new(out_width, out_height);
    for (x, y, pixel) in luma.enumerate_pixels() {
        let value = pixel.0[0];
        output.put_pixel(x, y, image::Rgb([value, value, value]));
    }
    output
}

/// Runs the OCR engine on a single (preprocessed) image.
fn ocr_single(image: &RgbImage, options: &PreprocessOptions) -> Result<String, String> {
    let source = ImageSource::from_bytes(image.as_raw(), image.dimensions())
        .map_err(|error| error.to_string())?;
    let engine = engine(options)?;
    let input = engine
        .prepare_input(source)
        .map_err(|error| error.to_string())?;
    engine.get_text(&input).map_err(|error| error.to_string())
}

/// Finds the row ranges of text lines using a horizontal ink projection.
fn text_line_rows(image: &RgbImage) -> Vec<(u32, u32)> {
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let total = (width * height) as f64;
    let sum: f64 = image.pixels().map(|pixel| pixel.0[0] as f64).sum();
    let text_is_dark = (sum / total) > 128.0;

    let mut ink = vec![0u32; height as usize];
    for (y, row) in ink.iter_mut().enumerate() {
        let mut count = 0u32;
        for x in 0..width {
            let value = image.get_pixel(x, y as u32).0[0];
            let is_text = if text_is_dark { value < 128 } else { value >= 128 };
            if is_text {
                count += 1;
            }
        }
        *row = count;
    }

    let min_ink = ((width as f32) * 0.02).max(1.0) as u32;

    let mut bands: Vec<(u32, u32)> = Vec::new();
    let mut start: Option<u32> = None;
    for y in 0..height {
        let has_ink = ink[y as usize] >= min_ink;
        match start {
            None if has_ink => start = Some(y),
            Some(s) if !has_ink => {
                bands.push((s, y));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        bands.push((s, height));
    }

    let mut merged: Vec<(u32, u32)> = Vec::new();
    for band in bands {
        if let Some(last) = merged.last_mut() {
            if band.0.saturating_sub(last.1) <= 4 {
                last.1 = band.1;
                continue;
            }
        }
        merged.push(band);
    }

    merged.retain(|band| band.1 - band.0 >= 6);
    merged
}

/// Drops lines that are not plausible text (fewer than two alphanumerics).
fn clean_text(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| line.chars().filter(|c| c.is_alphanumeric()).count() >= 2)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Runs OCR on an RGB image after preprocessing. Multi-line regions are split
/// into lines first, because the engine's line detection struggles with tightly
/// packed UI text.
pub fn read_rgb(image: RgbImage, options: &PreprocessOptions) -> Result<String, String> {
    let processed = preprocess(image, options);
    if processed.width() == 0 || processed.height() == 0 {
        return Err("Region has no size".to_string());
    }

    let lines = text_line_rows(&processed);
    if lines.len() <= 1 {
        return ocr_single(&processed, options).map(|text| clean_text(&text));
    }

    let mut texts: Vec<String> = Vec::new();
    for (top, bottom) in lines {
        let padding = 3u32;
        let top = top.saturating_sub(padding);
        let bottom = (bottom + padding).min(processed.height());
        let crop =
            image::imageops::crop_imm(&processed, 0, top, processed.width(), bottom - top)
                .to_image();

        if let Ok(text) = ocr_single(&crop, options) {
            texts.push(text);
        }
    }

    Ok(clean_text(&texts.join("\n")))
}

/// Runs OCR on an RGBA image (compositing transparency over white first).
pub fn read_rgba(image: RgbaImage, options: &PreprocessOptions) -> Result<String, String> {
    read_rgb(rgba_to_rgb(&image), options)
}

/// Preprocesses an RGBA image (for previews / debugging).
pub fn preprocess_rgba(image: RgbaImage, options: &PreprocessOptions) -> RgbImage {
    preprocess(rgba_to_rgb(&image), options)
}

/// Captures the region, preprocesses it and returns the recognised text.
pub fn read_text(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    options: &PreprocessOptions,
) -> Result<String, String> {
    let image = capture_region(x, y, width, height)?;
    read_rgb(image, options)
}

/// Captures the region, preprocesses it and returns a base64 PNG for preview.
pub fn preview_png(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    options: &PreprocessOptions,
) -> Result<String, String> {
    let image = capture_region(x, y, width, height)?;
    let processed = preprocess(image, options);
    let mut bytes: Vec<u8> = Vec::new();
    image::DynamicImage::ImageRgb8(processed)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_builds() {
        if let Err(error) = engine(&PreprocessOptions::default()) {
            panic!("failed to build OCR engine: {error}");
        }
        let restricted = PreprocessOptions {
            allowed_chars: Some("0123456789".into()),
            ..Default::default()
        };
        if let Err(error) = engine(&restricted) {
            panic!("failed to build restricted OCR engine: {error}");
        }
    }
}
