//! Debug helper: run the OCR pipeline on an image file.
//!
//! Usage:
//!   cargo run --example ocr_debug -- <image> [invert] [scale] [threshold] [color] [tolerance] [out.png]
//!
//! e.g. cargo run --example ocr_debug -- download.png auto 1 false "#bd2a2a" 60

use std::env;

use sleepframe_lib::ocr::{self, InvertMode, PreprocessOptions};

fn main() {
    let args: Vec<String> = env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!(
            "usage: ocr_debug <image> [invert=auto] [scale=1] [threshold=false] [color] [tolerance=60] [out.png]"
        );
        std::process::exit(1);
    };

    let options = PreprocessOptions {
        invert: InvertMode::parse(args.get(2).map(String::as_str).unwrap_or("auto")),
        scale: args
            .get(3)
            .and_then(|value| value.parse().ok())
            .unwrap_or(1),
        threshold: args.get(4).map(|value| value == "true").unwrap_or(false),
        color: args.get(5).and_then(|value| ocr::parse_color(value)),
        tolerance: args
            .get(6)
            .and_then(|value| value.parse().ok())
            .unwrap_or(60),
        allowed_chars: args
            .get(7)
            .filter(|value| !value.is_empty())
            .cloned(),
        detection_model: args
            .get(8)
            .filter(|value| !value.is_empty())
            .cloned(),
        recognition_model: args
            .get(9)
            .filter(|value| !value.is_empty())
            .cloned(),
    };

    let image = image::open(path)
        .expect("failed to open image")
        .into_rgba8();
    println!("image: {}x{}", image.width(), image.height());

    match ocr::read_rgba(image.clone(), &options) {
        Ok(text) => println!("--- OCR ---\n{text}"),
        Err(error) => println!("OCR error: {error}"),
    }

    if let Some(out) = args.get(10) {
        let processed = ocr::preprocess_rgba(image, &options);
        let _ = image::DynamicImage::ImageRgb8(processed).save(out);
        println!("saved processed preview to {out}");
    }
}
