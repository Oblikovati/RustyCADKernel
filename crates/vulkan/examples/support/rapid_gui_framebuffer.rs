//! Full-frame independent pixel expectations and atomic image publication after completed rendering.
use martensite_vulkan::FrameCapture;
use std::path::Path;
/// Assert every pixel, including the 300×110 GUI bounds; e.g. verify_capture(&frame, green, Some(cyan)).
pub fn verify_capture(frame: &FrameCapture, background: [u8; 4], gui: Option<[u8; 4]>) {
    verify_capture_shape(frame);
    let expected = expected_rgba(background, gui);
    let mismatch = frame
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(expected.as_chunks::<4>().0.iter())
        .position(|(a, b)| a != b);
    let colors = mismatch.map(|i| (&frame.rgba[i * 4..i * 4 + 4], &expected[i * 4..i * 4 + 4]));
    assert!(
        mismatch.is_none(),
        "framebuffer mismatch pixel={mismatch:?}, actual/expected={colors:?}; expected full independent 1440×900 RGBA image"
    );
}
fn verify_capture_shape(frame: &FrameCapture) {
    assert_eq!([frame.width, frame.height], [1440, 900]);
    assert_eq!(frame.rgba.len(), 1440 * 900 * 4);
    assert_eq!(
        frame.depth,
        vec![1.; 1440 * 900],
        "GUI changed empty-scene depth"
    );
}

fn expected_rgba(background: [u8; 4], gui: Option<[u8; 4]>) -> Vec<u8> {
    (0..1440 * 900)
        .flat_map(|index| {
            let inside = (8..308).contains(&(index % 1440)) && (8..118).contains(&(index / 1440));
            if inside {
                gui.unwrap_or(background)
            } else {
                background
            }
        })
        .collect()
}
/// Publish an inspectable PPM and raw RGBA atomically; e.g. save_capture(path,"rapid3gui",&frame).
pub fn save_capture(directory: &Path, name: &str, frame: &FrameCapture) {
    super::framebuffer_capture_image::publish_capture(
        &directory.join(name),
        frame,
        &super::framebuffer_capture_image::FileFrameImageStore,
    )
    .unwrap();
}
#[cfg(test)]
mod tests {
    #[test]
    fn exact_pixel_bounds_are_independent_of_capture() {
        let pixels = super::expected_rgba([0, 255, 0, 255], Some([0, 0, 255, 255]));
        let changed = pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|&&pixel| pixel == [0, 0, 255, 255])
            .count();
        assert_eq!(changed, 33000);
        for (x, y) in [(7, 8), (308, 8), (8, 7), (8, 118)] {
            let index = (y * 1440 + x) * 4;
            assert_eq!(&pixels[index..index + 4], &[0, 255, 0, 255]);
        }
    }
    #[test]
    #[should_panic(expected = "framebuffer mismatch")]
    fn old_green_frame_is_rejected_as_stale_gui() {
        let frame = martensite_vulkan::FrameCapture {
            width: 1440,
            height: 900,
            rgba: super::expected_rgba([0, 255, 0, 255], None),
            depth: vec![1.; 1440 * 900],
        };
        super::verify_capture(&frame, [0, 255, 0, 255], Some([0, 0, 255, 255]));
    }
    #[test]
    fn exact_full_frame_accepts_current_gui() {
        let frame = martensite_vulkan::FrameCapture {
            width: 1440,
            height: 900,
            rgba: super::expected_rgba([0, 255, 0, 255], Some([0, 0, 255, 255])),
            depth: vec![1.; 1440 * 900],
        };
        super::verify_capture(&frame, [0, 255, 0, 255], Some([0, 0, 255, 255]));
    }
}
