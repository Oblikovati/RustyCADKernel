//! Native snapshot columns remain explicit so every selection field is compared.
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers, Select3DSensitiveSet,
    Select3DSensitiveTriangulation,
};
use crate::select_basics::SelectBasicsPickResult;
use crate::select_mgr::SelectMgrEntityOwnerMembers;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

pub(super) fn oracle_snapshot(
    probe: &Select3DSensitiveTriangulation,
    ordinal: i32,
    step: i32,
) -> Vec<f64> {
    let volume = super::primitive_array_matching_tests::matching_volume(
        [0, 1, 2, 3, 4, 6][ordinal as usize % 6],
        step == 1,
    );
    let mut pick = SelectBasicsPickResult::default();
    let outcome = probe.matches(&volume, &mut pick);
    let mut columns = outcome_columns(ordinal, step, outcome);
    append_pick_columns(&mut columns, &pick);
    append_geometry_columns(&mut columns, probe);
    append_triangle_columns(&mut columns, probe);
    columns
}
pub(super) fn failure_code(failure: Option<&Failure>) -> f64 {
    match failure {
        None => 0.0,
        Some(failure) if failure.exception_type() == "Standard_OutOfRange" => 1.0,
        Some(_) => 2.0,
    }
}
fn append_point_columns(columns: &mut Vec<f64>, point: Point3) {
    let coordinates = [point.x(), point.y(), point.z()];
    columns.extend(coordinates);
}
fn append_pick_columns(columns: &mut Vec<f64>, pick: &SelectBasicsPickResult) {
    columns.extend([
        f64::from(pick.is_valid()),
        if pick.is_valid() { pick.depth() } else { 0.0 },
        if pick.dist_to_geom_center() == f64::MAX {
            0.0
        } else {
            pick.dist_to_geom_center()
        },
        f64::from(pick.has_picked_point()),
    ]);
    append_point_columns(
        columns,
        if pick.has_picked_point() {
            *pick.picked_point()
        } else {
            Point3::default()
        },
    );
}
fn append_geometry_columns(columns: &mut Vec<f64>, probe: &Select3DSensitiveTriangulation) {
    columns.extend([f64::from(probe.size()), f64::from(probe.nb_sub_elements())]);
    append_point_columns(columns, probe.center_of_geometry());
    columns.push(f64::from(probe.has_init_location()));
    let location = probe.get_init_location().transformation();
    append_transform_columns(columns, |row, col| location.value(row, col).unwrap());
    append_transform_columns(columns, |row, col| {
        probe.inv_init_location().value(row, col).unwrap()
    });
    append_bounds_columns(columns, probe);
}
fn append_bounds_columns(columns: &mut Vec<f64>, probe: &Select3DSensitiveTriangulation) {
    let bounds = probe.bounding_box();
    columns.push(f64::from(bounds.is_valid()));
    columns.extend([
        bounds.corner_min().x,
        bounds.corner_min().y,
        bounds.corner_min().z,
        bounds.corner_max().x,
        bounds.corner_max().y,
        bounds.corner_max().z,
    ]);
    columns.extend([
        f64::from(probe.sensitivity_factor()),
        f64::from(probe.to_build_bvh()),
        f64::from(probe.owner_id().get().unwrap().priority()),
    ]);
}
fn append_triangle_columns(columns: &mut Vec<f64>, probe: &Select3DSensitiveTriangulation) {
    let triangle = probe.last_detected_triangle();
    let nodes = probe.last_detected_triangle_nodes();
    columns.extend([
        f64::from(probe.last_detected_triangle_index()),
        f64::from(triangle.is_some()),
        f64::from(nodes.is_some()),
    ]);
    let indices = triangle.map_or((0, 0, 0), |triangle| triangle.get());
    columns.extend([
        f64::from(indices.0),
        f64::from(indices.1),
        f64::from(indices.2),
    ]);
    for point in nodes.map_or([Point3::default(); 3], |(_, points)| points) {
        append_point_columns(columns, point);
    }
}
pub(crate) fn compare_oracle_snapshot(actual: &[f64], expected: &str) {
    let expected: Vec<f64> = expected
        .split_whitespace()
        .map(|token| token.parse().unwrap())
        .collect();
    assert_eq!(
        actual.len(),
        expected.len(),
        "scenario {} step {}",
        actual[0],
        actual[1]
    );
    compare_oracle_fields(actual, &expected);
}
fn compare_oracle_fields(actual: &[f64], expected: &[f64]) {
    for (field, (&rust, &native)) in actual.iter().zip(expected).enumerate() {
        assert!(
            rust == native || rust.is_nan() && native.is_nan() || (rust - native).abs() <= 1e-8,
            "scenario {} step {} field {field}: Rust {rust}, native {native}",
            actual[0],
            actual[1]
        );
    }
}

fn outcome_columns(ordinal: i32, step: i32, outcome: Result<bool, Failure>) -> Vec<f64> {
    vec![
        f64::from(ordinal),
        f64::from(step),
        failure_code(outcome.as_ref().err()),
        f64::from(outcome.unwrap_or(false)),
    ]
}
fn append_transform_columns(columns: &mut Vec<f64>, value: impl Fn(i32, i32) -> f64) {
    for row in 1..=3 {
        for col in 1..=4 {
            columns.push(value(row, col));
        }
    }
}
