use codimate_render::Bitmap;

use crate::ExportError;
use codimate_animation::Playable;
use codimate_layout::{layout_scene, Viewport};
use codimate_render::{rasterize, render_frame};
use std::io;
use std::path::Path;

/// Pure: encode a `Bitmap` to PNG bytes. No I/O, deterministic.
#[must_use]
pub fn encode_png(bitmap: &Bitmap) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, bitmap.width, bitmap.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .expect("writing to a Vec does not fail");
    writer
        .write_image_data(&bitmap.rgba)
        .expect("rgba matches width*height*4");
    writer.finish().expect("writing to a Vec does not fail");
    bytes
}

/// Encode `bitmap` and write the PNG to `path`. The only I/O here.
pub fn write_png(path: impl AsRef<Path>, bitmap: &Bitmap) -> io::Result<()> {
    std::fs::write(path, encode_png(bitmap))
}

/// Pure: encode a `Bitmap` to JPEG bytes. JPEG has no alpha channel, so pixels
/// are composited over black before encoding.
pub fn encode_jpeg(bitmap: &Bitmap, quality: u8) -> Result<Vec<u8>, ExportError> {
    let mut rgb = Vec::with_capacity((bitmap.width * bitmap.height * 3) as usize);
    for px in bitmap.rgba.chunks_exact(4) {
        let alpha = px[3] as u16;
        rgb.push(((px[0] as u16 * alpha) / 255).min(255) as u8);
        rgb.push(((px[1] as u16 * alpha) / 255).min(255) as u8);
        rgb.push(((px[2] as u16 * alpha) / 255).min(255) as u8);
    }

    let mut bytes = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut bytes, quality);
    encoder
        .encode(
            &rgb,
            bitmap.width as u16,
            bitmap.height as u16,
            jpeg_encoder::ColorType::Rgb,
        )
        .map_err(io::Error::other)?;
    Ok(bytes)
}

/// Encode `bitmap` and write the JPEG to `path`. The only I/O here.
pub fn write_jpeg(path: impl AsRef<Path>, bitmap: &Bitmap, quality: u8) -> Result<(), ExportError> {
    std::fs::write(path, encode_jpeg(bitmap, quality)?)?;
    Ok(())
}

/// Render one frame of `playable` at `seconds` and save it as a PNG — for
/// eyeballing a single moment without dumping a whole sequence to disk.
pub fn export_frame_png(
    playable: &(impl Playable + ?Sized),
    seconds: f32,
    viewport: Viewport,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    let scene = playable.resolve_at(seconds);
    let layout = layout_scene(scene, viewport);
    let frame = render_frame(playable.name(), seconds, &layout);
    let bitmap = rasterize(&frame);
    write_png(path, &bitmap)
}

/// Render one frame of `playable` at `seconds` and save it as a JPEG — for
/// eyeballing a single moment without dumping a whole sequence to disk.
pub fn export_frame_jpeg(
    playable: &(impl Playable + ?Sized),
    seconds: f32,
    viewport: Viewport,
    path: impl AsRef<Path>,
    quality: u8,
) -> Result<(), ExportError> {
    let scene = playable.resolve_at(seconds);
    let layout = layout_scene(scene, viewport);
    let frame = render_frame(playable.name(), seconds, &layout);
    let bitmap = rasterize(&frame);
    write_jpeg(path, &bitmap, quality)
}
