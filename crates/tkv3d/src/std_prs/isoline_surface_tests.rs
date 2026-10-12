//! Native surface construction/sampling, retained output seeds and same-list references replay independently of OCCT.
use super::{
    StdPrsIsolineDestination, StdPrsIsolinePolylines, StdPrsIsolineUvOutputs, StdPrsIsolines,
    isoline_face_fixture::isoline_fixture_face,
};
use crate::{prs3d::Prs3dDrawer, select3d::compare_oracle_snapshot};
use rustycad_tkernel::{
    ncollection::{HSequence, Sequence},
    standard::{Handle, SharedState},
};
use rustycad_tkmath::gp::Point3;
const NATIVE_SURFACE: &str = include_str!("../../tests/fixtures/isolines/native-surface.txt");
#[test]
fn native_surface_parameter_geometry_and_shared_output_replay() {
    let mut rows = NATIVE_SURFACE.lines();
    for kind in 0..10 {
        let face = isoline_fixture_face(kind);
        compare_isoline_row(&isoline_parameter_row(&face, kind), rows.next().unwrap());
        for shared in 0..2 {
            compare_isoline_row(
                &isoline_surface_row(&face, kind, shared),
                rows.next().unwrap(),
            );
        }
    }
    assert!(rows.next().is_none());
}
fn isoline_parameter_row(face: &rustycad_tkbrep::topods::Face, kind: i32) -> Vec<f64> {
    let (u, v) = (
        SharedState::new(Sequence::default()),
        SharedState::new(Sequence::default()),
    );
    u.write().append(91.);
    v.write().append(92.);
    let bounds = [71., 72., 73., 74.].map(SharedState::new);
    populate_isoline_parameter_row(face, &u, &v, &bounds);
    let mut row = vec![f64::from(kind)];
    row.extend(bounds.each_ref().map(|value| *value.read()));
    for parameters in [&u, &v] {
        row.push(parameters.read().length() as f64);
        row.extend(parameters.read().iter().copied());
    }
    row
}
fn isoline_surface_row(face: &rustycad_tkbrep::topods::Face, kind: i32, shared: i32) -> Vec<f64> {
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    drawer.u_iso_aspect().set_number(2);
    drawer.v_iso_aspect().set_number(3);
    let (mut u, mut v) = (seeded_isoline_output(), seeded_isoline_output());
    let output = if shared != 0 {
        StdPrsIsolineDestination::Shared(&mut u)
    } else {
        StdPrsIsolineDestination::distinct(&mut u, &mut v)
    };
    let failure = StdPrsIsolines::add_on_surface_polylines(face, &drawer, 0.2, output).is_err();
    let mut row = vec![f64::from(kind), f64::from(shared), f64::from(failure)];
    append_isoline_snapshot(&u, &mut row);
    append_isoline_snapshot(&v, &mut row);
    row
}
fn seeded_isoline_output() -> StdPrsIsolinePolylines {
    let mut output = StdPrsIsolinePolylines::default();
    let seed = Handle::new(HSequence::default());
    seed.append(Point3::new(42., 43., 44.));
    output.append(seed);
    output
}
fn append_isoline_snapshot(output: &StdPrsIsolinePolylines, row: &mut Vec<f64>) {
    row.push(output.length() as f64);
    for points in output.iter() {
        points.sequence(|values| append_isoline_point_snapshot(values, row));
    }
}
fn compare_isoline_row(actual: &[f64], expected: &str) {
    let numeric = expected.split_once(' ').unwrap().1;
    compare_oracle_snapshot(actual, numeric);
}

fn populate_isoline_parameter_row(
    face: &rustycad_tkbrep::topods::Face,
    u: &SharedState<Sequence<f64>>,
    v: &SharedState<Sequence<f64>>,
    bounds: &[SharedState<f64>; 4],
) {
    let outputs = StdPrsIsolineUvOutputs {
        u,
        v,
        bounds: bounds.each_ref(),
    };
    StdPrsIsolines::uv_iso_parameters(face, [2, 3], 100., outputs).unwrap();
}

fn append_isoline_point_snapshot(values: &Sequence<Point3>, row: &mut Vec<f64>) {
    row.push(values.length() as f64);
    for point in values.iter() {
        row.extend([point.x(), point.y(), point.z()]);
    }
}

#[test]
fn native_vec_convenience_appends_the_same_real_polyline_points() {
    let face = isoline_fixture_face(0);
    let drawer = convenience_isoline_drawer();
    let mut u = vec![vec![Point3::new(42., 43., 44.)]];
    let mut v = u.clone();
    StdPrsIsolines::add_polylines(&face, &drawer, 0.2, &mut u, &mut v).unwrap();
    let mut row = vec![0., 0., 0.];
    append_vector_isoline_snapshot(&u, &mut row);
    append_vector_isoline_snapshot(&v, &mut row);
    compare_isoline_row(&row, NATIVE_SURFACE.lines().nth(1).unwrap());
}
fn convenience_isoline_drawer() -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    drawer.u_iso_aspect().set_number(2);
    drawer.v_iso_aspect().set_number(3);
    drawer.set_iso_on_triangulation(true);
    drawer
}
fn append_vector_isoline_snapshot(lines: &[Vec<Point3>], row: &mut Vec<f64>) {
    row.push(lines.len() as f64);
    for points in lines {
        row.push(points.len() as f64);
        row.extend(
            points
                .iter()
                .flat_map(|point| [point.x(), point.y(), point.z()]),
        );
    }
}
#[test]
fn vec_convenience_preserves_caller_seeds_after_native_checked_node_failure() {
    let face = invalid_node_isoline_face();
    let drawer = convenience_isoline_drawer();
    let mut u = vec![vec![Point3::new(42., 43., 44.)]];
    let mut v = u.clone();
    let expected = u.clone();
    assert!(StdPrsIsolines::add_polylines(&face, &drawer, 0.2, &mut u, &mut v).is_err());
    assert_eq!(u, expected);
    assert_eq!(v, expected);
}
fn invalid_node_isoline_face() -> rustycad_tkbrep::topods::Face {
    let face = isoline_fixture_face(0);
    let surface = super::isoline_face_fixture::isoline_fixture_surface(0);
    let mesh = super::isoline_triangle_fixture::isoline_triangle_grid(&surface, 0);
    mesh.set_triangle(1, rustycad_tkmath::poly::Triangle::new(0, 2, 5))
        .unwrap();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .update_face_triangulation(&face, &mesh, true)
        .unwrap();
    face
}
