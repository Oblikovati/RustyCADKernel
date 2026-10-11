//! Rust-only replay of committed native initialization snapshots.
use super::{
    Select3DBndBox3d, Select3DPrimArrayInit, Select3DSensitiveEntity,
    Select3DSensitivePrimitiveArray, Select3DSensitiveSet,
};
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::{
    Graphic3dAttribute, Graphic3dBuffer, Graphic3dBufferBase, Graphic3dBufferMembers,
    Graphic3dBufferSource, Graphic3dIndexBuffer, Graphic3dIndexBufferMembers,
    Graphic3dIndexBufferSource, Graphic3dTypeOfAttribute, Graphic3dTypeOfData,
};

fn vertex_fixture(dimension: i32) -> Handle<dyn Graphic3dBufferSource> {
    let buffer =
        Handle::new(Graphic3dBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let data_type = match dimension {
        2 => Graphic3dTypeOfData::Vec2,
        3 => Graphic3dTypeOfData::Vec3,
        _ => Graphic3dTypeOfData::Vec4,
    };
    buffer
        .init(
            99,
            &[Graphic3dAttribute {
                id: Graphic3dTypeOfAttribute::Pos,
                data_type,
            }],
        )
        .unwrap();
    fill_vertices(&buffer, dimension);
    buffer.cast()
}
fn fill_vertices(buffer: &Graphic3dBuffer, dimension: i32) {
    for index in 0..99 {
        let x = (index % 11) as f32 * 0.25;
        let y = (index / 11) as f32 * 0.5;
        let z = (index % 3) as f32 * 0.125;
        let offset = buffer.element_offset(index);
        match dimension {
            2 => {
                buffer.write_at(offset, Vec2::new(x, y));
            }
            3 => {
                buffer.write_at(offset, Vec3::new(x, y, z));
            }
            _ => {
                buffer.write_at(offset, Vec4::new(x, y, z, 2.0));
            }
        }
    }
}
fn index_fixture(indexed: i32) -> Handle<dyn Graphic3dIndexBufferSource> {
    if indexed == 0 {
        return Handle::default();
    }
    let buffer =
        Handle::new(Graphic3dIndexBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    if indexed == 1 {
        buffer.init_indices::<u16>(99).unwrap();
    } else {
        buffer.init_int32(99).unwrap();
    }
    for index in 0..99 {
        buffer.set_index(index, 98 - index);
    }
    buffer.cast()
}
fn append_bounds(snapshot: &mut Vec<f64>, bounds: Select3DBndBox3d) {
    snapshot.push(f64::from(bounds.is_valid()));
    if !bounds.is_valid() {
        snapshot.extend([0.0; 6]);
        return;
    }
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    snapshot.extend([min.x, min.y, min.z, max.x, max.y, max.z]);
}
fn initialization_snapshot(
    probe: &Select3DSensitivePrimitiveArray,
    scenario: i32,
    step: i32,
    ok: bool,
) -> Vec<f64> {
    let mut snapshot = initialization_identity(probe, scenario, step, ok);
    append_bounds(&mut snapshot, probe.bounding_box());
    snapshot.push(f64::from(probe.size()));
    append_element_boxes(&mut snapshot, probe);
    snapshot.push(position_stride_snapshot(probe));
    snapshot
}
fn initialization_identity(
    probe: &Select3DSensitivePrimitiveArray,
    scenario: i32,
    step: i32,
    ok: bool,
) -> Vec<f64> {
    let center = probe.center_of_geometry();
    vec![
        f64::from(scenario),
        f64::from(step),
        f64::from(ok),
        f64::from(probe.nb_sub_elements()),
        center.x(),
        center.y(),
        center.z(),
    ]
}
fn position_stride_snapshot(probe: &Select3DSensitivePrimitiveArray) -> f64 {
    let stride = probe.read().pos_stride;
    if stride == usize::MAX {
        return -1.0;
    }
    stride as f64
}
fn append_element_boxes(snapshot: &mut Vec<f64>, probe: &Select3DSensitivePrimitiveArray) {
    for index in 0..probe.size() {
        let bounds = probe.element_box(index);
        append_bounds(snapshot, bounds);
        for axis in 0..3 {
            snapshot.push(if bounds.is_valid() {
                probe.center(index, axis).unwrap()
            } else {
                0.0
            });
        }
    }
}
fn compare_snapshot(actual: &[f64], expected: &str) {
    let wanted: Vec<f64> = expected
        .split_whitespace()
        .map(|token| token.parse().unwrap())
        .collect();
    assert_eq!(
        actual.len(),
        wanted.len(),
        "snapshot scenario {} step {}",
        actual[0],
        actual[1]
    );
    compare_native_values(actual, &wanted);
}
fn compare_native_values(actual: &[f64], wanted: &[f64]) {
    for (index, (&value, &native)) in actual.iter().zip(wanted).enumerate() {
        assert!(
            (value - native).abs() <= 1e-12,
            "scenario {} step {} field {index}: Rust {value}, native {native}",
            actual[0],
            actual[1]
        );
    }
}
#[derive(Clone, Copy)]
struct InitScenario {
    dimension: i32,
    indexed: i32,
    triangles: bool,
    patch: i32,
    groups: i32,
    range: i32,
    ordinal: i32,
}
impl InitScenario {
    fn from_ordinal(ordinal: i32) -> Self {
        Self {
            ordinal,
            range: ordinal % 3,
            groups: if ordinal / 3 % 2 == 0 { 1 } else { 3 },
            patch: [1, 2, 5][(ordinal / 6 % 3) as usize],
            triangles: ordinal / 18 % 2 != 0,
            indexed: ordinal / 36 % 3,
            dimension: ordinal / 108 + 2,
        }
    }
    fn initialization(self) -> Select3DPrimArrayInit {
        Select3DPrimArrayInit {
            verts: vertex_fixture(self.dimension),
            indices: index_fixture(self.indexed),
            init_loc: fixture_location(self.range),
            index_lower: if self.range == 0 { 0 } else { 6 },
            index_upper: if self.range == 2 { 100 } else { 98 },
            to_eval_min_max: self.ordinal % 2 == 0,
            nb_groups: self.groups,
        }
    }
}
fn ranged_initialization(probe: &Select3DSensitivePrimitiveArray, scenario: InitScenario) -> bool {
    let init = scenario.initialization();
    if scenario.range != 0 {
        return if scenario.triangles {
            probe.init_triangulation(&init).unwrap()
        } else {
            probe.init_points(&init).unwrap()
        };
    }
    whole_initialization(probe, &init, scenario)
}
fn whole_initialization(
    probe: &Select3DSensitivePrimitiveArray,
    init: &Select3DPrimArrayInit,
    scenario: InitScenario,
) -> bool {
    let (verts, indices, loc, evaluate, groups) = (
        &init.verts,
        &init.indices,
        &init.init_loc,
        init.to_eval_min_max,
        scenario.groups,
    );
    if scenario.triangles {
        return probe
            .init_triangulation_all(verts, indices, loc, evaluate, groups)
            .unwrap();
    }
    point_whole_initialization(probe, init, scenario)
}
fn point_whole_initialization(
    probe: &Select3DSensitivePrimitiveArray,
    init: &Select3DPrimArrayInit,
    scenario: InitScenario,
) -> bool {
    let (verts, indices, loc) = (&init.verts, &init.indices, &init.init_loc);
    let (evaluate, groups) = (init.to_eval_min_max, scenario.groups);
    if scenario.indexed == 0 {
        return probe
            .init_points_vertices(verts, loc, evaluate, groups)
            .unwrap();
    }
    probe
        .init_points_all(verts, indices, loc, evaluate, groups)
        .unwrap()
}

#[test]
fn native_initialization_ranges_dimensions_patches_and_groups() {
    let mut oracle =
        include_str!("../../tests/fixtures/sensitive-primitive-init-native.txt").lines();
    for ordinal in 0..324 {
        let scenario = InitScenario::from_ordinal(ordinal);
        let probe = Select3DSensitivePrimitiveArray::new(&Handle::default()).unwrap();
        probe.set_patch_size_max(scenario.patch);
        probe.set_patch_distance(0.6);
        assert_eq!(probe.patch_size_max(), scenario.patch);
        assert_eq!(probe.patch_distance(), 0.6);
        let ok = ranged_initialization(&probe, scenario);
        compare_snapshot(
            &initialization_snapshot(&probe, ordinal, 0, ok),
            oracle.next().unwrap(),
        );
    }
}
#[test]
fn native_grouped_point_reinitialization_preserves_previous_center() {
    let mut oracle = include_str!("../../tests/fixtures/sensitive-primitive-init-native.txt")
        .lines()
        .skip(324);
    let probe = Select3DSensitivePrimitiveArray::new(&Handle::default()).unwrap();
    let scenario = InitScenario {
        dimension: 3,
        groups: 3,
        ordinal: 324,
        ..InitScenario::from_ordinal(0)
    };
    for step in 0..3 {
        let ok = ranged_initialization(&probe, scenario);
        compare_snapshot(
            &initialization_snapshot(&probe, 324, step, ok),
            oracle.next().unwrap(),
        );
    }
}
fn fixture_location(range: i32) -> Location {
    let mut transform = Transform3::default();
    if range == 1 {
        transform.set_translation(Vector3::new(1.25, -2.5, 0.75));
    }
    Location::new(transform)
}

fn rejected_initialization(variant: i32) -> Select3DPrimArrayInit {
    let mut init = InitScenario::from_ordinal(108).initialization();
    init.init_loc = Location::default();
    reject_vertex_buffer(&mut init, variant);
    if variant == 4 {
        init.indices = index_fixture(2);
        init.indices.init_int32(0).unwrap();
    }
    if variant == 5 {
        init.index_lower = 6;
        init.index_upper = 5;
    }
    init
}
fn reject_vertex_buffer(init: &mut Select3DPrimArrayInit, variant: i32) {
    if variant == 0 {
        init.verts = Handle::default();
    }
    if variant == 1 {
        init.verts.set_nb_elements(0);
    }
    if variant == 2 {
        init.verts.buffer_base().state.write().attributes[0].id = Graphic3dTypeOfAttribute::Norm;
    }
    if variant == 3 {
        reject_position_type(init);
    }
}
fn reject_position_type(init: &mut Select3DPrimArrayInit) {
    init.verts
        .init(
            99,
            &[Graphic3dAttribute {
                id: Graphic3dTypeOfAttribute::Pos,
                data_type: Graphic3dTypeOfData::Float,
            }],
        )
        .unwrap();
}
#[test]
fn native_failed_initialization_preserves_the_observable_partial_state() {
    let mut oracle = include_str!("../../tests/fixtures/sensitive-primitive-init-native.txt")
        .lines()
        .skip(327);
    for ordinal in 325..337 {
        let probe = Select3DSensitivePrimitiveArray::new(&Handle::default()).unwrap();
        let initial = InitScenario::from_ordinal(108).initialization();
        assert!(probe.init_points(&initial).unwrap());
        let rejected = rejected_initialization((ordinal - 325) % 6);
        let ok = apply_rejected_initializer(&probe, &rejected, ordinal < 331);
        compare_snapshot(
            &initialization_snapshot(&probe, ordinal, 0, ok),
            oracle.next().unwrap(),
        );
    }
    assert!(oracle.next().is_none());
}

fn apply_rejected_initializer(
    probe: &Select3DSensitivePrimitiveArray,
    init: &Select3DPrimArrayInit,
    points: bool,
) -> bool {
    if points {
        return probe.init_points(init).unwrap();
    }
    probe.init_triangulation(init).unwrap()
}
