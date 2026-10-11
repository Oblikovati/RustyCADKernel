//! Rust-only replay of committed native state and world/light propagation snapshots.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::{gp::*, toploc::Datum3};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
struct FakeLightPresentation {
    transient: StandardTransient,
    light: AisLightSourceBase,
    fail_refresh: std::sync::atomic::AtomicBool,
}
rustycad_tkernel::impl_transient!(
    FakeLightPresentation,
    "FakeLightPresentation",
    StandardTransient,
    transient,
    interfaces:[dyn AisLightSourceSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]
);
impl FakeLightPresentation {
    fn new(kind: Graphic3dTypeOfLightSource) -> Self {
        Self {
            transient: StandardTransient::default(),
            fail_refresh: std::sync::atomic::AtomicBool::new(false),
            light: AisLightSourceBase::new(&Handle::new(Graphic3dCLight::new(kind))),
        }
    }
}
impl AisLightSourceSource for FakeLightPresentation {
    fn light_source_base(&self) -> &AisLightSourceBase {
        &self.light
    }
    fn update_light_aspects(&self) -> Result<(), Failure> {
        if self.fail_refresh.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(rustycad_tkernel::standard::exceptions::program_error(
                "FakeLightPresentation refresh failed: expected available aspect refresh",
            ));
        }
        Ok(())
    }
}
impl AisInteractiveObject for FakeLightPresentation {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.light.interactive
    }
}
impl PrsMgrPresentableObject for FakeLightPresentation {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.light.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        _: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
impl SelectMgrSelectableObject for FakeLightPresentation {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.light.interactive.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, _: i32) -> Result<(), Failure> {
        Ok(())
    }
}
fn light_kind(kind: usize) -> Graphic3dTypeOfLightSource {
    [
        Graphic3dTypeOfLightSource::Ambient,
        Graphic3dTypeOfLightSource::Directional,
        Graphic3dTypeOfLightSource::Positional,
        Graphic3dTypeOfLightSource::Spot,
    ][kind]
}
fn configure(object: &FakeLightPresentation, kind: usize, zoom: bool, head: bool) {
    if kind != 0 {
        object
            .light()
            .set_display_position(&Point3::new(2.0, 3.0, 4.0))
            .unwrap();
        object.light().set_headlight(head).unwrap();
    }
    if kind == 1 || kind == 3 {
        object
            .light()
            .set_direction(&Direction3::new(1.0, 2.0, -3.0).unwrap())
            .unwrap();
    }
    object.set_zoomable(zoom);
}
fn assigned_transform() -> Transform3 {
    let mut transform = Transform3::default();
    transform
        .set_rotation(
            Axis1::new(Point3::default(), Direction3::new(1.0, 2.0, 3.0).unwrap()),
            0.7,
        )
        .unwrap();
    transform.set_translation_part(Vector3::new(5.0, 6.0, 7.0));
    transform
}
fn snapshot(object: &FakeLightPresentation, kind: usize) -> Vec<f64> {
    let mut values = vec![
        object.size(),
        object.arc_size() as f64,
        object.nb_splits_quadric() as f64,
        object.nb_splits_arrow() as f64,
        object.nb_arrows() as f64,
    ];
    append_persistence(object, &mut values);
    append_transform(object, &mut values);
    let pos = object.light().position();
    values.extend([pos.x(), pos.y(), pos.z()]);
    let dir = if kind == 0 || kind == 2 {
        Direction3::Z
    } else {
        object.light().direction().unwrap()
    };
    values.extend([dir.x(), dir.y(), dir.z()]);
    values
}
fn append_persistence(object: &FakeLightPresentation, values: &mut Vec<f64>) {
    let pers = object.presentable_base().read().transform_persistence;
    values.push(
        pers.get()
            .map(|pers| pers.mode().0 as f64)
            .unwrap_or_default(),
    );
    let anchor = pers
        .get()
        .filter(|pers| pers.mode() == Graphic3dTransModeFlags::ZOOM_PERS)
        .map(|pers| pers.anchor_point().unwrap())
        .unwrap_or_default();
    values.extend([anchor.x(), anchor.y(), anchor.z()]);
}
fn append_transform(object: &FakeLightPresentation, values: &mut Vec<f64>) {
    let datum = object.presentable_base().read().local_transformation;
    let trsf = datum
        .get()
        .map(|datum| *datum.transformation())
        .unwrap_or_default();
    for row in 1..=3 {
        for col in 1..=4 {
            values.push(trsf.value(row, col).unwrap());
        }
    }
}
fn native_scenario(values: &[f64]) -> FakeLightPresentation {
    let kind = values[0] as usize;
    let object = FakeLightPresentation::new(light_kind(kind));
    configure(&object, kind, values[1] != 0.0, values[2] != 0.0);
    ais_light_source_update_persistence(&object).unwrap();
    ais_light_source_update_local_transform(&object).unwrap();
    if values[3] != 0.0 {
        ais_light_source_set_local_transform(
            &object,
            &Handle::new(Datum3::new(assigned_transform())),
        )
        .unwrap();
    }
    if values[3] == 2.0 {
        ais_light_source_update_persistence(&object).unwrap();
        ais_light_source_update_local_transform(&object).unwrap();
    }
    object
}
#[test]
fn native_world_light_transform_snapshots() {
    for line in include_str!("../../tests/fixtures/light-source-native.txt").lines() {
        let values: Vec<f64> = line
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        let object = native_scenario(&values);
        let actual_values = snapshot(&object, values[0] as usize);
        assert_eq!(actual_values.len(), values.len() - 4);
        for (actual, expected) in actual_values.iter().zip(&values[4..]) {
            assert!(
                (actual - expected).abs() < 2e-12,
                "native row {line}: actual {actual}, expected {expected}"
            );
        }
    }
}
#[test]
fn live_light_replacement_retains_pointer_identity() {
    let object = FakeLightPresentation::new(light_kind(1));
    let replacement = Handle::new(Graphic3dCLight::new(light_kind(3)));
    object.set_light(&replacement);
    assert_eq!(
        object.light().object_address(),
        replacement.object_address()
    );
    replacement.set_enabled(false);
    assert!(!object.light().is_enabled());
}
#[test]
fn persistence_reuses_native_identity_and_updates_zoom_anchor() {
    let object = FakeLightPresentation::new(light_kind(2));
    object.set_zoomable(false);
    ais_light_source_update_persistence(&object).unwrap();
    let previous = object.presentable_base().read().transform_persistence;
    object
        .light()
        .set_position(&Point3::new(10.0, 20.0, 30.0))
        .unwrap();
    ais_light_source_update_persistence(&object).unwrap();
    let current = object.presentable_base().read().transform_persistence;
    assert_eq!(previous.object_address(), current.object_address());
    assert_eq!(current.anchor_point().unwrap().x(), 10.0);
}
#[test]
fn zoomable_persistence_clears_and_null_datum_resets_position() {
    let object = FakeLightPresentation::new(light_kind(2));
    object.set_zoomable(true);
    ais_light_source_update_persistence(&object).unwrap();
    assert!(
        object
            .presentable_base()
            .read()
            .transform_persistence
            .is_null()
    );
    ais_light_source_set_local_transform(&object, &Handle::default()).unwrap();
    assert_eq!(object.light().position().x(), 0.0);
}

#[test]
fn custom_marker_identity_survives_standard_marker_override() {
    let object = FakeLightPresentation::new(light_kind(1));
    let image = Handle::new(Graphic3dMarkerImage::from_bitmap(Handle::default(), 8, 8));
    for enabled in [false, true] {
        object.set_marker_image(&image, enabled);
        assert_eq!(object.marker_type(enabled), AspectTypeOfMarker::Userdefined);
        object.set_marker_type(AspectTypeOfMarker::Star, enabled);
        assert_eq!(
            object.marker_image(enabled).object_address(),
            image.object_address()
        );
    }
}
#[test]
fn ambient_keeps_existing_2d_persistence_identity() {
    let object = FakeLightPresentation::new(light_kind(0));
    let pers =
        Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags::TMF_2D).unwrap()).cast();
    object.set_transform_persistence(&pers);
    ais_light_source_update_persistence(&object).unwrap();
    assert_eq!(
        object
            .presentable_base()
            .read()
            .transform_persistence
            .object_address(),
        pers.object_address()
    );
}
#[test]
fn property_mutations_match_committed_native_rows() {
    for line in include_str!("../../tests/fixtures/light-source-properties-native.txt").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let object = FakeLightPresentation::new(light_kind(expected[0] as usize));
        mutate_properties(&object);
        let actual = property_values(&object);
        assert_eq!(actual.len(), expected.len() - 1);
        for (actual, expected) in actual.iter().zip(&expected[1..]) {
            assert!(actual == expected || (actual.is_nan() && expected.is_nan()));
        }
    }
}
fn mutate_properties(object: &FakeLightPresentation) {
    object.set_display_name(false);
    object.set_display_range(false);
    object.set_switch_on_click(false);
    object.set_size(f64::NAN);
    object.set_arc_size(-2);
    object.set_nb_arrows(7);
    object.set_nb_splits_quadric(-4);
    object.set_nb_splits_arrow(0);
    object.set_marker_image(&Handle::default(), true);
    object.set_marker_image(&Handle::default(), false);
}
fn property_values(object: &FakeLightPresentation) -> Vec<f64> {
    vec![
        f64::from(object.to_display_name()),
        f64::from(object.to_display_range()),
        f64::from(object.to_switch_on_click()),
        object.size(),
        object.arc_size() as f64,
        object.nb_arrows() as f64,
        object.nb_splits_quadric() as f64,
        object.nb_splits_arrow() as f64,
        object.marker_type(true) as i32 as f64,
        object.marker_type(false) as i32 as f64,
    ]
}
fn presentation_for(object: &Handle<FakeLightPresentation>) -> Handle<PrsMgrPresentation> {
    let driver = rustycad_tkservice::null_driver::NullGraphicDriver::new_handle();
    let structures = Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
    let manager = PrsMgrPresentationManager::new(&structures);
    let presentation = PrsMgrPresentation::new(&manager, &object.cast(), 0).unwrap();
    object
        .presentable_base()
        .fields
        .write()
        .presentations
        .push(presentation.clone());
    presentation
}
#[test]
fn setters_preserve_native_dirty_status_rules() {
    let object = Handle::new(FakeLightPresentation::new(light_kind(1)));
    let presentation = presentation_for(&object);
    object.set_display_name(true);
    object.set_display_range(true);
    object.set_size(50.0);
    object.set_arc_size(25);
    object.set_zoomable(false);
    object.set_nb_arrows(5);
    assert!(!presentation.must_be_updated());
    mutate_without_invalidation(&object);
    assert!(!presentation.must_be_updated());
    object.set_size(60.0);
    assert!(presentation.must_be_updated());
    presentation.set_update_status(false);
    object.set_light(&object.light());
    assert!(presentation.must_be_updated());
}

fn mutate_without_invalidation(object: &FakeLightPresentation) {
    object.set_draggable(false);
    object.set_switch_on_click(false);
    object.set_nb_splits_quadric(40);
    object.set_nb_splits_arrow(10);
    object.set_marker_type(AspectTypeOfMarker::X, true);
    object.set_marker_image(&Handle::default(), false);
}
#[test]
fn every_invalidating_setter_marks_presentations_dirty() {
    for case in 0..6 {
        let object = Handle::new(FakeLightPresentation::new(light_kind(1)));
        let presentation = presentation_for(&object);
        change_invalidating_property(&object, case);
        assert!(presentation.must_be_updated(), "mutation case {case}");
    }
}
fn change_invalidating_property(object: &FakeLightPresentation, case: usize) {
    match case {
        0 => object.set_display_name(false),
        1 => object.set_display_range(false),
        2 => object.set_size(f64::NAN),
        3 => object.set_arc_size(-10),
        4 => object.set_zoomable(true),
        5 => object.set_nb_arrows(2),
        _ => unreachable!(),
    }
}

#[test]
fn aspect_error_preserves_already_assigned_light_and_local_transform() {
    let object = FakeLightPresentation::new(light_kind(2));
    object.set_zoomable(false);
    ais_light_source_update_persistence(&object).unwrap();
    object
        .fail_refresh
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let result = ais_light_source_set_local_transform(
        &object,
        &Handle::new(Datum3::new(assigned_transform())),
    );
    assert!(result.is_err());
    assert_eq!(object.light().position().x(), 5.0);
    assert_eq!(object.local_transformation().value(1, 4).unwrap(), 5.0);
    let persistence = object.presentable_base().read().transform_persistence;
    assert_eq!(persistence.anchor_point().unwrap().x(), 0.0);
}

#[test]
fn application_subclasses_expose_the_native_light_interface() {
    let object = Handle::new(FakeLightPresentation::new(light_kind(1)));
    let interface: Handle<dyn AisLightSourceSource> = object.cast();
    assert!(!interface.is_null());
    assert_eq!(
        interface.light().object_address(),
        object.light().object_address()
    );
}
