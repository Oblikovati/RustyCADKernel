use super::{BOPAlgoPaveFiller, Glue};
use crate::{
    bop_ds::{DataStructure, InterfEE, InterfFF},
    int_tools::{Curve, Tools},
};
use rustycad_tkernel::ncollection::List;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Plane3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn plane_cylinder_perform_records_native_section_curve() {
    let mut filler = configured_filler();
    filler.initialize().unwrap();
    filler.perform_ff().unwrap();

    let record = filler
        .data_structure_mut()
        .unwrap()
        .interf_ff()
        .value(0)
        .unwrap();
    let expected = native_values();
    assert_eq!(expected[0], 1.0);
    assert_eq!(record.indices(), (expected[1] as i32, expected[2] as i32));
    assert_eq!(record.curves().length(), expected[3] as i32);
    assert_eq!(record.points().length(), expected[4] as i32);
    let curve = record.curves().value(0).unwrap();
    assert_eq!(curve.curve().curve().first_parameter(), expected[6]);
    assert_eq!(curve.curve().curve().last_parameter(), expected[7]);
    assert_eq!(curve.tolerance(), expected[8]);
    assert_box(curve.box_bounds(), &expected[9..15]);
    assert_eq!(
        filler
            .data_structure()
            .unwrap()
            .face_info(0)
            .unwrap()
            .vertices_on()
            .extent(),
        expected[19] as usize
    );
    assert_eq!(
        filler
            .data_structure()
            .unwrap()
            .face_info(10)
            .unwrap()
            .vertices_on()
            .extent(),
        expected[21] as usize
    );
    assert_eq!(expected[17], 3.0);
    assert_eq!(expected[21], 2.0);
    assert_stage_face_info(&filler, &expected[19..23]);
}

fn assert_stage_face_info(filler: &BOPAlgoPaveFiller, expected: &[f64]) {
    let structure = filler.data_structure().unwrap();
    let first = structure.face_info(0).unwrap();
    let second = structure.face_info(10).unwrap();
    assert_eq!(first.vertices_on().extent(), expected[0] as usize);
    assert_eq!(first.vertices_in().extent(), expected[1] as usize);
    assert_eq!(second.vertices_on().extent(), expected[2] as usize);
    assert_eq!(second.vertices_in().extent(), expected[3] as usize);
}

#[test]
fn glue_mode_records_face_pairs_without_intersection_tasks() {
    let mut filler = configured_filler();
    filler.set_glue(Glue::Full);
    filler.initialize().unwrap();
    filler.perform_ff().unwrap();

    let records = filler.data_structure_mut().unwrap().interf_ff();
    assert_eq!(records.length(), 1);
    assert_eq!(records.value(0).unwrap().curves().length(), 0);
}

#[test]
fn edge_face_seed_input_fails_explicitly_until_native_walking_is_ported() {
    let mut filler = configured_filler();
    filler.initialize().unwrap();
    let mut seed = InterfEE::default();
    seed.set_index_new(8);
    filler
        .data_structure
        .as_mut()
        .unwrap()
        .interf_ee()
        .append(seed);

    let error = filler.perform_ff().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
}

#[test]
fn raw_invalid_section_curve_still_marks_face_pair_interference() {
    let curve = Curve::default();
    let mut bounds = rustycad_tkmath::bnd::BoundingBox3::default();
    assert!(!Tools::check_curve(&curve, &mut bounds).unwrap());
    let mut structure = DataStructure::default();
    let raw_results = super::face_face_perform::raw_face_face_results_exist(1, 0);
    super::face_face_perform::append_face_face_result(
        &mut structure,
        [0, 10],
        InterfFF::default(),
        raw_results,
    )
    .unwrap();
    assert!(structure.has_interference_between(0, 10));
    assert_eq!(structure.interf_ff().length(), 1);
    assert!(!super::face_face_perform::raw_face_face_results_exist(0, 0));
}

fn configured_filler() -> BOPAlgoPaveFiller {
    let first = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-3., 3., -3., 3.))
        .unwrap()
        .face()
        .unwrap();
    let cylinder = Cylinder3::new(Axis3::default(), 1.).unwrap();
    let second = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        cylinder,
        (0., std::f64::consts::TAU, -2., 2.),
    )
    .unwrap()
    .face()
    .unwrap();
    let mut arguments = List::default();
    arguments.append(first.into_shape());
    arguments.append(second.into_shape());
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&arguments);
    filler
}

fn assert_box(bounds: &rustycad_tkmath::bnd::BoundingBox3, expected: &[f64]) {
    let minimum = bounds.corner_min().unwrap();
    let maximum = bounds.corner_max().unwrap();
    let actual = [
        minimum.x(),
        minimum.y(),
        minimum.z(),
        maximum.x(),
        maximum.y(),
        maximum.z(),
    ];
    for (value, target) in actual.iter().zip(expected) {
        assert!((value - target).abs() < 1e-14);
    }
}

fn native_values() -> Vec<f64> {
    include_str!("../../../../../tools/oracle/tkbo_perform_ff.expected")
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}
