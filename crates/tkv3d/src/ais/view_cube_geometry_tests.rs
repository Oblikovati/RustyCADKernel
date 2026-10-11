//! Every native orientation, shape policy, sizing-only pass and transformed rectangle.
use super::{view_cube_fixture_replay::*, view_cube_test_probe::*, *};
use crate::v3d::V3dTypeOfOrientation as Orientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Axis1, Direction3, Point2, Point3, Transform3};
use rustycad_tkservice::graphic3d::*;
#[test]
fn native_cube_orientation_classification() {
    for row in fixture_rows(GEOMETRY, "classify") {
        let orientation = Orientation::ALL[row[0].parse::<usize>().unwrap()];
        assert_eq!(ais_view_cube_is_box_side(orientation), row[1] == "1");
        assert_eq!(ais_view_cube_is_box_edge(orientation), row[2] == "1");
        assert_eq!(ais_view_cube_is_box_corner(orientation), row[3] == "1");
    }
}
#[test]
fn native_cube_capacity_geometry_normals_and_winding() {
    for row in fixture_rows(GEOMETRY, "capacity") {
        replay_cube_capacity(&row);
    }
}
fn replay_cube_capacity(row: &[&str]) {
    let id = row[0].parse::<i32>().unwrap();
    let orientation = row[1].parse::<usize>().unwrap();
    let cube = configured_cube(id);
    let mut counts = AisViewCubeTriangleCounts::default();
    cube.create_box_part_triangles(
        &Handle::default(),
        &mut counts,
        Orientation::ALL[orientation],
    )
    .unwrap();
    assert_eq!(counts, expected_cube_capacity(row));
    if counts.vertices != 0 {
        replay_cube_geometry(
            &cube,
            id * 26 + orientation as i32,
            Orientation::ALL[orientation],
            counts,
        );
    }
}

fn expected_cube_capacity(row: &[&str]) -> AisViewCubeTriangleCounts {
    AisViewCubeTriangleCounts {
        vertices: row[2].parse().unwrap(),
        triangles: row[3].parse().unwrap(),
    }
}

fn replay_cube_geometry(
    cube: &CubePresentationProbe,
    id: i32,
    orientation: Orientation,
    counts: AisViewCubeTriangleCounts,
) {
    let array = Handle::new(
        Graphic3dArrayOfTriangles::new(
            counts.vertices,
            counts.triangles * 3,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
        )
        .unwrap(),
    );
    let mut actual_counts = AisViewCubeTriangleCounts::default();
    cube.create_box_part_triangles(&array, &mut actual_counts, orientation)
        .unwrap();
    assert_eq!(counts, actual_counts);
    let row = fixture_rows(GEOMETRY, "array")
        .find(|row| row[0].parse::<i32>().unwrap() == id)
        .unwrap();
    assert_cube_request(&actual_cube_array(&array), &parse_cube_array(&row));
}
#[test]
fn native_rounded_rectangles_clamp_radius_transform_frames_and_preserve_count_overestimate() {
    for row in fixture_rows(GEOMETRY, "rectangle") {
        replay_cube_rectangle(&row);
    }
}
fn replay_cube_rectangle(row: &[&str]) {
    let mut counts = AisViewCubeTriangleCounts::default();
    let transform = rectangle_test_transform();
    let size = Point2::new(row[1].parse().unwrap(), row[2].parse().unwrap());
    let radius = row[3].parse().unwrap();
    ais_view_cube_round_rectangle_triangles(
        &Handle::default(),
        &mut counts,
        size,
        radius,
        &transform,
    )
    .unwrap();
    assert_eq!(
        (counts.vertices, counts.triangles),
        (row[4].parse().unwrap(), row[5].parse().unwrap())
    );
    replay_cube_rectangle_array(row[0].parse().unwrap(), counts, size, radius, &transform);
}
fn replay_cube_rectangle_array(
    id: i32,
    counts: AisViewCubeTriangleCounts,
    size: Point2,
    radius: f64,
    transform: &Transform3,
) {
    let array = Handle::new(
        Graphic3dArrayOfTriangles::new(
            counts.vertices,
            counts.triangles * 3,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
        )
        .unwrap(),
    );
    ais_view_cube_round_rectangle_triangles(
        &array,
        &mut AisViewCubeTriangleCounts::default(),
        size,
        radius,
        transform,
    )
    .unwrap();
    let expected = fixture_rows(GEOMETRY, "array")
        .find(|r| r[0].parse::<i32>().unwrap() == 208 + id)
        .unwrap();
    assert_cube_request(&actual_cube_array(&array), &parse_cube_array(&expected));
}

fn rectangle_test_transform() -> Transform3 {
    let axis = Axis1::new(
        Point3::new(2.0, -3.0, 5.0),
        Direction3::new(1.0, 2.0, 3.0).unwrap(),
    );
    let mut transform = Transform3::default();
    transform.set_rotation(axis, 0.3).unwrap();
    transform
}
