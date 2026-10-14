//! Native copy transfers conditional global state, never custom nodes or parent drawer identity.
use super::{
    colored_shape_appearance_snapshot::*, colored_shape_expansion_inputs::ColoredExpansionInputs, *,
};
use crate::prs_mgr::*;
use rustycad_tkernel::standard::Transient;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use std::{fmt::Write, sync::atomic::Ordering};
fn copy_object(input: &ColoredExpansionInputs) -> Handle<AisColoredShape> {
    let object = AisColoredShape::new(&input.base);
    object.attributes().setup_own_defaults().unwrap();
    object
}
fn copy_state(rows: &mut String, label: &str, source: &Handle<AisColoredShape>) {
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let copy = AisColoredShape::from_shape(&parent).unwrap();
    appearance_parent(rows, label, &copy);
    appearance_color(rows, copy.color());
    let own = copy.attributes();
    writeln!(
        rows,
        " {} {} {} {}",
        copy.material() as i32,
        i32::from(copy.shape_ref().is_equal(&source.shape_ref().clone_shape())),
        i32::from(own == source.attributes()),
        i32::from(own.line_aspect() == own.free_boundary_aspect())
    )
    .unwrap();
}
fn copy_owned_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = copy_object(input);
    copy_state(rows, "default-copy", &object);
    object
        .set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))
        .unwrap();
    object.set_color(Color::named(NamedColor::RED)).unwrap();
    object.set_width(2.5).unwrap();
    object.set_transparency(0.4).unwrap();
    object
        .set_custom_color(&input.edges[0], Color::named(NamedColor::BLUE))
        .unwrap();
    copy_state(rows, "owned-copy", &object);
    object.unset_width().unwrap();
    object.unset_transparency().unwrap();
    copy_state(rows, "unset-copy", &object);
}
fn copy_facing_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = copy_object(input);
    object.set_color(Color::named(NamedColor::RED)).unwrap();
    object.set_current_facing_model(AspectTypeOfFacingModel::BackSide);
    let shading = object.attributes().shading_aspect();
    configure_copy_back_material(&shading);
    shading
        .set_transparency(0.001, AspectTypeOfFacingModel::BackSide)
        .unwrap();
    copy_state(rows, "tiny-back-copy", &object);
    copy_facing_queries(rows, "tiny-back-queries", &*object);
    shading
        .set_transparency(0.25, AspectTypeOfFacingModel::BackSide)
        .unwrap();
    copy_state(rows, "back-copy", &object);
    copy_facing_queries(rows, "back-queries", &*object);
}
#[test]
fn actual_native_copy_and_facing_getters_preserve_owned_state_and_independence() {
    let input = ColoredExpansionInputs::new();
    let mut rows = String::new();
    copy_owned_cases(&mut rows, &input);
    copy_facing_cases(&mut rows, &input);
    textured_facing_queries(&mut rows, &input);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-expansion/native-copy.txt")
    );
}

fn configure_copy_back_material(shading: &crate::prs3d::Prs3dShadingAspect) {
    shading.set_material(
        &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Silver),
        AspectTypeOfFacingModel::BackSide,
    );
    shading
        .set_color(
            Color::named(NamedColor::BLUE),
            AspectTypeOfFacingModel::BackSide,
        )
        .unwrap();
}

fn copy_facing_queries<S: AisShapeSource + ?Sized>(rows: &mut String, label: &str, object: &S) {
    write!(rows, "{label}").unwrap();
    let mut color = Color::named(NamedColor::GREEN);
    ais_shape_color_into(object, &mut color);
    appearance_color(rows, color);
    writeln!(
        rows,
        " {} {}",
        object.material() as i32,
        (object.transparency() * 1000.0).round() as i64
    )
    .unwrap();
}

fn textured_facing_queries(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = AisTexturedShape::new(&input.base);
    object.attributes().setup_own_defaults().unwrap();
    object.set_current_facing_model(AspectTypeOfFacingModel::BackSide);
    let shading = object.attributes().shading_aspect();
    configure_copy_back_material(&shading);
    shading
        .set_transparency(0.001, AspectTypeOfFacingModel::BackSide)
        .unwrap();
    copy_facing_queries(rows, "textured-back-queries", &*object);
}

fn copy_failure_source(
    input: &ColoredExpansionInputs,
) -> Handle<super::colored_shape_dispatch_fixture::ColoredDispatchFake> {
    let source = super::colored_shape_dispatch_fixture::ColoredDispatchFake::new(&input.base);
    source.attributes().setup_own_defaults().unwrap();
    ais_shape_set_material(
        &*source,
        &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold),
    )
    .unwrap();
    ais_shape_set_color(&*source, Color::named(NamedColor::RED)).unwrap();
    ais_shape_set_transparency(&*source, 0.4).unwrap();
    source
}
fn copy_failure_case(
    rows: &mut String,
    source: &Handle<super::colored_shape_dispatch_fixture::ColoredDispatchFake>,
    stage: i32,
) {
    use std::sync::atomic::Ordering;
    source.failure.store(stage, Ordering::Relaxed);
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let error = AisColoredShape::from_shape(&parent)
        .expect_err("copy getter failure missing; expected Standard_Failure");
    copy_failure_ownership(rows, source, stage, &error);
    copy_repair_ownership(rows, source, &parent, stage);
}
#[test]
fn native_copy_virtual_getter_failures_release_partial_objects_then_repair() {
    let input = ColoredExpansionInputs::new();
    let source = copy_failure_source(&input);
    let mut rows = String::new();
    for stage in [3, 4, 5] {
        copy_failure_case(&mut rows, &source, stage);
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-expansion/native-copy-failure.txt")
    );
}

fn copy_failure_ownership(
    rows: &mut String,
    source: &super::colored_shape_dispatch_fixture::ColoredDispatchFake,
    stage: i32,
    error: &rustycad_tkernel::standard::Failure,
) {
    let shape_count = source.shape_base().fields.read().shape.tshape().ref_count();
    let drawer_count = source
        .attributes_ref()
        .with_object(|drawer| drawer.unwrap().transient_base().ref_count());
    writeln!(
        rows,
        "copy-failure {stage} {} {shape_count} {drawer_count} {}",
        error.exception_type(),
        source.transient_base().ref_count()
    )
    .unwrap();
}

fn copy_repair_ownership(
    rows: &mut String,
    source: &super::colored_shape_dispatch_fixture::ColoredDispatchFake,
    parent: &Handle<dyn AisShapeSource>,
    stage: i32,
) {
    use std::sync::atomic::Ordering;
    source.failure.store(0, Ordering::Relaxed);
    let copy = AisColoredShape::from_shape(parent).unwrap();
    let shape_count = source.shape_base().fields.read().shape.tshape().ref_count();
    writeln!(
        rows,
        "copy-repaired {stage} {shape_count} {} {}",
        copy.custom_aspects_map().len(),
        i32::from(copy.has_color())
    )
    .unwrap();
}

#[test]
fn copy_constructor_resumes_unrelated_source_panics_without_masking_them() {
    let input = ColoredExpansionInputs::new();
    let source = copy_failure_source(&input);
    source
        .failure
        .store(6, std::sync::atomic::Ordering::Relaxed);
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        AisColoredShape::from_shape(&parent)
    }))
    .expect_err("copy source panic missing; expected original payload");
    assert_eq!(
        *panic.downcast::<&str>().unwrap(),
        "unrelated copy-source panic"
    );
}

#[test]
fn copy_contract_source_replacement_preserves_copied_shape_and_color() {
    let input = ColoredExpansionInputs::new();
    let source = copy_object(&input);
    source.set_color(Color::named(NamedColor::RED)).unwrap();
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let copy = AisColoredShape::from_shape(&parent).unwrap();
    source.set_shape(&input.external);
    source.set_color(Color::named(NamedColor::BLUE)).unwrap();
    source.set_custom_width(&input.edges[0], 7.0).unwrap();
    assert!(copy.shape_ref().is_equal(&input.base));
    assert!(source.shape_ref().is_equal(&input.external));
    assert_eq!(copy.color(), Color::named(NamedColor::RED));
    assert!(copy.custom_aspects_map().is_empty());
    assert_eq!(source.custom_aspects_map().len(), 1);
}

#[test]
fn copy_contract_copy_mutation_preserves_source_appearance_and_map() {
    let input = ColoredExpansionInputs::new();
    let source = copy_object(&input);
    source.set_color(Color::named(NamedColor::RED)).unwrap();
    source.set_width(2.5).unwrap();
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let copy = AisColoredShape::from_shape(&parent).unwrap();
    copy.set_color(Color::named(NamedColor::BLUE)).unwrap();
    copy.set_width(7.0).unwrap();
    copy.set_custom_width(&input.edges[0], 9.0).unwrap();
    assert_eq!(source.color(), Color::named(NamedColor::RED));
    assert_eq!(source.width(), 2.5);
    assert!(source.custom_aspects_map().is_empty());
    assert_eq!(copy.custom_aspects_map().len(), 1);
}

#[test]
fn copy_contract_retains_shape_and_appearance_after_source_drop() {
    let input = ColoredExpansionInputs::new();
    let source = copy_object(&input);
    source.set_color(Color::named(NamedColor::RED)).unwrap();
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let copy = AisColoredShape::from_shape(&parent).unwrap();
    drop(parent);
    drop(source);
    assert!(copy.shape_ref().is_equal(&input.base));
    assert_eq!(copy.color(), Color::named(NamedColor::RED));
    copy.set_width(4.0).unwrap();
    assert_eq!(copy.width(), 4.0);
}

#[test]
fn copy_contract_topology_payload_remains_shared_and_mutable() {
    let input = ColoredExpansionInputs::new();
    let source = copy_object(&input);
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let copy = AisColoredShape::from_shape(&parent).unwrap();
    let source_shape = source.shape_ref().clone_shape();
    let copy_shape = copy.shape_ref().clone_shape();
    // Pin this limitation so presentation copying cannot be mistaken for kernel isolation.
    assert!(source_shape.is_partner(&copy_shape));
    source_shape.set_checked(true).unwrap();
    assert!(copy_shape.checked().unwrap());
    copy_shape.set_checked(false).unwrap();
    assert!(!source_shape.checked().unwrap());
}

#[test]
fn copy_contract_failed_getters_release_topology_and_allow_reuse() {
    let input = ColoredExpansionInputs::new();
    let source = copy_failure_source(&input);
    let parent: Handle<dyn AisShapeSource> = source.cast();
    let topology_owners = input.base.tshape().ref_count();
    for stage in [3, 4, 5, 6] {
        copy_contract_failure_attempt(&source, &parent, stage);
        assert_eq!(input.base.tshape().ref_count(), topology_owners);
        source.failure.store(0, Ordering::Relaxed);
        assert!(source.shape_ref().is_equal(&input.base));
        assert_eq!(source.color(), Color::named(NamedColor::RED));
        let copy = AisColoredShape::from_shape(&parent).unwrap();
        assert!(copy.shape_ref().is_equal(&input.base));
        assert_eq!(copy.color(), Color::named(NamedColor::RED));
        assert_eq!(copy.transparency(), f64::from(0.4_f32));
    }
    assert_eq!(input.base.tshape().ref_count(), topology_owners);
}

fn copy_contract_failure_attempt(
    source: &super::colored_shape_dispatch_fixture::ColoredDispatchFake,
    parent: &Handle<dyn AisShapeSource>,
    stage: i32,
) {
    source.failure.store(stage, Ordering::Relaxed);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        AisColoredShape::from_shape(parent)
    }));
    if stage == 6 {
        let payload = outcome.expect_err("stage=6; expected original foreign panic");
        let message = payload.downcast::<&str>().unwrap();
        assert_eq!(*message, "unrelated copy-source panic");
        return;
    }
    let error = outcome
        .unwrap()
        .expect_err("stage=3..=5; expected typed getter failure");
    assert_eq!(error.exception_type(), "Standard_Failure");
}
