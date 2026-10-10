//! Copies checked GPU captures into caller-owned Image_PixMap rows, including padded/bottom-up images.
use crate::FrameCapture;
use rustycad_tkservice::{
    graphic3d::Graphic3dBufferType,
    image::{ImageFormat, ImagePixMap},
};
pub(crate) fn dump_capture(
    image: &ImagePixMap,
    kind: &Graphic3dBufferType,
    capture: &FrameCapture,
) -> bool {
    let Some(pixels) = (capture.width as usize).checked_mul(capture.height as usize) else {
        return false;
    };
    if pixels == 0 || !capture_shape_matches(image, capture, pixels) {
        return false;
    }
    let Some(bytes) = capture_bytes(kind, image.format(), capture) else {
        return false;
    };
    let row_size = capture.width as usize * image.size_pixel_bytes();
    let mut target = image.change_data();
    for (y, row) in bytes.chunks_exact(row_size).enumerate() {
        let Some(destination) = target.data.change_row(y) else {
            return false;
        };
        destination[..row_size].copy_from_slice(row);
    }
    true
}
fn capture_bytes(
    kind: &Graphic3dBufferType,
    format: ImageFormat,
    capture: &FrameCapture,
) -> Option<Vec<u8>> {
    match (kind, format) {
        (Graphic3dBufferType::Rgba, ImageFormat::Rgba) => Some(capture.rgba.clone()),
        (Graphic3dBufferType::Rgb, ImageFormat::Rgb) => Some(color_channels(capture, 3)),
        (Graphic3dBufferType::Red, ImageFormat::Gray) => Some(color_channels(capture, 1)),
        (Graphic3dBufferType::Depth, ImageFormat::GrayF) => {
            Some(capture.depth.iter().flat_map(|v| v.to_ne_bytes()).collect())
        }
        _ => None,
    }
}

fn color_channels(capture: &FrameCapture, count: usize) -> Vec<u8> {
    capture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|pixel| pixel[..count].iter().copied())
        .collect()
}
fn capture_shape_matches(image: &ImagePixMap, capture: &FrameCapture, pixels: usize) -> bool {
    pixels.checked_mul(4) == Some(capture.rgba.len())
        && capture.depth.len() == pixels
        && image.width() == capture.width as usize
        && image.height() == capture.height as usize
}
