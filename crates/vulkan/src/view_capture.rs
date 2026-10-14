//! Copies checked GPU captures into caller-owned Image_PixMap rows, including padded/bottom-up images.
use crate::FrameCapture;
use rustycad_tkservice::{
    graphic3d::Graphic3dBufferType,
    image::{ImageFormat, ImagePixMap, ImagePixMapData},
};
pub(crate) fn dump_capture(
    image: &ImagePixMap,
    kind: &Graphic3dBufferType,
    capture: &FrameCapture,
) -> bool {
    if checked_capture_pixels(capture).is_none()
        || image.width() != capture.width as usize
        || image.height() != capture.height as usize
    {
        return false;
    }
    let Some(bytes) = capture_bytes(kind, image.format(), capture) else {
        return false;
    };
    let row_size = capture.width as usize * image.size_pixel_bytes();
    let mut target = image.change_data();
    copy_capture_rows(&mut target.data, &bytes, row_size)
}
pub(crate) fn copy_capture_rows(
    target: &mut ImagePixMapData,
    bytes: &[u8],
    row_size: usize,
) -> bool {
    for (y, row) in bytes.chunks_exact(row_size).enumerate() {
        let Some(destination) = target.change_row(y) else {
            return false;
        };
        destination[..row_size].copy_from_slice(row);
    }
    true
}
pub(crate) fn capture_bytes(
    kind: &Graphic3dBufferType,
    format: ImageFormat,
    capture: &FrameCapture,
) -> Option<Vec<u8>> {
    if format != capture_image_format(kind)? {
        return None;
    }
    match kind {
        Graphic3dBufferType::Rgba => Some(capture.rgba.clone()),
        Graphic3dBufferType::Rgb => Some(color_channels(capture, 3)),
        Graphic3dBufferType::Red => Some(color_channels(capture, 1)),
        Graphic3dBufferType::Depth => {
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
pub(crate) fn checked_capture_pixels(capture: &FrameCapture) -> Option<usize> {
    let pixels = (capture.width as usize).checked_mul(capture.height as usize)?;
    (pixels != 0
        && pixels.checked_mul(4) == Some(capture.rgba.len())
        && capture.depth.len() == pixels)
        .then_some(pixels)
}

pub(crate) fn capture_image_format(kind: &Graphic3dBufferType) -> Option<ImageFormat> {
    match kind {
        Graphic3dBufferType::Rgba => Some(ImageFormat::Rgba),
        Graphic3dBufferType::Rgb => Some(ImageFormat::Rgb),
        Graphic3dBufferType::Red => Some(ImageFormat::Gray),
        Graphic3dBufferType::Depth => Some(ImageFormat::GrayF),
        _ => None,
    }
}
