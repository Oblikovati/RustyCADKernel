use super::*;

const NATIVE_GLOBAL: &str =
    include_str!("../../../../tools/oracle/tkshhealing_unify_global_context.expected");

#[test]
fn shared_boundary_is_merged_once_across_sewn_faces() {
    let input = sewn_split_faces();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    assert_global_counts(&result, "two_face_shell");
    assert_merged_shared_boundary(&result);
}

#[test]
fn one_context_rewrites_disconnected_faces_in_a_compound() {
    let input = compound_of_split_faces();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_global_counts(&unifier.shape(), "two_face_compound");
}

#[test]
fn shared_context_reaches_solid_faces_and_preserves_validity() {
    let input = prism_of_split_face();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    assert_global_counts(&result, "prism_solid");
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

fn sewn_split_faces() -> Shape {
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&split_side(0., 1., true)).unwrap();
    sewing.add(&split_side(1., 2., false)).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing.sewed_shape().clone()
}

fn split_side(x0: f64, x1: f64, split_right: bool) -> Shape {
    let points = if split_right {
        split_right_points(x0, x1)
    } else {
        split_left_points(x0, x1)
    };
    split_planar_face(&points)
}

fn split_right_points(x0: f64, x1: f64) -> [Point3; 6] {
    [
        Point3::new(x0, 0., 0.),
        Point3::new(x1, 0., 0.),
        Point3::new(x1, 0.5, 0.),
        Point3::new(x1, 1., 0.),
        Point3::new(x0, 1., 0.),
        Point3::new(x0, 0., 0.),
    ]
}

fn split_left_points(x0: f64, x1: f64) -> [Point3; 6] {
    [
        Point3::new(x0, 0., 0.),
        Point3::new(x1, 0., 0.),
        Point3::new(x1, 1., 0.),
        Point3::new(x0, 1., 0.),
        Point3::new(x0, 0.5, 0.),
        Point3::new(x0, 0., 0.),
    ]
}

fn split_planar_face(points: &[Point3]) -> Shape {
    let wire = wire_from_points(points);
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn compound_of_split_faces() -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    builder.make_compound(&mut compound);
    builder
        .add(&mut compound, &split_side(0., 1., true))
        .unwrap();
    builder
        .add(&mut compound, &split_side(1., 2., false))
        .unwrap();
    compound.into_shape()
}

fn prism_of_split_face() -> Shape {
    rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism::new(
        &split_side(0., 2., true),
        rustycad_tkmath::gp::Vector3::new(0., 0., 1.),
        false,
        true,
    )
    .unwrap()
    .shape()
    .unwrap()
    .clone()
}

fn assert_global_counts(shape: &Shape, case: &str) {
    let actual = format!(
        "{case} faces {} wires {} edges {} unique_edges {} vertices {} valid {}",
        Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .count(),
        Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)
            .unwrap()
            .count(),
        edge_shapes(shape).len(),
        unique_edges(shape),
        Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)
            .unwrap()
            .count(),
        i32::from(
            BRepCheckAnalyzer::new(shape, true, false, false)
                .unwrap()
                .is_valid()
                .unwrap()
        ),
    );
    assert!(NATIVE_GLOBAL.lines().any(|expected| expected == actual));
}

fn unique_edges(shape: &Shape) -> usize {
    let edges = edge_shapes(shape);
    edges
        .iter()
        .enumerate()
        .filter(|(index, edge)| !edges[..*index].iter().any(|earlier| earlier.is_same(edge)))
        .count()
}

fn assert_merged_shared_boundary(shape: &Shape) {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(|face| rustycad_tkbrep::topods::face(&face).unwrap())
        .collect::<Vec<_>>();
    let first = edges_of_face(&faces[0]);
    let second = edges_of_face(&faces[1]);
    let shared = first
        .iter()
        .find(|edge| second.iter().any(|other| edge.is_same(other)))
        .expect("the sewn faces retain one shared boundary edge");
    let (start, end) = edge_endpoints(shared);
    let actual = endpoint_coordinates(start, end);
    let expected = vec![1., 0., 0., 1., 1., 0.];
    let reversed = vec![1., 1., 0., 1., 0., 0.];
    assert!(actual == expected || actual == reversed);
    assert_eq!(unique_edges(shape), 7);
}

fn edges_of_face(face: &rustycad_tkbrep::topods::Face) -> Vec<rustycad_tkbrep::topods::Edge> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    .map(|edge| rustycad_tkbrep::topods::edge(&edge).unwrap())
    .collect()
}
