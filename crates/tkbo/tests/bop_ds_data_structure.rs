//! Exact storage and source-vertex initialization behavior of BOPDS_DS.
use rustycad_tkbo::bop_ds::{DataStructure, IndexRange, ShapeInfo};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Compound, Shape, Vertex};
use rustycad_tkg3d::top_abs::ShapeEnum;
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

#[test]
fn init_preserves_depth_first_shape_order_ranges_and_vertex_gaps() {
    let builder = BRepBuilder::default();
    let (root, first, second, third) = topology_arguments(&builder);
    let mut structure = DataStructure::default();
    structure.set_arguments(&[root.clone(), first.clone(), third.clone()]);
    structure.init(4e-3).unwrap();
    assert_initialized_ranges(&structure);
    assert_initialized_shapes(&structure, &root, &first, &second, &third);
    assert_vertex_bounds(&structure, 1, &first, 4e-3);
    assert_vertex_bounds(&structure, 2, &second, 4e-3);
    assert_vertex_bounds(&structure, 3, &third, 4e-3);
}

fn topology_arguments(builder: &BRepBuilder) -> (Shape, Shape, Shape, Shape) {
    let first = vertex_with_tolerance(builder, [1.0, 2.0, 3.0], 2e-5);
    let second = vertex_with_tolerance(builder, [4.0, 5.0, 6.0], 5e-7);
    let third = vertex_with_tolerance(builder, [7.0, 8.0, 9.0], 1e-4);
    let mut root = Compound::default();
    builder.make_compound(&mut root);
    builder.add(&mut root, &first).unwrap();
    builder.add(&mut root, &second).unwrap();
    builder.add(&mut root, &first).unwrap();
    (root.into_shape(), first, second, third)
}

fn vertex_with_tolerance(builder: &BRepBuilder, point: [f64; 3], tolerance: f64) -> Shape {
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(
            &mut vertex,
            Point3::new(point[0], point[1], point[2]),
            tolerance,
        )
        .unwrap();
    vertex.into_shape()
}

fn assert_initialized_ranges(structure: &DataStructure) {
    assert_eq!(structure.nb_shapes(), 4);
    assert_eq!(structure.nb_source_shapes(), 4);
    assert_eq!(structure.nb_ranges(), 2);
    assert_eq!(structure.range(0).unwrap(), &IndexRange::new(0, 2));
    assert_eq!(structure.range(1).unwrap(), &IndexRange::new(3, 3));
    assert_eq!(structure.rank(0), 0);
    assert_eq!(structure.rank(2), 0);
    assert_eq!(structure.rank(3), 1);
}

fn assert_initialized_shapes(
    structure: &DataStructure,
    root: &Shape,
    first: &Shape,
    second: &Shape,
    third: &Shape,
) {
    assert_eq!(structure.index(root), 0);
    assert_eq!(structure.index(first), 1);
    assert_eq!(structure.index(second), 2);
    assert_eq!(structure.index(third), 3);
    assert_eq!(
        structure.shape_info(0).unwrap().shape_type(),
        ShapeEnum::Compound
    );
    assert_eq!(
        structure.shape_info(1).unwrap().shape_type(),
        ShapeEnum::Vertex
    );
    assert_eq!(structure.shape_info(0).unwrap().sub_shapes(), &[1, 2]);
}

fn assert_vertex_bounds(structure: &DataStructure, index: i32, shape: &Shape, fuzz: f64) {
    let vertex = Vertex::unchecked(shape.clone());
    let tolerance = BRepTool::vertex_tolerance(&vertex).unwrap();
    let point = BRepTool::pnt(&vertex).unwrap();
    let expected_gap = tolerance + fuzz.max(1e-7) * 0.5;
    let box_bounds = structure.shape_info(index).unwrap().box_bounds();
    assert!((box_bounds.gap() - expected_gap).abs() < 1e-15);
    let bounds = box_bounds.get().unwrap();
    for axis in 0..3 {
        assert!((bounds[axis] - (point.coordinates()[axis] - expected_gap)).abs() < 1e-14);
        assert!((bounds[axis + 3] - (point.coordinates()[axis] + expected_gap)).abs() < 1e-14);
    }
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
