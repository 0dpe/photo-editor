//! CPU-side color adjustments (matches the WGSL shader for export).

use image::RgbaImage;

pub fn apply_adjustments(img: &mut RgbaImage, brightness: f32, saturation: f32) {
    let brightness = brightness.max(0.0);
    let saturation = saturation.max(0.0);
    for pixel in img.pixels_mut() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        let exposed = [r * brightness, g * brightness, b * brightness];
        let luma = 0.2126 * exposed[0] + 0.7152 * exposed[1] + 0.0722 * exposed[2];
        let out = |c: f32| (luma + (c - luma) * saturation).clamp(0.0, 1.0);
        pixel[0] = (out(exposed[0]) * 255.0).round() as u8;
        pixel[1] = (out(exposed[1]) * 255.0).round() as u8;
        pixel[2] = (out(exposed[2]) * 255.0).round() as u8;
    }
}
