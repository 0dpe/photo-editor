//! Decode DNG bytes to an 8-bit RGBA image (demosaic + simple tone mapping).

use demosaic::{demosaic_interleaved, Algorithm, CfaPattern, Channel};
use image::RgbaImage;
use rawler::decoders::RawDecodeParams;
use rawler::rawimage::RawImageData;
use rawler::rawsource::RawSource;
use rawler::CFA;

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
const DNG_FILE: &str = "photo_small.dng";

pub fn load_dng_bytes() -> Vec<u8> {
    #[cfg(target_arch = "wasm32")]
    {
        include_bytes!("../photo_small.dng").to_vec()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read(DNG_FILE).unwrap_or_else(|e| {
            panic!("Failed to read {DNG_FILE} from project root: {e}")
        })
    }
}

pub fn decode_dng_to_rgba(bytes: &[u8]) -> RgbaImage {
    let rawfile = RawSource::new_from_slice(bytes);
    let mut raw = rawler::decode(&rawfile, &RawDecodeParams::default()).unwrap_or_else(|e| {
        panic!("Failed to decode DNG: {e}")
    });
    raw.apply_scaling()
        .unwrap_or_else(|e| panic!("Failed to scale raw image: {e}"));

    let width = raw.width;
    let height = raw.height;
    if width == 0 || height == 0 {
        panic!("DNG has zero dimensions");
    }

    let pattern = cfa_to_demosaic(&raw.camera.cfa);

    let cfa_pixels = match &raw.data {
        RawImageData::Float(data) => data.clone(),
        RawImageData::Integer(data) => data
            .iter()
            .map(|&v| v as f32 / ((1u32 << raw.bps) - 1) as f32)
            .collect(),
    };

    let expected = width * height * raw.cpp;
    if cfa_pixels.len() != expected {
        panic!(
            "Unexpected raw buffer size: got {}, expected {} ({}x{}x{})",
            cfa_pixels.len(),
            expected,
            width,
            height,
            raw.cpp
        );
    }

    // Single-plane Bayer mosaic
    let mosaic: Vec<f32> = if raw.cpp == 1 {
        cfa_pixels
    } else {
        // Already multi-channel — pick first channel as fallback
        cfa_pixels
            .chunks(raw.cpp)
            .map(|px| px[0])
            .collect()
    };

    let mut rgb = vec![0.0f32; width * height * 3];
    demosaic_interleaved(
        &mosaic,
        width,
        height,
        &pattern,
        Algorithm::Bilinear,
        &mut rgb,
    )
    .unwrap_or_else(|e| panic!("Demosaic failed: {e}"));

    // Simple white balance using camera coefficients (skip ColorMatrix per spec).
    let mut wb = raw.neutralwb();
    if wb[0].is_nan() {
        wb = [1.0, 1.0, 1.0, 1.0];
    }
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) * 3;
            rgb[i] = (rgb[i] * wb[0]).clamp(0.0, 1.0);
            rgb[i + 1] = (rgb[i + 1] * wb[1]).clamp(0.0, 1.0);
            rgb[i + 2] = (rgb[i + 2] * wb[2]).clamp(0.0, 1.0);
        }
    }

    let mut rgba = RgbaImage::new(width as u32, height as u32);
    for (i, pixel) in rgba.pixels_mut().enumerate() {
        let j = i * 3;
        pixel[0] = (rgb[j].powf(1.0 / 2.4) * 255.0).round() as u8;
        pixel[1] = (rgb[j + 1].powf(1.0 / 2.4) * 255.0).round() as u8;
        pixel[2] = (rgb[j + 2].powf(1.0 / 2.4) * 255.0).round() as u8;
        pixel[3] = 255;
    }

    let (r, b, t, l) = match raw.camera.crop_area.or(raw.camera.active_area) {
        Some([r, b, t, l]) => (r as u32, b as u32, t as u32, l as u32),
        None => (0, 0, 0, 0),
    };

    if r + l < rgba.width() && t + b < rgba.height() && (r > 0 || l > 0 || t > 0 || b > 0) {
        let crop_w = rgba.width() - l - r;
        let crop_h = rgba.height() - t - b;
        image::imageops::crop_imm(&rgba, l, t, crop_w, crop_h).to_image()
    } else {
        rgba
    }
}

fn cfa_to_demosaic(cfa: &CFA) -> CfaPattern {
    let name = cfa.to_string();
    match name.as_str() {
        "RGGB" => CfaPattern::bayer_rggb(),
        "BGGR" => CfaPattern::bayer_bggr(),
        "GRBG" => CfaPattern::bayer_grbg(),
        "GBRG" => CfaPattern::bayer_gbrg(),
        other if other.len() == 16 => match other {
            "RRGGGGBB" => CfaPattern::quad_bayer_rggb(),
            "BBGGRRGG" => CfaPattern::quad_bayer_bggr(),
            "GRGRBRBR" => CfaPattern::quad_bayer_grbg(),
            "GBGBRBRB" => CfaPattern::quad_bayer_gbrg(),
            _ => CfaPattern::bayer_rggb(),
        },
        _ if cfa.width == 6 && cfa.height == 6 => {
            let map = |c: u8| match c {
                0 => Channel::Red,
                1 => Channel::Green,
                2 => Channel::Blue,
                _ => Channel::Green,
            };
            let flat = cfa.flat_pattern();
            let mut pat = [Channel::Green; 36];
            for (i, &c) in flat.iter().enumerate() {
                pat[i] = map(c);
            }
            CfaPattern::xtrans(pat)
        }
        _ => CfaPattern::bayer_rggb(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_photo_small_dng() {
        let bytes = std::fs::read(DNG_FILE).expect("photo_small.dng in project root");
        let img = decode_dng_to_rgba(&bytes);
        assert!(img.width() > 0 && img.height() > 0);
    }

    #[test]
    fn debug_dng() {
        let bytes = std::fs::read(DNG_FILE).unwrap();
        let rawfile = RawSource::new_from_slice(&bytes);
        let mut raw = rawler::decode(&rawfile, &RawDecodeParams::default()).unwrap();
        raw.apply_scaling().unwrap();
        let (min, max, sum) = match &raw.data {
            RawImageData::Float(data) => (
                data.iter().copied().fold(f32::INFINITY, f32::min),
                data.iter().copied().fold(f32::NEG_INFINITY, f32::max),
                data.iter().copied().sum::<f32>()
            ),
            RawImageData::Integer(_) => (0.0, 0.0, 0.0)
        };
        let len = match &raw.data {
            RawImageData::Float(data) => data.len(),
            _ => 1
        };
        println!("Min: {}, Max: {}, Avg: {}", min, max, sum / len as f32);
        println!("wb: {:?}", raw.neutralwb());
        println!("Camera: {:#?}", raw.camera);
    }
}