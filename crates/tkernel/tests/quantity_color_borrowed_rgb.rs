//! The native Rgb accessor returns the stored components by reference.
use rustycad_tkernel::quantity::{Color, ColorSpace};

#[test]
fn borrowed_rgb_reuses_storage_across_calls() {
    let color: Color = Color::default();
    let first: &[f32; 3] = color.rgb_ref();
    let second: &[f32; 3] = color.rgb_ref();
    assert!(std::ptr::eq(first, second));
    assert_eq!(*first, color.rgb());
    assert_eq!(*first, [1.0, 1.0, 0.0]);
}

#[test]
fn borrowed_rgb_observes_linear_component_storage() {
    let color: Color = Color::new(0.25, 0.5, 0.75, ColorSpace::Rgb).unwrap();
    assert_eq!(*color.rgb_ref(), [0.25, 0.5, 0.75]);
    assert_eq!(*color.rgb_ref(), color.rgb_vec3().data());
}
