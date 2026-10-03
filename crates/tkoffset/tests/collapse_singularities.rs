//! Native differential tests for BRepOffset::CollapseSingularities.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkoffset::brep_offset::collapse_singularities;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};

#[test]
fn non_bspline_surface_is_returned_unchanged() {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let result = collapse_singularities(&plane, &Face::default(), 1.0e-7).unwrap();
    assert_eq!(result.object_address(), plane.object_address());
}

#[test]
fn only_singular_bspline_boundary_poles_are_collapsed_to_their_centroid() {
    let surface = nearly_collapsed_surface();
    let face = surface_face(&surface);
    let input: Handle<dyn GeomSurface> = surface.clone().cast();
    let result = collapse_singularities(&input, &face, 0.01).unwrap();
    assert_native_surface("precision-only", &input, &result);
    assert!(
        surface
            .pole(1, 1)
            .unwrap()
            .is_equal(&Point3::new(0.0, 0.0, 0.0), 0.0)
    );
}

#[test]
fn degenerated_vertex_constraints_match_native_collapse_decisions() {
    for (name, points) in [
        ("no-constraint", Vec::new()),
        ("one-fitting-vertex", vec![Point3::new(0.0, 0.0005, 0.0)]),
        (
            "two-fitting-vertices",
            vec![Point3::new(0.0, 0.0005, 0.0); 2],
        ),
        ("no-fitting-vertex", vec![Point3::new(0.0, 0.1, 0.0)]),
    ] {
        assert_native_constraint_case(name, &points);
    }
}

fn assert_native_constraint_case(name: &str, points: &[Point3]) {
    let surface = nearly_collapsed_surface();
    let face = face_with_degenerated_vertices(&surface, points);
    let input: Handle<dyn GeomSurface> = surface.clone().cast();
    let result = collapse_singularities(&input, &face, 1.0e-7).unwrap();
    assert_native_surface(name, &input, &result);
}

fn assert_native_surface(
    name: &str,
    input: &Handle<dyn GeomSurface>,
    result: &Handle<dyn GeomSurface>,
) {
    let expected = native_fixture_row(name);
    assert_eq!(
        result.object_address() == input.object_address(),
        expected[1] == "1"
    );
    let output: Handle<GeomBSplineSurface> = result.cast();
    for (index, (u, v)) in [(1, 1), (1, 2), (2, 1), (2, 2)].into_iter().enumerate() {
        let expected_point = Point3::new(
            parse_number(&expected[2 + index * 3]),
            parse_number(&expected[3 + index * 3]),
            parse_number(&expected[4 + index * 3]),
        );
        assert_pole(&output, u, v, expected_point);
    }
}

fn native_fixture_row(name: &str) -> Vec<String> {
    include_str!("../../../tools/oracle/tkoffset_collapse_constraints.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn parse_number(value: &str) -> f64 {
    value.parse().unwrap()
}

fn face_with_degenerated_vertices(surface: &Handle<GeomBSplineSurface>, points: &[Point3]) -> Face {
    let builder = BRepBuilder::default();
    let base: Handle<dyn GeomSurface> = surface.clone().cast();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &base, 1.0e-7)
        .unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for point in points {
        builder
            .add(&mut wire, &degenerated_edge(point, &builder))
            .unwrap();
    }
    builder.add(&mut face, &wire).unwrap();
    face
}

fn degenerated_edge(point: &Point3, builder: &BRepBuilder) -> Shape {
    let mut vertex = Vertex::default();
    builder.make_vertex_at(&mut vertex, *point, 0.01).unwrap();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    builder.degenerated(&edge, true).unwrap();
    let mut shape: Shape = edge.into();
    let forward: Shape = vertex.clone().into();
    // TopExp::Vertices requires oppositely oriented endpoint occurrences to report IsSame.
    let reversed = Vertex::unchecked(vertex.oriented(Orientation::Reversed)).into_shape();
    builder.add(&mut shape, &forward).unwrap();
    builder.add(&mut shape, &reversed).unwrap();
    shape
}

#[test]
fn nonsingular_bspline_surface_keeps_its_original_handle() {
    let surface = nearly_collapsed_surface();
    let face = surface_face(&surface);
    let input: Handle<dyn GeomSurface> = surface.cast();
    let result = collapse_singularities(&input, &face, 1.0e-7).unwrap();
    assert_eq!(result.object_address(), input.object_address());
}

fn assert_pole(surface: &GeomBSplineSurface, u_index: i32, v_index: i32, expected: Point3) {
    assert!(
        surface
            .pole(u_index, v_index)
            .unwrap()
            .is_equal(&expected, 1.0e-15)
    );
}

fn surface_face(surface: &Handle<GeomBSplineSurface>) -> Face {
    let base: Handle<dyn GeomSurface> = surface.clone().cast();
    let mut maker =
        BRepBuilderAPIMakeFace::from_surface_bounds(&base, (0.0, 1.0, 0.0, 1.0), 1.0e-7).unwrap();
    Face::unchecked(maker.shape().unwrap().clone())
}

fn nearly_collapsed_surface() -> Handle<GeomBSplineSurface> {
    let poles = surface_poles();
    let knots = array1(&[0.0, 1.0]);
    let multiplicities = array1(&[2, 2]);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &multiplicities),
            (&knots, &multiplicities),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    )
}

fn surface_poles() -> Array2<'static, Point3> {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles.set_value(1, 1, Point3::new(0.0, 0.0, 0.0)).unwrap();
    poles.set_value(1, 2, Point3::new(0.0, 0.001, 0.0)).unwrap();
    poles.set_value(2, 1, Point3::new(1.0, 0.0, 0.0)).unwrap();
    poles.set_value(2, 2, Point3::new(1.0, 1.0, 0.0)).unwrap();
    poles
}

fn array1<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}
