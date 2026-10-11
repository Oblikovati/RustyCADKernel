//! Native light sensitive entities and ownership clicks use committed oracle assets.
use rustycad_tkernel::{ncollection::Vec2, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, select3d::*};
fn source(kind: usize) -> Handle<AisLightSource> {
    AisLightSource::new(&fixture_source::fixture_light(kind)).unwrap()
}
#[test]
fn native_light_sensitive_entity_snapshots() {
    for line in include_str!("fixtures/light-selection-native.txt").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = selection_snapshot(&expected);
        assert_eq!(actual, &expected[4..], "{line}");
    }
}
fn selection_snapshot(expected: &[f64]) -> Vec<f64> {
    let object = source(expected[0] as usize);
    object.set_zoomable(expected[1] != 0.0);
    object.set_size(expected[3]);
    ais_light_source_update_persistence(&*object).unwrap();
    let selection = Handle::new(SelectMgrSelection::new(expected[2] as i32));
    object
        .compute_selection(&selection, expected[2] as i32)
        .unwrap();
    let mut actual = vec![selection.entities().len() as f64];
    for entity in selection.entities() {
        append_sensitive(&entity.base_sensitive(), &mut actual);
    }
    actual
}
fn append_sensitive(entity: &Handle<dyn Select3DSensitiveEntity>, values: &mut Vec<f64>) {
    let sphere = entity.cast::<Select3DSensitiveSphere>();
    let center = entity.center_of_geometry();
    values.extend([
        f64::from(!sphere.is_null()),
        f64::from(entity.sensitivity_factor()),
        f64::from(entity.owner_id().priority()),
        center.x(),
        center.y(),
        center.z(),
        sphere
            .get()
            .map(|sphere| sphere.radius())
            .unwrap_or_default(),
    ]);
}
#[test]
fn native_light_owner_click_snapshots() {
    for line in include_str!("fixtures/light-owner-clicks-native.txt").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = click_snapshot(&expected);
        assert_eq!(actual.len(), expected.len() - 5);
        for (actual, expected) in actual.iter().zip(&expected[5..]) {
            assert!((actual - expected).abs() < 2e-12, "{line}");
        }
    }
}
fn click_snapshot(expected: &[f64]) -> Vec<f64> {
    let object = source(expected[0] as usize);
    object.set_switch_on_click(expected[1] != 0.0);
    let owner = AisLightSourceOwner::new(&object.cast(), 7);
    let result = owner.handle_mouse_click(
        Vec2::new(11, 12),
        expected[2] as u32,
        expected[3] as u32,
        expected[4] != 0.0,
    );
    let mut actual = vec![
        f64::from(result),
        f64::from(object.light().is_enabled()),
        f64::from(owner.is_forced_hilight()),
        f64::from(owner.priority()),
    ];
    append_click_aspects(&object, &mut actual);
    actual
}
#[test]
fn orphan_owner_declines_clicks_and_forced_highlight() {
    let owner = AisLightSourceOwner::new(&Handle::default(), 5);
    assert!(!owner.is_forced_hilight());
    assert!(!owner.handle_mouse_click(Vec2::new(0, 0), 8192, 0, false));
    assert!(!owner.handle_light_click(8192, 0).unwrap());
    let object = source(0);
    let owner = AisLightSourceOwner::new(&object.cast(), 15);
    drop(object);
    assert!(owner.selectable().is_null());
    assert!(!owner.is_forced_hilight());
}

#[path = "support/light_fixture_source.rs"]
mod fixture_source;

fn append_click_aspects(object: &AisLightSource, actual: &mut Vec<f64>) {
    let point = object.attributes().point_aspect().aspect();
    actual.extend(point.interior_color().rgb().map(f64::from));
    actual.extend([
        point.marker_type() as i32 as f64,
        object.attributes().arrow_aspect().aspect().marker_type() as i32 as f64,
    ]);
}
