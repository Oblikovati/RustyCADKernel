//! Supplemental regressions for the Poly polygons, shared parameter arrays, text I/O and
//! the static Poly geometry functions.
use rustycad_tkernel::{
    ncollection::{Array1, List},
    standard::Handle,
};
use rustycad_tkmath::{
    gp::{Axis1, Direction3, Point2, Point3, Xy, Xyz},
    poly::{
        self, HArray1OfReal, Polygon2D, Polygon3D, PolygonOnTriangulation, Triangle, Triangulation,
    },
};
use std::io::Cursor;
/// An owned array holding `values` from `lower`; `table(&[1.0], 1)`.
fn table<T: Clone + Default>(values: &[T], lower: i32) -> Array1<'static, T> {
    let mut array = Array1::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    array.as_mut_slice().clone_from_slice(values);
    array
}
#[test]
fn polygon_on_triangulation_keeps_input_bounds_and_shares_parameters() {
    let polygon = PolygonOnTriangulation::from_nodes(&table(&[4, 5, 6], 0));
    assert_eq!(polygon.node(0).unwrap(), 4);
    assert_eq!(
        polygon.parameter(0).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    let shared = HArray1OfReal::shared(table(&[0.5, 1.5, 2.5], 0));
    polygon.set_parameters(shared.clone()).unwrap();
    shared.set_value(1, 9.0).unwrap();
    assert_eq!(polygon.parameter(1).unwrap(), 9.0);
    let wrong = HArray1OfReal::shared(Array1::with_bounds(1, 3).unwrap());
    assert_eq!(
        polygon.set_parameters(wrong).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    polygon.set_parameters(Handle::default()).unwrap();
    assert!(!polygon.has_parameters());
    // Copy allocates 1..=NbNodes, then copies values of any bounds.
    assert_eq!(polygon.copy().unwrap().node(1).unwrap(), 4);
}
#[test]
fn polygon3d_parameters_use_node_indices_and_copy_checks_sizes() {
    let nodes = table(&[Point3::default(), Point3::new(1.0, 2.0, 3.0)], 1);
    let parameters = table(&[0.25, 0.75, 1.0], 1);
    let polygon = Polygon3D::from_nodes_and_parameters(&nodes, &parameters).unwrap();
    assert_eq!(polygon.parameters().unwrap().len(), 3);
    assert_eq!(
        polygon.copy().err().unwrap().exception_type(),
        "Standard_DimensionMismatch"
    );
    let shifted = table(&[0.25, 0.75], 0);
    assert!(Polygon3D::from_nodes_and_parameters(&nodes, &shifted).is_err());
    assert_eq!(
        Polygon3D::from_nodes(&nodes)
            .parameters()
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
    let copy = Polygon2D::new(2).unwrap().copy();
    assert_eq!(copy.nodes().upper(), 2);
}
#[test]
fn text_round_trip_rounds_to_six_digits_and_skips_foreign_headers() {
    let mesh = Triangulation::with_sizes(3, 1, true, false).unwrap();
    for (index, x) in [(1, 0.123456789), (2, 1e-7), (3, 12345678.0)] {
        mesh.set_node(index, Point3::new(x, -x, 0.5)).unwrap();
        mesh.set_uv_node(index, Point2::new(x, 1.0)).unwrap();
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    mesh.set_deflection(0.001);
    let mut text = Vec::new();
    poly::write_triangulation(&mesh, &mut text, true).unwrap();
    let written = String::from_utf8(text.clone()).unwrap();
    assert!(written.starts_with(
        "Poly_Triangulation\n3 1 1\n0.001\n0.123457 -0.123457 0.5\n1e-07 -1e-07 0.5\n1.23457e+07"
    ));
    let mut input = Cursor::new(text);
    let read = poly::read_triangulation(&mut input).unwrap();
    assert_eq!(read.node(1).unwrap().x(), 0.123457);
    assert_eq!(read.uv_node(3).unwrap().x(), 12345700.0);
    let mut foreign = Cursor::new(b"Poly_Polygon3D 1 0 0 0 0 0".to_vec());
    assert!(poly::read_triangulation(&mut foreign).unwrap().is_null());
    let mut broken = Cursor::new(b"Poly_Polygon2D 2 0 1 inf 3 4".to_vec());
    assert!(poly::read_polygon2d(&mut broken).is_err());
}
#[test]
fn dump_uses_native_columns() {
    let polygon = Polygon2D::from_nodes(&table(&[Point2::new(1.5, -2.0)], 7));
    let mut text = Vec::new();
    poly::dump_polygon2d(&polygon, &mut text).unwrap();
    let expected = "Poly_Polygon2D\n       1 Nodes\nDeflection : 0\n\nNodes :\n         1 :               1.5                -2\n";
    assert_eq!(String::from_utf8(text).unwrap(), expected);
}
#[test]
fn point_on_triangle_projects_outside_points_and_degenerate_triangles() {
    let (p1, p2, p3) = (Xy::new(0.0, 0.0), Xy::new(1.0, 0.0), Xy::new(0.0, 1.0));
    assert_eq!(
        poly::point_on_triangle(p1, p2, p3, Xy::new(0.25, 0.25)),
        (0.0, Xy::new(0.25, 0.25))
    );
    assert_eq!(
        poly::point_on_triangle(p1, p2, p3, Xy::new(2.0, 0.0)),
        (1.0, Xy::new(1.0, 0.0))
    );
    // Collinear sides report the unclamped parameter of the closer side.
    let (distance, uv) = poly::point_on_triangle(p1, p2, Xy::new(2.0, 0.0), Xy::new(3.0, 1.0));
    assert_eq!((distance, uv), (2.0, Xy::new(0.0, 1.5)));
    let (distance, _) = poly::point_on_triangle(p1, p1, p1, Xy::new(3.0, 4.0));
    assert_eq!(distance, 25.0);
}
#[test]
fn ray_and_line_intersections_and_polygon_properties() {
    let start = Xyz::new(0.2, 0.2, -1.0);
    let hit = poly::intersect_tri_line(
        start,
        Direction3::Z,
        Xyz::new(0.0, 0.0, 0.0),
        Xyz::new(1.0, 0.0, 0.0),
        Xyz::new(0.0, 1.0, 0.0),
    );
    assert_eq!(hit, Some(1.0));
    let mesh = Triangulation::with_sizes(3, 1, false, false).unwrap();
    for (index, point) in [(1, (0.0, 0.0)), (2, (1.0, 0.0)), (3, (0.0, 1.0))] {
        mesh.set_node(index, Point3::new(point.0, point.1, 0.0))
            .unwrap();
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    let (mut triangle, mut distance) = (Triangle::default(), 7.0);
    let axis = Axis1::new(Point3::new(0.2, 0.2, -2.0), Direction3::Z);
    assert!(poly::intersect(&mesh, axis, true, &mut triangle, &mut distance).unwrap());
    assert_eq!((triangle, distance), (Triangle::new(1, 2, 3), 2.0));
    let square = [
        Point2::new(0.0, 0.0),
        Point2::new(2.0, 0.0),
        Point2::new(2.0, 2.0),
        Point2::new(0.0, 2.0),
    ];
    assert_eq!(poly::polygon_properties(&square), (4.0, 8.0));
    let mut list: List<Handle<Triangulation>> = List::default();
    list.append(Handle::new(mesh));
    list.append(Handle::default());
    assert_eq!(poly::catenate(&list).unwrap().nb_nodes(), 3);
    assert!(poly::catenate(&List::default()).unwrap().is_null());
}
