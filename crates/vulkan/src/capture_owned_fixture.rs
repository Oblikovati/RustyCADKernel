//! Hand-authored captures and expected logical rows, independent of renderer execution.
use crate::FrameCapture;
use rustycad_tkservice::{graphic3d::Graphic3dBufferType, image::ImageFormat};

pub(super) struct OwnedCaptureCase {
    pub kind: Graphic3dBufferType,
    pub format: ImageFormat,
    pub first: &'static [u8],
    pub second: &'static [u8],
}
pub(super) const OWNED_COLOR_CASES: [OwnedCaptureCase; 3] = [
    OwnedCaptureCase {
        kind: Graphic3dBufferType::Rgba,
        format: ImageFormat::Rgba,
        first: &[1, 2, 3, 4, 5, 6, 7, 8],
        second: &[9, 10, 11, 12, 13, 14, 15, 16],
    },
    OwnedCaptureCase {
        kind: Graphic3dBufferType::Rgb,
        format: ImageFormat::Rgb,
        first: &[1, 2, 3, 5, 6, 7],
        second: &[9, 10, 11, 13, 14, 15],
    },
    OwnedCaptureCase {
        kind: Graphic3dBufferType::Red,
        format: ImageFormat::Gray,
        first: &[1, 5],
        second: &[9, 13],
    },
];

pub(super) fn owned_frame_capture() -> FrameCapture {
    FrameCapture {
        width: 2,
        height: 2,
        rgba: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        depth: vec![0.0, 0.25, 0.5, 1.0],
    }
}

pub(super) fn owned_depth_rows() -> [Vec<u8>; 2] {
    // IEEE-754 encodings of 0, 1/4, 1/2 and 1 are independent expected values.
    [
        [0_u32.to_ne_bytes(), 0x3e80_0000_u32.to_ne_bytes()].concat(),
        [0x3f00_0000_u32.to_ne_bytes(), 0x3f80_0000_u32.to_ne_bytes()].concat(),
    ]
}
