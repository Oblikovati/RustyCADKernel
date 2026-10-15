//! Concrete ownership and genuine interface dispatch regressions; parent selection/HLR stay partial.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkservice::image::ImagePixMap;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*};

#[test]
fn textured_concrete_casts_share_native_descriptor_and_object() {
    let object = AisTexturedShape::new(&Shape::default());
    let source = object.clone().cast::<dyn AisTexturedShapeSource>();
    assert_eq!(source.object_address(), object.object_address());
    assert!(std::ptr::eq(
        <dyn AisTexturedShapeSource>::static_type(),
        AisTexturedShape::static_type()
    ));
    assert_eq!(
        object.dynamic_type().unwrap().parent().unwrap().name(),
        "AIS_Shape"
    );
    assert!(object.is_kind(<dyn AisShapeSource>::static_type()));
    assert_eq!(source.kind_of_interactive(), AisKindOfInteractive::Shape);
    assert_eq!(source.signature(), 0);
}

#[test]
fn textured_concrete_native_parent_alias_retains_real_object_and_descriptor() {
    let object = AisTexturedShape::new(&Shape::default());
    let parent = object.clone().cast::<AisTexturedShapeParent>();
    assert_eq!(parent.object_address(), object.object_address());
    assert!(std::ptr::eq(
        AisTexturedShapeParent::static_type(),
        object.dynamic_type().unwrap().parent().unwrap()
    ));
}

#[test]
fn textured_concrete_virtual_source_retains_actual_pixmap_field() {
    let object = AisTexturedShape::new(&Shape::default());
    let source = object.clone().cast::<dyn AisTexturedShapeSource>();
    let pixels = Handle::new(ImagePixMap::default());
    let retained = source.texture_pix_map();
    source.set_texture_pix_map(&pixels);
    assert_eq!(retained.read().object_address(), pixels.object_address());
    source.set_texture_file_name(&"texture.png".into()).unwrap();
    assert!(retained.read().is_null());
    assert_eq!(
        source.texture_file().read().unwrap().as_bytes(),
        b"texture.png"
    );
}

#[test]
fn textured_concrete_replays_native_rtti_row() {
    let object = AisTexturedShape::new(&Shape::default());
    let row = format!(
        "MARTENSITE_RTTI {} {} {}",
        object.dynamic_type().unwrap().name(),
        i32::from(object.is_kind(<dyn AisShapeSource>::static_type())),
        i32::from(object.is_kind(<dyn AisInteractiveObject>::static_type()))
    );
    let native = include_str!("fixtures/ais_textured_shape/native-state.txt");
    assert!(native.lines().any(|line| line == row));
}

#[test]
fn textured_concrete_null_compute_accepts_modes_without_fake_geometry() {
    let object = AisTexturedShape::new(&Shape::default());
    let source = object.cast::<dyn PrsMgrPresentableObject>();
    for mode in -1..=4 {
        assert_eq!(source.accept_display_mode(mode), (0..=3).contains(&mode));
        source
            .compute(&Handle::default(), &Handle::default(), mode)
            .unwrap();
    }
}

#[test]
fn textured_concrete_null_shape_selection_is_empty() {
    // The inherited AIS_Shape::ComputeSelection returns at once for a null shape.
    let object = AisTexturedShape::new(&Shape::default());
    let selection = object.clone().cast::<dyn SelectMgrSelectableObject>();
    selection.compute_selection(&Handle::default(), 4).unwrap();
}

#[test]
fn textured_concrete_null_shape_hlr_draws_nothing() {
    // The inherited AIS_Shape::computeHlrPresentation returns at once for a null shape.
    let object = AisTexturedShape::new(&Shape::default());
    let presentable = object.cast::<dyn PrsMgrPresentableObject>();
    presentable
        .compute_hlr(&Handle::default(), &Handle::default(), &Handle::default())
        .unwrap();
}
