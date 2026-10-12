//! Native parent-facing transparency and failure publication order on real textured state.
#[allow(dead_code)]
#[path = "support/textured_shape_state_fixture.rs"]
mod state_fixture;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer};
use state_fixture::TexturedStateFixture;

fn transparency_fixture() -> Handle<TexturedStateFixture> {
    let object = TexturedStateFixture::new();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    object.attributes().link_to(&linked);
    object
}
#[test]
fn textured_parent_transparency_uses_current_facing_and_releases_unowned_shading() {
    let sides = [
        AspectTypeOfFacingModel::BothSide,
        AspectTypeOfFacingModel::BackSide,
        AspectTypeOfFacingModel::FrontSide,
    ];
    for (index, side) in sides.into_iter().enumerate() {
        let object = transparency_fixture();
        object.set_current_facing_model(side);
        ais_shape_set_transparency(&*object, 0.375).unwrap();
        compare_transparency(&object, &format!("side{index}"));
        ais_shape_unset_transparency(&*object).unwrap();
        compare_transparency(&object, &format!("unset{index}"));
    }
}
#[test]
fn textured_parent_transparency_retains_color_and_rejects_before_scalar_publication() {
    let object = transparency_fixture();
    ais_shape_set_color(&*object, Color::named(NamedColor::RED)).unwrap();
    object.set_current_facing_model(AspectTypeOfFacingModel::FrontSide);
    ais_shape_set_transparency(&*object, 0.375).unwrap();
    ais_shape_unset_transparency(&*object).unwrap();
    compare_transparency(&object, "color");
    let failed = transparency_fixture();
    assert!(ais_shape_set_transparency(&*failed, 1.25).is_err());
    compare_transparency(&failed, "failure");
}
fn compare_transparency(object: &TexturedStateFixture, label: &str) {
    let drawer = object.attributes();
    let shading = drawer.shading_aspect();
    let actual = [
        f64::from(i32::from(drawer.has_own_shading_aspect())),
        f64::from(drawer.transparency()),
        shading.transparency(AspectTypeOfFacingModel::FrontSide),
        shading.transparency(AspectTypeOfFacingModel::BackSide),
    ];
    assert_eq!(
        actual.as_slice(),
        native_transparency(label),
        "native transparency {label}"
    );
}

fn native_transparency(label: &str) -> Vec<f64> {
    let prefix = format!("MARTENSITE_TRANSPARENCY {label} ");
    let native = include_str!("fixtures/ais_textured_shape/native-transparency.txt");
    native
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}

#[test]
fn textured_parent_transparency_preserves_owned_material() {
    let material = transparency_fixture();
    let gold = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
    ais_shape_set_material(&*material, &gold).unwrap();
    ais_shape_set_transparency(&*material, 0.375).unwrap();
    ais_shape_unset_transparency(&*material).unwrap();
    compare_transparency(&material, "material");
}

#[test]
fn textured_parent_transparency_keeps_mapping_and_skips_unowned_shading() {
    let texture = transparency_fixture();
    texture
        .attributes()
        .setup_own_shading_aspect(&Handle::default())
        .unwrap();
    texture
        .attributes()
        .shading_aspect()
        .aspect()
        .set_texture_map_on(true);
    ais_shape_set_transparency(&*texture, 0.375).unwrap();
    ais_shape_unset_transparency(&*texture).unwrap();
    compare_transparency(&texture, "texture");
    let empty = transparency_fixture();
    ais_shape_unset_transparency(&*empty).unwrap();
    compare_transparency(&empty, "empty");
}
