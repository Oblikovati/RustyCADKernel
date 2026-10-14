//! Owned fixtures pin the current capture converter before sharing its row-copy loop.
use super::capture_owned_fixture::{OWNED_COLOR_CASES, owned_depth_rows, owned_frame_capture};
use super::view_capture::dump_capture;
use rustycad_tkservice::{
    graphic3d::Graphic3dBufferType,
    image::{ImageFormat, ImagePixMap},
};

fn assert_owned_capture_rows(image: &ImagePixMap, first: &[u8], second: &[u8]) {
    let storage = image.data();
    let expected_first = [first, &[167; 3]].concat();
    let expected_second = [second, &[167; 3]].concat();
    assert_eq!(storage.data.row(0).unwrap(), expected_first);
    assert_eq!(storage.data.row(1).unwrap(), expected_second);
    let physical_first = if image.is_top_down() {
        expected_first
    } else {
        expected_second
    };
    assert_eq!(&storage.data.data()[..physical_first.len()], physical_first);
}

fn owned_capture_destination(format: ImageFormat, row_bytes: usize, top_down: bool) -> ImagePixMap {
    let image = ImagePixMap::default();
    image.set_top_down(top_down);
    assert!(image.init_zero(format, 2, 2, row_bytes + 3, 167));
    image
}

#[test]
fn owned_color_capture_rows_keep_padding_and_both_orientations() {
    for case in OWNED_COLOR_CASES {
        for top_down in [false, true] {
            let image = owned_capture_destination(case.format, case.first.len(), top_down);
            assert!(dump_capture(&image, &case.kind, &owned_frame_capture()));
            assert_owned_capture_rows(&image, case.first, case.second);
        }
    }
}

#[test]
fn owned_depth_capture_rows_keep_ieee_values_padding_and_orientation() {
    let rows = owned_depth_rows();
    for top_down in [false, true] {
        let image = owned_capture_destination(ImageFormat::GrayF, 8, top_down);
        assert!(dump_capture(
            &image,
            &Graphic3dBufferType::Depth,
            &owned_frame_capture()
        ));
        assert_owned_capture_rows(&image, &rows[0], &rows[1]);
    }
}

#[test]
fn owned_capture_rejections_leave_destination_bytes_unchanged() {
    let image = owned_capture_destination(ImageFormat::Rgba, 8, true);
    let original = image.data().clone();
    let mut capture = owned_frame_capture();
    capture.rgba.pop();
    assert!(!dump_capture(&image, &Graphic3dBufferType::Rgba, &capture));
    assert!(!dump_capture(
        &image,
        &Graphic3dBufferType::ShadowMap,
        &owned_frame_capture()
    ));
    assert!(!dump_capture(
        &image,
        &Graphic3dBufferType::Rgb,
        &owned_frame_capture()
    ));
    assert_eq!(*image.data(), original);
}
