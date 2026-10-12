//! Pinned native source-rebinding and unwind rows prove owning boundaries and later slot rereads.
use super::{
    colored_copy_source_fixture::*, colored_shape_expansion_inputs::ColoredExpansionInputs, *,
};
use crate::prs_mgr::*;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::*,
};
use rustycad_tkservice::graphic3d::*;
use std::{fmt::Write, sync::Arc};
fn source_appearance<S: AisShapeSource + ?Sized>(object: &S, incoming: bool) {
    object.attributes().setup_own_defaults().unwrap();
    let material = if incoming {
        Graphic3dNameOfMaterial::Silver
    } else {
        Graphic3dNameOfMaterial::Gold
    };
    ais_shape_set_material(object, &Graphic3dMaterialAspect::new(material)).unwrap();
    let color = if incoming {
        NamedColor::BLUE
    } else {
        NamedColor::RED
    };
    ais_shape_set_color(object, Color::named(color)).unwrap();
    ais_shape_set_width(object, if incoming { 5.0 } else { 2.5 }).unwrap();
    ais_shape_set_transparency(object, if incoming { 0.7 } else { 0.4 }).unwrap();
}
fn copy_source_input(
    incoming: &Handle<dyn AisShapeSource>,
    stage: i32,
    failure: i32,
) -> Arc<CopySourceInput> {
    Arc::new(CopySourceInput {
        slot: SharedState::default(),
        incoming: incoming.downgrade(),
        rows: SharedState::default(),
        stage,
        failure,
    })
}
fn record_source_result(
    rows: &mut String,
    label: &str,
    copy: &AisColoredShape,
    input: &ColoredExpansionInputs,
) {
    write!(
        rows,
        "{label} {} {}",
        i32::from(copy.shape_ref().is_equal(&input.base)),
        i32::from(copy.shape_ref().is_equal(&input.external)),
    )
    .unwrap();
    record_copy_appearance(rows, copy);
}
fn record_copy_appearance(rows: &mut String, copy: &AisColoredShape) {
    write!(
        rows,
        " {} {} {} {} {}",
        i32::from(copy.has_material()),
        i32::from(copy.has_color()),
        i32::from(copy.has_width()),
        (copy.width() * 1000.0).round() as i64,
        (copy.transparency() * 1000.0).round() as i64,
    )
    .unwrap();
    super::colored_shape_appearance_snapshot::appearance_color(rows, copy.color());
    writeln!(
        rows,
        " {} {}",
        copy.material() as i32,
        copy.custom_aspects_map().len()
    )
    .unwrap();
}
fn attempt_source_copy(
    slot: &dyn HandleReference<dyn AisShapeSource>,
    input: &ColoredExpansionInputs,
) -> String {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        AisColoredShape::from_shape(slot)
    }));
    let mut rows = String::new();
    match result {
        Ok(Ok(copy)) => record_source_result(&mut rows, "copy-result", &copy, input),
        Ok(Err(error)) => writeln!(rows, "copy-failure {}", error.exception_type()).unwrap(),
        Err(payload) => record_source_panic(&mut rows, payload),
    }
    rows
}
fn record_source_panic(rows: &mut String, payload: Box<dyn std::any::Any + Send>) {
    assert_eq!(
        *payload.downcast::<&str>().unwrap(),
        "unrelated copy-source panic"
    );
    writeln!(rows, "copy-failure unrelated").unwrap();
}
fn record_source_after(
    rows: &mut String,
    original: &CopySourceProbe,
    incoming: &Handle<dyn AisShapeSource>,
    input: &ColoredExpansionInputs,
) {
    writeln!(
        rows,
        "after-copy {} {} {} {}",
        original.transient_base().ref_count(),
        incoming.ref_count(),
        input.base.tshape().ref_count(),
        input.external.tshape().ref_count(),
    )
    .unwrap();
}
struct CopySourceScene {
    original: Handle<CopySourceProbe>,
    input: Arc<CopySourceInput>,
    incoming: Handle<dyn AisShapeSource>,
}
fn source_case_objects(
    input: &ColoredExpansionInputs,
    stage: i32,
    owned: bool,
    failure: i32,
) -> CopySourceScene {
    let incoming: Handle<dyn AisShapeSource> = AisColoredShape::new(&input.external).cast();
    if owned {
        source_appearance(&*incoming, true);
    }
    let probe_input = copy_source_input(&incoming, stage, failure);
    let original = CopySourceProbe::new(&input.base, &probe_input);
    *probe_input.slot.write() = original.cast();
    source_appearance(&*original, false);
    CopySourceScene {
        original,
        input: probe_input,
        incoming,
    }
}
fn source_copy_case(
    rows: &mut String,
    input: &ColoredExpansionInputs,
    stage: i32,
    owned: bool,
    failure: i32,
) {
    let scene = source_case_objects(input, stage, owned, failure);
    writeln!(rows, "case {stage} {} {failure}", i32::from(owned)).unwrap();
    let slot = HandleFieldRef::new(&scene.input.slot, copy_source_slot);
    let result = attempt_source_copy(&slot, input);
    rows.push_str(&scene.input.rows.read());
    rows.push_str(&result);
    record_source_after(rows, &scene.original, &scene.incoming, input);
    let repair = AisColoredShape::from_shape(&slot).unwrap();
    record_source_result(rows, "repair-result", &repair, input);
}
#[test]
fn native_copy_source_slot_replacement_failures_and_repair_replay() {
    let input = ColoredExpansionInputs::new();
    let mut rows = String::new();
    for (stage, owned, failure) in source_copy_scenarios() {
        source_copy_case(&mut rows, &input, stage, owned, failure);
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-expansion/native-copy-source.txt")
    );
}
const COPY_SOURCE_SCENARIOS: [(i32, bool, i32); 18] = [
    (3, false, 0),
    (3, false, 1),
    (3, false, 2),
    (3, true, 0),
    (3, true, 1),
    (3, true, 2),
    (4, false, 0),
    (4, false, 1),
    (4, false, 2),
    (4, true, 0),
    (4, true, 1),
    (4, true, 2),
    (5, false, 0),
    (5, false, 1),
    (5, false, 2),
    (5, true, 0),
    (5, true, 1),
    (5, true, 2),
];

fn source_copy_scenarios() -> impl Iterator<Item = (i32, bool, i32)> {
    // Ordering is pinned to the native stage/flags/failure triple loop.
    COPY_SOURCE_SCENARIOS.into_iter()
}

fn copy_probe_vertex() -> Handle<CopySourceProbe> {
    let shape = super::shape_drawer_oracle_inputs::shape_drawer_vertex(
        rustycad_tkmath::gp::Point3::new(1., 2., 3.),
    );
    let input = copy_source_input(&Handle::default(), 0, 0);
    let object = CopySourceProbe::new(&shape, &input);
    object.attributes().setup_own_defaults().unwrap();
    object
}
fn copy_probe_presentation(
    object: &Handle<CopySourceProbe>,
) -> (
    Handle<PrsMgrPresentationManager>,
    Handle<PrsMgrPresentation>,
) {
    let driver = rustycad_tkservice::null_driver::NullGraphicDriver::new_handle();
    let structures = Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
    let manager = PrsMgrPresentationManager::new(&structures);
    let presentation = PrsMgrPresentation::new(&manager, &object.cast(), 0).unwrap();
    (manager, presentation)
}
#[test]
fn copy_source_application_subclass_keeps_genuine_inherited_compute_dispatch() {
    use crate::select_mgr::{SelectMgrSelectableObject, SelectMgrSelection};
    let object = copy_probe_vertex();
    let (manager, presentation) = copy_probe_presentation(&object);
    let structure: Handle<dyn Graphic3dStructureSource> = presentation.cast();
    object.compute(&manager, &structure, 0).unwrap();
    assert!(!structure.groups().is_empty());
    let selection = Handle::new(SelectMgrSelection::new(1));
    object.compute_selection(&selection, 1).unwrap();
    assert!(!selection.is_empty());
}
