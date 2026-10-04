use super::*;
use rustycad_tkbrep::topexp::{common_vertex, first_vertex, last_vertex};

const NATIVE_CONTROLS: &str =
    include_str!("../../../../tools/oracle/tkshhealing_unify_controls.expected");

#[test]
fn keep_shape_vertex_blocks_collinear_merge() {
    assert_kept_vertex_blocks_merge(false);
}

#[test]
fn keep_shapes_vertex_blocks_collinear_merge() {
    assert_kept_vertex_blocks_merge(true);
}

fn assert_kept_vertex_blocks_merge(plural: bool) {
    let input = if plural {
        line_chain(&[0., 1., 2., 3.])
    } else {
        line_chain(&[0., 1., 2.])
    };
    let edges = edge_shapes(&input);
    let kept = kept_chain_vertices(&edges, plural);
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    add_kept_shapes(&unifier, plural, &kept);
    unifier.build().unwrap();
    assert_eq!(edge_shapes(&unifier.shape()).len(), edges.len());
    assert_native_control(if plural {
        "keep_shapes_vertex edges 3"
    } else {
        "keep_shape_vertex edges 2"
    });
}

#[test]
fn safe_input_mode_copies_output_vertices_by_default() {
    assert_output_vertex_identity(true, false);
}

#[test]
fn unsafe_input_mode_reuses_original_output_vertices() {
    assert_output_vertex_identity(false, true);
}

fn assert_output_vertex_identity(safe: bool, expected_same: bool) {
    let input = line_chain(&[0., 1., 2.]);
    let (first, second) = first_two_edges(&input);
    let original_start = first_vertex(&first, true);
    let original_end = last_vertex(&second, true);
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.set_safe_input_mode(safe);
    unifier.build().unwrap();
    let (result_start, result_end) = edge_endpoints(&edge_shapes(&unifier.shape())[0]);
    assert_eq!(result_start.is_same(&original_start), expected_same);
    assert_eq!(result_end.is_same(&original_end), expected_same);
    assert_native_control(if safe {
        "safe_input endpoint_identity 0 0"
    } else {
        "unsafe_input endpoint_identity 1 1"
    });
}

#[test]
fn keep_shape_edge_blocks_coplanar_face_merge() {
    let first = face_with_internal_path(0., 1., 0., 1., &[]);
    let second = face_with_internal_path(1., 2., 0., 1., &[]);
    let input = sew_coplanar_faces(first, second);
    let shared = shared_edge(&input);
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, false, true, false);
    unifier.keep_shape(&shared.into_shape()).unwrap();
    unifier.build().unwrap();
    assert_eq!(face_count(&unifier.shape()), 2);
    assert_native_control("keep_shape_edge faces 2");
}

#[test]
fn keep_shape_ignores_non_edge_and_non_vertex_types() {
    let input = adjacent_coplanar_faces();
    let face = first_face(&input);
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, false, true, false);
    unifier.keep_shape(&face).unwrap();
    unifier.build().unwrap();
    assert_eq!(face_count(&unifier.shape()), 1);
    assert_native_control("keep_shape_face_ignored faces 1");
}

#[test]
fn initialize_clears_kept_shapes_and_preserves_safe_mode() {
    let input = line_chain(&[0., 1., 2.]);
    let (first, second) = first_two_edges(&input);
    let junction = common_vertex(&first, &second).unwrap();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.keep_shape(&junction.into_shape()).unwrap();
    unifier.set_safe_input_mode(false);
    unifier.initialize(&input, true, false, false);
    unifier.build().unwrap();
    assert_eq!(edge_shapes(&unifier.shape()).len(), 1);
    assert_original_endpoints_reused(&input, &unifier.shape());
    assert_native_control("initialize_resets_kept_and_preserves_safe_mode edges 1 identities 1 1");
}

fn first_two_edges(
    shape: &Shape,
) -> (rustycad_tkbrep::topods::Edge, rustycad_tkbrep::topods::Edge) {
    let mut edges = edge_shapes(shape).into_iter();
    (edges.next().unwrap(), edges.next().unwrap())
}

fn add_kept_shapes(unifier: &ShapeUpgradeUnifySameDomain, plural: bool, shapes: &[Shape]) {
    if plural {
        unifier.keep_shapes(shapes).unwrap();
        return;
    }
    for shape in shapes {
        unifier.keep_shape(shape).unwrap();
    }
}

fn kept_chain_vertices(edges: &[rustycad_tkbrep::topods::Edge], plural: bool) -> Vec<Shape> {
    let junctions = if plural { edges.len() - 1 } else { 1 };
    (0..junctions)
        .map(|index| {
            common_vertex(&edges[index], &edges[index + 1])
                .unwrap()
                .into_shape()
        })
        .collect()
}

fn shared_edge(shape: &Shape) -> rustycad_tkbrep::topods::Edge {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(|face| rustycad_tkbrep::topods::face(&face).unwrap())
        .collect::<Vec<_>>();
    let first = Explorer::new(
        &faces[0].clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    .map(|edge| rustycad_tkbrep::topods::edge(&edge).unwrap())
    .collect::<Vec<_>>();
    let second = Explorer::new(
        &faces[1].clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    .map(|edge| rustycad_tkbrep::topods::edge(&edge).unwrap())
    .collect::<Vec<_>>();
    first
        .into_iter()
        .find(|edge| second.iter().any(|candidate| edge.is_same(candidate)))
        .unwrap()
}

fn adjacent_coplanar_faces() -> Shape {
    let first = face_with_internal_path(0., 1., 0., 1., &[]);
    let second = face_with_internal_path(1., 2., 0., 1., &[]);
    sew_coplanar_faces(first, second)
}

fn first_face(shape: &Shape) -> Shape {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .collect::<Vec<_>>()[0]
        .clone()
}

fn assert_original_endpoints_reused(input: &Shape, result: &Shape) {
    let (first, second) = first_two_edges(input);
    let original_start = first_vertex(&first, true);
    let original_end = last_vertex(&second, true);
    let (result_start, result_end) = edge_endpoints(&edge_shapes(result)[0]);
    assert!(result_start.is_same(&original_start));
    assert!(result_end.is_same(&original_end));
}

fn assert_native_control(record: &str) {
    assert!(NATIVE_CONTROLS.lines().any(|line| line == record));
}
