//! ChFi3d::ConcaveSide on the convex edges of a box and the concave edge of an L-shaped
//! prism, ChFi3d_Coefficient, and the ChFi3d_SearchSing root of two crossing lines.
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi3d::{ChFi3dSearchSing, coefficient, concave_side};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tkmath::math::FunctionRoot;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakePrism};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};

/// Every edge between two faces with its faces.
fn edges_with_faces(shape: &Shape) -> Vec<(Edge, Face, Face)> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut map).unwrap();
    (1..=map.extent())
        .filter_map(|i| {
            let faces: Vec<&Shape> = map.find_from_index(i).unwrap().iter().collect();
            let edge = topods::edge(map.find_key(i).unwrap()).unwrap();
            (faces.len() == 2).then(|| {
                (
                    edge,
                    topods::face(faces[0]).unwrap(),
                    topods::face(faces[1]).unwrap(),
                )
            })
        })
        .collect()
}

fn sides(edge: &Edge, f1: &Face, f2: &Face) -> (i32, Orientation, Orientation) {
    let s1 = BRepAdaptorSurface::with_face(f1, true).unwrap();
    let s2 = BRepAdaptorSurface::with_face(f2, true).unwrap();
    concave_side(&s1, &s2, edge).unwrap()
}

/// The L-shaped prism of legs 10 and 8, thickness 2 and height 5.
fn l_prism() -> Shape {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 2.0),
        (2.0, 2.0),
        (2.0, 8.0),
        (0.0, 8.0),
    ] {
        poly.add_point(Point3::new(x, y, 0.0)).unwrap();
    }
    poly.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&poly.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap();
    let mut prism =
        BRepPrimAPIMakePrism::new(&face.into_shape(), Vector3::new(0.0, 0.0, 5.0), false, true)
            .unwrap();
    prism.shape().unwrap().clone()
}

#[test]
fn box_edges_are_convex_on_both_faces() {
    let shape = BRepPrimAPIMakeBox::from_sizes(10.0, 20.0, 30.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let edges = edges_with_faces(&shape);
    assert_eq!(edges.len(), 12);
    for (edge, f1, f2) in &edges {
        let (choice, or1, or2) = sides(edge, f1, f2);
        assert_eq!((or1, or2), (Orientation::Reversed, Orientation::Reversed));
        assert!(choice == 5 || choice == 6, "choice {choice}");
        let (swapped, _, _) = sides(edge, f2, f1);
        assert_eq!(
            choice + swapped,
            11,
            "swapping the faces turns the normals the other way"
        );
    }
}

#[test]
fn the_inner_edge_of_an_l_prism_is_concave() {
    let edges = edges_with_faces(&l_prism());
    let concave: Vec<_> = edges
        .iter()
        .map(|(e, f1, f2)| sides(e, f1, f2))
        .filter(|s| s.1 == Orientation::Forward)
        .collect();
    assert_eq!(concave.len(), 1);
    assert_eq!(
        (concave[0].1, concave[0].2),
        (Orientation::Forward, Orientation::Forward)
    );
    assert!(concave[0].0 == 1 || concave[0].0 == 2);
}

#[test]
fn coefficient_inverts_the_tangent_plane_derivatives() {
    let (du, dv) = coefficient(
        Vector3::new(3.0, 4.0, 7.0),
        Vector3::new(2.0, 0.0, 0.0),
        Vector3::new(1.0, 1.0, 0.0),
    );
    assert_eq!((du, dv), (-0.5, 4.0));
}

#[test]
fn search_sing_finds_where_two_lines_meet() {
    let line = |p: Point3, d: Direction3| -> Handle<dyn GeomCurve> {
        Handle::new(GeomLine::new(Axis1::new(p, d))).cast()
    };
    let c1 = line(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let c2 = line(
        Point3::new(0.0, 2.0, 0.0),
        Direction3::new(1.0, -1.0, 0.0).unwrap(),
    );
    let mut f = ChFi3dSearchSing::new(&c1, &c2);
    let root = FunctionRoot::bounded(&mut f, 0.3, 1e-12, -5.0, 5.0, 100).unwrap();
    assert!(root.is_done());
    // C2(x) - C1(x) = (x / sqrt 2 - x, 2 - x / sqrt 2, 0) is shortest where it is normal to C2' - C1'.
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let expected = 2.0 * s / ((s - 1.0) * (s - 1.0) + s * s);
    assert!(
        (root.root().unwrap() - expected).abs() < 1e-9,
        "root {}",
        root.root().unwrap()
    );
    assert!(f.take_failure().is_none());
}
