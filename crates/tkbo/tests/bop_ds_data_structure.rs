//! Exact storage behavior shared by OCCT's pre-Init BOPDS data structure.
use rustycad_tkbo::bop_ds::{DataStructure, ShapeInfo};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkmath::gp::Point3;

fn vertex_at(x: f64) -> Shape {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1e-7)
        .unwrap();
    vertex.into_shape()
}

#[test]
fn append_indexes_use_is_same_and_rebind_duplicate_shapes() {
    let mut structure = DataStructure::default();
    let first = vertex_at(1.0);
    let second = vertex_at(2.0);
    assert_eq!(structure.index(&first), -1);
    assert_eq!(structure.append_shape(&first), 0);
    assert_eq!(structure.append_shape(&second), 1);
    assert_eq!(structure.append_shape(&first.reversed()), 2);
    assert_eq!(structure.nb_shapes(), 3);
    assert_eq!(structure.index(&first), 2);
    assert_eq!(structure.index(&first.reversed()), 2);
    assert_eq!(structure.index(&second), 1);
    assert!(structure.shape(0).unwrap().is_same(&first));
}

#[test]
fn append_shape_info_copies_metadata_and_clear_resets_storage() {
    let mut structure = DataStructure::default();
    let shape = vertex_at(3.0);
    append_metadata(&mut structure, &shape);
    assert_pre_init_state(&structure, &shape);
    assert_checked_access_rejects_missing_slots(&structure);
    structure.clear();
    assert_cleared_state(&structure, &shape);
}

fn append_metadata(structure: &mut DataStructure, shape: &Shape) {
    structure.set_arguments(std::slice::from_ref(shape));
    let mut info = ShapeInfo::default();
    info.set_shape(shape);
    info.set_flag(17);
    assert_eq!(structure.append_shape_info(&info), 0);
    structure.change_shape_info(0).unwrap().set_flag(21);
    assert_eq!(info.flag(), 17);
    assert_eq!(structure.shape_info(0).unwrap().flag(), 21);
}

fn assert_pre_init_state(structure: &DataStructure, shape: &Shape) {
    assert_eq!(structure.arguments().len(), 1);
    assert_eq!(structure.nb_source_shapes(), 0);
    assert_eq!(structure.nb_ranges(), 0);
    assert_eq!(structure.rank(0), -1);
    assert!(structure.is_new_shape(0));
    assert!(structure.index(shape) >= 0);
}

fn assert_checked_access_rejects_missing_slots(structure: &DataStructure) {
    assert!(structure.range(0).is_err());
    assert!(structure.shape_info(1).is_err());
}

fn assert_cleared_state(structure: &DataStructure, shape: &Shape) {
    assert!(structure.arguments().is_empty());
    assert_eq!(structure.nb_shapes(), 0);
    assert_eq!(structure.index(shape), -1);
}
