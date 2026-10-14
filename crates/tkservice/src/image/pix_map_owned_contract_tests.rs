//! Owned fixtures pin Martensite allocation behavior before adding image source dispatch.
use super::{ImageFormat, ImagePixMap};
use rustycad_tkernel::ncollection::Vec3;

#[test]
fn owned_allocation_layouts_use_pixel_width_padding_and_slice_count() {
    let cases = [
        (ImageFormat::Gray, Vec3::new(2, 3, 1), 4, (4, 12, 12)),
        (ImageFormat::Rgb, Vec3::new(2, 2, 2), 0, (6, 12, 24)),
        (ImageFormat::Rgba, Vec3::new(3, 2, 1), 1, (12, 24, 24)),
    ];
    for (format, size, stride, expected) in cases {
        let image = ImagePixMap::default();
        assert!(image.init_trash_3d(format, size, stride));
        assert_eq!(image.size_xyz(), size);
        assert_eq!(image.format(), format);
        assert_eq!(image.size_row_bytes(), expected.0);
        assert_eq!(image.size_slice_bytes(), expected.1);
        assert_eq!(image.size_bytes(), expected.2);
        assert!(image.data().data.data().iter().all(|byte| *byte == 0));
    }
}

#[test]
fn owned_clear_keeps_format_and_row_order_while_releasing_all_pixels() {
    let image = ImagePixMap::default();
    image.set_top_down(true);
    assert!(image.init_zero_3d(ImageFormat::Rgb, Vec3::new(3, 2, 2), 12, 7));
    assert_eq!(image.data().data.data(), &[7; 48]);
    image.clear();
    assert_eq!(image.format(), ImageFormat::Rgb);
    assert!(image.is_top_down());
    assert!(image.is_empty());
    assert_eq!(image.size_xyz(), Vec3::new(0, 0, 0));
    assert_eq!(image.size_bytes(), 0);
}

#[test]
fn owned_zero_dimensions_refuse_after_clear_and_record_requested_format() {
    for size in [Vec3::new(0, 2, 1), Vec3::new(2, 0, 1), Vec3::new(2, 2, 0)] {
        let image = ImagePixMap::default();
        assert!(image.init_zero(ImageFormat::Gray, 2, 2, 0, 19));
        assert!(!image.init_trash_3d(ImageFormat::Rgba, size, 0));
        assert!(image.is_empty());
        assert_eq!(image.size_xyz(), Vec3::new(0, 0, 0));
        assert_eq!(image.format(), ImageFormat::Rgba);
    }
}

#[test]
fn owned_zero_fill_covers_padding_and_keeps_zero_depth_refusal() {
    let image = ImagePixMap::default();
    assert!(image.init_zero_3d(ImageFormat::Gray, Vec3::new(2, 2, 2), 4, 73));
    assert_eq!(image.data().data.data(), &[73; 16]);
    assert!(!image.init_zero_3d(ImageFormat::Gray, Vec3::new(2, 2, 0), 4, 91));
    assert!(image.is_empty());
    assert_eq!(image.size_bytes(), 0);
}
