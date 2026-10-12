//! Native Texture2D ancestry must not contain a synthetic same-name source-interface parent.
use super::*;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
#[test]
fn native_texture_2d_core_plane_and_source_descriptor_ancestry() {
    let plain = Handle::new(Graphic3dTexture2D::from_file(&"".into()));
    let plane = Handle::new(Graphic3dTexture2Dplane::from_file(&"".into()));
    let row = format!(
        "{} {} {} {} {} {}",
        plain.dynamic_type().unwrap().name(),
        plain.dynamic_type().unwrap().parent().unwrap().name(),
        plane.dynamic_type().unwrap().name(),
        plane.dynamic_type().unwrap().parent().unwrap().name(),
        i32::from(plain.is_kind(<dyn Graphic3dTextureMap>::static_type())),
        i32::from(plane.is_kind(Graphic3dTexture2D::static_type()))
    );
    assert_eq!(
        row,
        include_str!("../../tests/fixtures/texture-2d-native-rtti.txt").trim()
    );
    assert!(std::ptr::eq(
        <dyn Graphic3dTexture2DSource>::static_type(),
        Graphic3dTexture2D::static_type()
    ));
    assert_eq!(
        plain
            .clone()
            .cast::<dyn Graphic3dTexture2DSource>()
            .object_address(),
        plain.object_address()
    );
    assert_eq!(
        plane
            .clone()
            .cast::<dyn Graphic3dTexture2DSource>()
            .object_address(),
        plane.object_address()
    );
}
