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
use std::fmt::Write;
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
