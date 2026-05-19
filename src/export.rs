//! Export the edited image as JPEG (native file or browser download).

use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, RgbImage};

use crate::adjust::apply_adjustments;

pub fn export_jpg(
    source: &image::RgbaImage,
    brightness: f32,
    saturation: f32,
) -> Result<Vec<u8>, String> {
    let mut img = source.clone();
    apply_adjustments(&mut img, brightness, saturation);

    // Convert RGBA to RGB by dropping the alpha channel (JPEG doesn't support alpha)
    let (width, height) = img.dimensions();
    let mut rgb_img = RgbImage::new(width, height);
    for (rgba_pixel, rgb_pixel) in img.pixels().zip(rgb_img.pixels_mut()) {
        rgb_pixel[0] = rgba_pixel[0];
        rgb_pixel[1] = rgba_pixel[1];
        rgb_pixel[2] = rgba_pixel[2];
    }

    let mut jpeg_bytes = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 92);
    image::ImageEncoder::write_image(
        encoder,
        rgb_img.as_raw(),
        width,
        height,
        ExtendedColorType::Rgb8,
    )
    .map_err(|e| format!("JPEG encode failed: {e}"))?;
    Ok(jpeg_bytes)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_jpg_to_disk(jpeg_bytes: &[u8], path: &str) -> Result<(), String> {
    std::fs::write(path, jpeg_bytes).map_err(|e| format!("Failed to write {path}: {e}"))
}

#[cfg(target_arch = "wasm32")]
pub fn download_jpg(jpeg_bytes: &[u8], filename: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let array = js_sys::Uint8Array::from(jpeg_bytes);
    let parts = js_sys::Array::new();
    parts.push(&array);

    let bag = BlobPropertyBag::new();
    bag.set_type("image/jpeg");
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &bag)
        .map_err(|_| "Failed to create blob")?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|_| "Failed to create object URL")?;

    let anchor = document
        .create_element("a")
        .map_err(|_| "Failed to create anchor")?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "Anchor was not HtmlAnchorElement")?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    Url::revoke_object_url(&url).ok();
    Ok(())
}
