//! Native plane-pair preparation, including duplicate In/On membership.
use rustycad_tkbo::{bop_algo::BOPAlgoPaveFiller, bop_ds::FaceInfo};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::fmt::Write;

const MEMBERSHIP_MASKS: [[i32; 4]; 8] = [
    [0, 0, 0, 0],
    [1, 0, 1, 0],
    [3, 0, 3, 0],
    [1, 0, 1, 1],
    [1, 1, 1, 0],
    [1, 2, 0, 3],
    [1, 0, 2, 4],
    [0, 3, 1, 2],
];

#[test]
fn plane_preparation_matches_native_lazy_state_and_membership_multiplicity() {
    let (mut filler, first, second) = plane_arguments();
    let mut rows = String::new();
    append_lazy_state(&filler, first, second, "lazy", &mut rows);
    append_decision(&mut filler, first, second, "initial", &mut rows);
    append_lazy_state(&filler, first, second, "ready", &mut rows);
    append_decision(&mut filler, first, first, "same", &mut rows);
    append_membership_cases(&mut filler, first, second, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_check_planes.expected")
    );
}

fn plane_arguments() -> (BOPAlgoPaveFiller, i32, i32) {
    let first = plane_face(0.0);
    let second = plane_face(2.0);
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&first);
    filler.add_argument(&second);
    filler.initialize().unwrap();
    let structure = filler.data_structure().unwrap();
    let indices = (structure.index(&first), structure.index(&second));
    (filler, indices.0, indices.1)
}

fn plane_face(height: f64) -> Shape {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, height),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-1.0, 1.0, -1.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn append_lazy_state(
    filler: &BOPAlgoPaveFiller,
    first: i32,
    second: i32,
    label: &str,
    rows: &mut String,
) {
    let structure = filler.data_structure().unwrap();
    writeln!(
        rows,
        "{label} {} {}",
        structure.has_face_info(first).unwrap() as u8,
        structure.has_face_info(second).unwrap() as u8
    )
    .unwrap();
}

fn append_membership_cases(
    filler: &mut BOPAlgoPaveFiller,
    first: i32,
    second: i32,
    rows: &mut String,
) {
    for (index, mask) in MEMBERSHIP_MASKS.into_iter().enumerate() {
        let structure = filler.data_structure_mut().unwrap();
        assign_membership(structure.change_face_info(first).unwrap(), mask[0], mask[1]);
        assign_membership(
            structure.change_face_info(second).unwrap(),
            mask[2],
            mask[3],
        );
        append_decision(filler, first, second, &format!("case {index}"), rows);
    }
}

fn assign_membership(info: &mut FaceInfo, inside: i32, on: i32) {
    info.change_vertices_in().clear();
    info.change_vertices_on().clear();
    for index in 0..3 {
        if inside & (1 << index) != 0 {
            info.change_vertices_in().add(index + 2);
        }
        if on & (1 << index) != 0 {
            info.change_vertices_on().add(index + 2);
        }
    }
}

fn append_decision(
    filler: &mut BOPAlgoPaveFiller,
    first: i32,
    second: i32,
    label: &str,
    rows: &mut String,
) {
    let result = filler.check_planes(first, second).unwrap();
    writeln!(rows, "{label} {}", result as u8).unwrap();
}

#[test]
fn uninitialized_plane_preparation_reports_missing_source_structure() {
    let mut filler = BOPAlgoPaveFiller::new();
    let error = filler.check_planes(0, 1).unwrap_err();
    assert!(error.message().contains("data structure is None"));
    assert!(error.message().contains("expected initialize()"));
}
