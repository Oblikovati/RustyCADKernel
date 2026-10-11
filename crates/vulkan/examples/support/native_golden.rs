//! Device-bound native assets are replayed in Rust without OCCT linkage.
use martensite_vulkan::FrameCapture;
/// Check exact native RGB/alpha and bounded depth; `assert_capture(&capture, rgba, depth, mode)`.
pub fn assert_capture(capture: &FrameCapture, rgba: &[u8], depth: &[u8], mode: &str) {
    assert_eq!(
        capture.rgba.len(),
        rgba.len(),
        "received wrong byte count for {mode}; expected native shape"
    );
    let difference = capture
        .rgba
        .iter()
        .zip(rgba)
        .position(|(actual, native)| actual != native);
    assert!(
        difference.is_none(),
        "received different pixels for {mode} at byte {difference:?}; expected pinned native RGBA"
    );
    assert_golden_depth(&capture.depth, depth, mode);
}
/// Preserve the native depth gate independently of RGB; `assert_golden_depth(&values, bytes, mode)`.
pub fn assert_golden_depth(actual: &[f32], expected: &[u8], mode: &str) {
    assert_eq!(actual.len() * 4, expected.len());
    for (actual, bytes) in actual.iter().zip(expected.as_chunks::<4>().0) {
        let native = f32::from_le_bytes(*bytes);
        assert!(
            (actual - native).abs() <= 1e-6,
            "received {actual} depth for {mode}; expected {native} within 1e-6"
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_shape_and_depth_gates() {
        let capture = FrameCapture {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
            depth: vec![0.25],
        };
        assert_capture(&capture, &[0, 0, 0, 255], &0.25f32.to_le_bytes(), "test");
        assert_golden_depth(&[0.2500005], &0.25f32.to_le_bytes(), "test");
    }
}
