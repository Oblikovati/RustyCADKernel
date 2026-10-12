//! Actual selecting volumes replay committed native primitive detection snapshots.
use super::{Select3DPrimArrayInit, Select3DSensitiveEntity, Select3DSensitivePrimitiveArray};
use crate::select_basics::SelectBasicsPickResult;
use crate::select_mgr::SelectMgrSelectingVolumeManager;
use rustycad_tkernel::ncollection::{Array1, Vec2, Vec3, Vec4};
use rustycad_tkernel::standard::{Handle, real::real_last};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkmath::gp::{Axis1, Direction3, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::{
    Graphic3dAttribute, Graphic3dBuffer, Graphic3dBufferBase, Graphic3dBufferMembers,
    Graphic3dBufferSource, Graphic3dCamera, Graphic3dIndexBuffer, Graphic3dIndexBufferMembers,
    Graphic3dIndexBufferSource, Graphic3dTypeOfAttribute, Graphic3dTypeOfData,
};

#[derive(Clone, Copy)]
pub(super) struct MatchScenario {
    ordinal: i32,
    dimension: i32,
    volume: i32,
    maps: bool,
    detection: i32,
    patch: i32,
    groups: i32,
    indexed: bool,
    triangles: bool,
}
impl MatchScenario {
    pub(super) fn new(ordinal: i32) -> Self {
        Self {
            ordinal,
            dimension: ordinal / 1024 + 2,
            volume: ordinal % 4,
            maps: ordinal / 4 % 2 != 0,
            detection: ordinal / 8 % 8,
            patch: if ordinal / 64 % 2 == 0 { 1 } else { 4 },
            groups: if ordinal / 128 % 2 == 0 { 1 } else { 3 },
            indexed: ordinal / 256 % 2 != 0,
            triangles: ordinal / 512 % 2 != 0,
        }
    }
    fn primitive_count(self) -> i32 {
        if !self.triangles {
            100
        } else if self.indexed {
            36
        } else {
            108
        }
    }
}
fn matching_geometry(scenario: MatchScenario) -> Handle<dyn Graphic3dBufferSource> {
    let vertices =
        Handle::new(Graphic3dBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let position = Graphic3dAttribute {
        id: Graphic3dTypeOfAttribute::Pos,
        data_type: matching_position_type(scenario.dimension),
    };
    vertices
        .init(scenario.primitive_count(), &[position])
        .unwrap();
    for index in 0..scenario.primitive_count() {
        write_matching_vertex(&vertices, scenario, index);
    }
    vertices.cast()
}
fn matching_position_type(dimension: i32) -> Graphic3dTypeOfData {
    match dimension {
        2 => Graphic3dTypeOfData::Vec2,
        3 => Graphic3dTypeOfData::Vec3,
        _ => Graphic3dTypeOfData::Vec4,
    }
}
fn write_matching_vertex(vertices: &Graphic3dBuffer, scenario: MatchScenario, index: i32) {
    let point = matching_vertex(scenario, index);
    let offset = vertices.element_offset(index);
    match scenario.dimension {
        2 => {
            vertices.write_at(offset, Vec2::new(point.x, point.y));
        }
        3 => {
            vertices.write_at(offset, point);
        }
        _ => {
            vertices.write_at(offset, Vec4::new(point.x, point.y, point.z, 2.0));
        }
    }
}
fn matching_vertex(scenario: MatchScenario, index: i32) -> Vec3<f32> {
    let primitive = if scenario.triangles && !scenario.indexed {
        index / 3
    } else {
        index
    };
    let (x, y) = matching_grid_origin(scenario, primitive);
    let (dx, dy) = matching_triangle_offset(scenario, index);
    Vec3::new(
        (f64::from(x) + dx) as f32,
        (f64::from(y) + dy) as f32,
        -(primitive % 4) as f32 * 0.25,
    )
}
fn matching_grid_origin(scenario: MatchScenario, primitive: i32) -> (i32, i32) {
    let x = if scenario.triangles {
        primitive % 6 - 3
    } else {
        primitive % 10 - 4
    };
    let y = if scenario.triangles {
        primitive / 6 - 3
    } else {
        primitive / 10 - 4
    };
    (x, y)
}
fn matching_triangle_offset(scenario: MatchScenario, index: i32) -> (f64, f64) {
    if !scenario.triangles || scenario.indexed {
        return (0.0, 0.0);
    }
    (
        if index % 3 == 0 { -0.4 } else { 0.4 },
        if index % 3 == 2 { 0.4 } else { -0.4 },
    )
}
fn matching_indices(scenario: MatchScenario) -> Handle<dyn Graphic3dIndexBufferSource> {
    if !scenario.indexed {
        return Handle::default();
    }
    let indices =
        Handle::new(Graphic3dIndexBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let length = if scenario.triangles {
        150
    } else {
        scenario.primitive_count()
    };
    indices.init_int32(length).unwrap();
    for index in 0..length {
        indices.set_index(index, matching_index(scenario, length - index - 1));
    }
    indices.cast()
}
fn matching_index(scenario: MatchScenario, reverse: i32) -> i32 {
    if !scenario.triangles {
        return reverse;
    }
    let cell = reverse / 6;
    (cell / 5) * 6 + cell % 5 + [0, 1, 6, 1, 7, 6][(reverse % 6) as usize]
}
fn matching_probe(scenario: MatchScenario) -> Select3DSensitivePrimitiveArray {
    matching_probe_owned(scenario, &Handle::default(), 2)
}
pub(super) fn matching_probe_owned(
    scenario: MatchScenario,
    owner: &Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>,
    sensitivity: i32,
) -> Select3DSensitivePrimitiveArray {
    let probe = Select3DSensitivePrimitiveArray::new(owner).unwrap();
    probe.entity.fields.write().sensitivity_factor = sensitivity;
    probe.set_patch_size_max(scenario.patch);
    probe.set_patch_distance(1.5);
    probe.set_detect_elements(scenario.detection & 1 != 0);
    probe.set_detect_nodes(scenario.detection & 2 != 0);
    probe.set_detect_edges(scenario.detection & 4 != 0);
    probe.set_detect_element_map(scenario.maps);
    probe.set_detect_node_map(scenario.maps);
    initialize_matching_probe(&probe, scenario);
    probe
}
fn initialize_matching_probe(probe: &Select3DSensitivePrimitiveArray, scenario: MatchScenario) {
    let (vertices, indices) = (matching_geometry(scenario), matching_indices(scenario));
    let init = Select3DPrimArrayInit::whole(
        &vertices,
        &indices,
        &Location::default(),
        true,
        scenario.groups,
    );
    let ok = if scenario.triangles {
        probe.init_triangulation(&init)
    } else {
        probe.init_points(&init)
    };
    assert!(ok.unwrap());
}
pub(crate) fn matching_volume(kind: i32, miss: bool) -> SelectMgrSelectingVolumeManager {
    let mut volume = SelectMgrSelectingVolumeManager::new();
    let shift = if miss { 300.0 } else { 0.0 };
    initialize_matching_volume(&mut volume, kind, shift);
    initialize_full_matching_volume(&mut volume, kind, shift);
    volume.allow_overlap_detection(kind != 2 && kind != 5);
    if kind != 0 {
        configure_volume_camera(&volume);
    }
    volume.build_selecting_volume().unwrap();
    volume
}
fn initialize_matching_volume(volume: &mut SelectMgrSelectingVolumeManager, kind: i32, shift: f64) {
    if kind == 0 {
        volume.init_axis_selecting_volume(matching_axis(shift != 0.0));
    }
    if kind == 1 || kind == 2 {
        volume.init_box_selecting_volume(
            Point2::new(85.0 + shift, 85.0),
            Point2::new(115.0 + shift, 115.0),
        );
    }
    if kind == 3 {
        volume
            .init_polyline_selecting_volume(&polygon_pixels(shift))
            .unwrap();
    }
}
fn initialize_full_matching_volume(
    volume: &mut SelectMgrSelectingVolumeManager,
    kind: i32,
    shift: f64,
) {
    if kind == 4 || kind == 5 {
        volume.init_box_selecting_volume(
            Point2::new(20.0 + shift, 20.0),
            Point2::new(180.0 + shift, 180.0),
        );
    }
    if kind == 6 {
        volume.init_point_selecting_volume(Point2::new(100.0 + shift, 100.0));
    }
}
fn matching_axis(miss: bool) -> Axis1 {
    Axis1::new(
        Point3::new(if miss { 30.0 } else { 0.0 }, 0.0, 10.0),
        Direction3::new(0.0, 0.0, -1.0).unwrap(),
    )
}
fn polygon_pixels(shift: f64) -> Array1<'static, Point2> {
    let mut polygon = Array1::with_bounds(1, 4).unwrap();
    polygon.as_mut_slice().copy_from_slice(&[
        Point2::new(85.0 + shift, 85.0),
        Point2::new(115.0 + shift, 85.0),
        Point2::new(115.0 + shift, 115.0),
        Point2::new(85.0 + shift, 115.0),
    ]);
    polygon
}
fn configure_volume_camera(volume: &SelectMgrSelectingVolumeManager) {
    let camera = Handle::new(Graphic3dCamera::default());
    camera
        .set_eye_and_center(&Point3::new(0.0, 0.0, 10.0), &Point3::default())
        .unwrap();
    camera.set_up(&Direction3::new(0.0, 1.0, 0.0).unwrap());
    camera.set_scale(20.0);
    camera.set_aspect(1.0);
    camera.set_z_range(0.1, 30.0).unwrap();
    volume.set_camera(&camera).unwrap();
    volume.set_window_size(200, 200).unwrap();
}
fn matching_snapshot(
    probe: &Select3DSensitivePrimitiveArray,
    scenario: i32,
    step: i32,
    hit: bool,
    pick: &SelectBasicsPickResult,
) -> Vec<f64> {
    let mut snapshot = pick_snapshot(scenario, step, hit, pick);
    snapshot.extend([
        f64::from(probe.last_detected_element()),
        f64::from(probe.last_detected_node()),
        f64::from(probe.last_detected_edge_node1()),
        f64::from(probe.last_detected_edge_node2()),
        f64::from(probe.set.detected_idx()),
        f64::from(probe.to_build_bvh()),
    ]);
    append_detected_map(&mut snapshot, &probe.last_detected_element_map());
    append_detected_map(&mut snapshot, &probe.last_detected_node_map());
    snapshot
}
fn pick_snapshot(scenario: i32, step: i32, hit: bool, pick: &SelectBasicsPickResult) -> Vec<f64> {
    let mut snapshot = vec![
        f64::from(scenario),
        f64::from(step),
        f64::from(hit),
        f64::from(pick.is_valid()),
        if pick.is_valid() { pick.depth() } else { 0.0 },
        if pick.dist_to_geom_center() == real_last() {
            0.0
        } else {
            pick.dist_to_geom_center()
        },
        f64::from(pick.has_picked_point()),
    ];
    append_picked_point(&mut snapshot, pick);
    snapshot
}
fn append_picked_point(snapshot: &mut Vec<f64>, pick: &SelectBasicsPickResult) {
    if pick.has_picked_point() {
        snapshot.extend([
            pick.picked_point().x(),
            pick.picked_point().y(),
            pick.picked_point().z(),
        ]);
    } else {
        snapshot.extend([0.0; 3]);
    }
}
fn append_detected_map(snapshot: &mut Vec<f64>, map: &Handle<HPackedMapOfInteger>) {
    let mut indices = map.get().map_or_else(Vec::new, |value| {
        (&*value.map()).into_iter().collect::<Vec<i32>>()
    });
    indices.sort_unstable();
    snapshot.extend([f64::from(!map.is_null()), indices.len() as f64]);
    snapshot.extend(indices.into_iter().map(f64::from));
}
pub(super) fn compare_matching_snapshot(actual: &[f64], expected: &str) {
    let native: Vec<f64> = expected
        .split_whitespace()
        .map(|token| token.parse().unwrap())
        .collect();
    assert_eq!(
        actual.len(),
        native.len(),
        "scenario {} step {}",
        actual[0],
        actual[1]
    );
    compare_matching_values(actual, &native);
}
fn compare_matching_values(actual: &[f64], native: &[f64]) {
    for (field, (&value, &oracle)) in actual.iter().zip(native).enumerate() {
        assert!(
            value == oracle || (value - oracle).abs() <= 1e-8,
            "scenario {} step {} field {field}: Rust {value}, native {oracle}",
            actual[0],
            actual[1]
        );
    }
}
#[test]
fn native_primitive_matching_detection_flags_maps_and_repeated_volumes() {
    let mut native = super::primitive_array_matching_fixtures::MATCHING_NATIVE_SNAPSHOTS.lines();
    for ordinal in 0..3072 {
        let scenario = MatchScenario::new(ordinal);
        let probe = matching_probe(scenario);
        for step in 0..3 {
            let volume = matching_volume(scenario.volume, step == 1);
            let mut pick = SelectBasicsPickResult::default();
            let hit = probe.matches(&volume, &mut pick).unwrap();
            compare_matching_snapshot(
                &matching_snapshot(&probe, scenario.ordinal, step, hit, &pick),
                native.next().unwrap(),
            );
        }
    }
    replay_full_volume_controls(&mut native);
    assert!(native.next().is_none());
}

fn replay_full_volume_controls(native: &mut std::str::Lines<'_>) {
    for extra in 0..48 {
        let scenario = full_volume_configuration(extra);
        let probe = matching_probe(scenario);
        for step in 0..3 {
            let volume = matching_volume(scenario.volume, step == 1);
            let mut pick = SelectBasicsPickResult::default();
            let hit = probe.matches(&volume, &mut pick).unwrap();
            compare_matching_snapshot(
                &matching_snapshot(&probe, scenario.ordinal, step, hit, &pick),
                native.next().unwrap(),
            );
        }
    }
}

fn full_volume_configuration(extra: i32) -> MatchScenario {
    let config = 1024
        + (extra / 3 / 8) * 512
        + (extra / 3 / 4 % 2) * 256
        + (extra / 3 / 2 % 2) * 128
        + (extra / 3 % 2) * 64
        + 56
        + 4;
    let mut scenario = MatchScenario::new(config);
    scenario.volume = 4 + extra % 3;
    scenario.ordinal = 3072 + extra;
    scenario
}
