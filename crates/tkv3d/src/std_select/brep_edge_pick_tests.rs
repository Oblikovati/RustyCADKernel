//! Native ray volumes test constructed edge entities, including appended tiny-circle points and retained sentinels.
use super::brep_edge_selection_tests::brep_edge_entities;
use crate::{select_basics::SelectBasicsPickResult, select_mgr::*, select3d::*};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
#[test]
fn actual_native_axis_volumes_replay_constructed_edge_matching() {
    let rows = include_str!("../../tests/fixtures/colored-shape-compute/native-edge-picks.txt");
    for row in rows.lines() {
        replay_constructed_edge_pick(row);
    }
}
fn replay_constructed_edge_pick(row: &str) {
    let fields: Vec<_> = row.split_whitespace().collect();
    let index = fields[1].parse::<usize>().unwrap();
    let ray = fields[2].parse::<usize>().unwrap();
    let scene = brep_edge_entities(index);
    let volume = constructed_edge_volume(ray, &scene.output);
    let count = assert_constructed_edge_pick(&scene.output, &volume, &fields[3..]);
    assert_appended_edge_pick(&scene.selection, &volume, &fields[3 + count..]);
}
fn constructed_edge_volume(
    ray: usize,
    entity: &Handle<dyn Select3DSensitiveEntity>,
) -> SelectMgrSelectingVolumeManager {
    let origins = [
        Point3::new(0., 0., 10.),
        Point3::new(2., 2., 10.),
        Point3::new(2., 0., 10.),
        Point3::new(9., 8., 10.),
    ];
    let mut volume = SelectMgrSelectingVolumeManager::new();
    let origin = if ray == 4 {
        constructed_edge_start(entity)
    } else {
        origins[ray]
    };
    volume.init_axis_selecting_volume(Axis1::new(origin, -Direction3::Z));
    volume.build_selecting_volume().unwrap();
    volume
}
fn constructed_edge_start(entity: &Handle<dyn Select3DSensitiveEntity>) -> Point3 {
    let start = if entity.is_kind(Select3DSensitiveSegment::static_type()) {
        entity.cast::<Select3DSensitiveSegment>().start_point()
    } else if entity.is_kind(Select3DSensitivePoly::static_type()) {
        entity.cast::<Select3DSensitivePoly>().points_3d()[0]
    } else if entity.is_kind(Select3DSensitiveCurve::static_type()) {
        entity.cast::<Select3DSensitiveCurve>().core.points_3d()[0]
    } else {
        entity.cast::<Select3DSensitivePoint>().point()
    };
    Point3::new(start.x(), start.y(), 10.)
}
fn assert_constructed_edge_pick(
    entity: &Handle<dyn Select3DSensitiveEntity>,
    volume: &SelectMgrSelectingVolumeManager,
    fields: &[&str],
) -> usize {
    let mut result = SelectBasicsPickResult::default();
    let hit = entity.matches(volume, &mut result).unwrap();
    assert_eq!(hit, fields[0] == "1");
    if !hit {
        return 1;
    }
    assert!((result.depth() - fields[1].parse::<f64>().unwrap()).abs() < 1e-12);
    assert!((result.dist_to_geom_center() - fields[2].parse::<f64>().unwrap()).abs() < 1e-12);
    3
}
fn assert_appended_edge_pick(
    selection: &SelectMgrSelection,
    volume: &SelectMgrSelectingVolumeManager,
    mut fields: &[&str],
) {
    for wrapper in selection.entities() {
        assert_eq!(fields[0], "appended");
        fields = &fields[1..];
        let count = assert_constructed_edge_pick(&wrapper.base_sensitive(), volume, fields);
        fields = &fields[count..];
    }
    assert!(fields.is_empty());
}
