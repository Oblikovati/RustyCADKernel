//! The ChFi3d_Builder_0 topology searches on a box, a cylinder and a lone face, and the
//! smoothness of guide curves.
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi_ds::Map;
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::gp::{Axis2, Circle3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};

fn map_of(s: &Shape, sub: ShapeEnum, ancestor: ShapeEnum) -> Map {
    let mut m = Map::new();
    m.fill(s, sub, ancestor).unwrap();
    m
}

fn first_of<T>(s: &Shape, kind: ShapeEnum, cast: impl Fn(&Shape) -> T) -> T {
    let ex = Explorer::new(s, kind, ShapeEnum::Shape).unwrap();
    cast(ex.current())
}

fn faces_of(s: &Shape) -> Vec<Face> {
    let mut out = Vec::new();
    let mut ex = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while ex.more() {
        out.push(topods::face(ex.current()).unwrap());
        ex.next().unwrap();
    }
    out
}

#[test]
fn a_box_corner_has_three_sharp_edges_on_three_faces() {
    let s = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let (ve, vf, ef) = (
        map_of(&s, ShapeEnum::Vertex, ShapeEnum::Edge),
        map_of(&s, ShapeEnum::Vertex, ShapeEnum::Face),
        map_of(&s, ShapeEnum::Edge, ShapeEnum::Face),
    );
    let v: Vertex = first_of(&s, ShapeEnum::Vertex, |x| topods::vertex(x).unwrap());
    assert_eq!(b0::nbface(vf.find_from_key(&v).unwrap()), 3);
    assert_eq!(
        b0::nb_not_degenerated_edges(&v, &ve).unwrap(),
        6,
        "each edge counts once per face"
    );
    assert_eq!(b0::number_of_edges(&v, &ve).unwrap(), 3);
    assert_eq!(b0::number_of_sharp_edges(&v, &ve, &ef).unwrap(), 3);
    assert!(!b0::cherche_bords_libres(&ve, &v).unwrap().0);
    let edges: Vec<Edge> = ve
        .find_from_key(&v)
        .unwrap()
        .iter()
        .map(|x| topods::edge(x).unwrap())
        .collect();
    let other = edges.iter().find(|e| !e.is_same(&edges[0])).unwrap();
    assert!(
        (b0::angle_edge(&v, &edges[0], other).unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-12
    );
    assert!(
        b0::cherche_vertex(&edges[0], other)
            .unwrap()
            .unwrap()
            .is_same(&v)
    );
    assert!(
        faces_of(&s)
            .iter()
            .all(|f| b0::couture(f).unwrap().is_none())
    );
}

#[test]
fn the_lateral_face_of_a_cylinder_has_a_seam() {
    let s = BRepPrimAPIMakeCylinder::new(2.0, 5.0)
        .shape()
        .unwrap()
        .clone();
    let lateral = faces_of(&s)
        .into_iter()
        .find(|f| b0::couture(f).unwrap().is_some())
        .expect("a seam");
    let seam = b0::couture(&lateral).unwrap().unwrap();
    let (v1, _) = topexp::vertices(&seam, false);
    assert!(
        b0::couture_on_vertex(&lateral, &v1)
            .unwrap()
            .unwrap()
            .is_same(&seam)
    );
    assert!(
        !b0::is_pseudo_seam(&seam, &lateral).unwrap(),
        "no other closed edge touches it"
    );
    let (mut f, faces) = (
        Face::default(),
        map_of(&s, ShapeEnum::Edge, ShapeEnum::Face),
    );
    assert!(
        b0::cherche_face1(faces.find_from_key(&seam).unwrap(), &lateral, &mut f).is_err(),
        "both sides of the seam are the same face"
    );
}

#[test]
fn a_lone_face_has_free_borders_at_every_vertex() {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)] {
        poly.add_point(Point3::new(x, y, 0.0)).unwrap();
    }
    poly.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&poly.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap();
    let s = face.clone().into_shape();
    let ve = map_of(&s, ShapeEnum::Vertex, ShapeEnum::Edge);
    let v: Vertex = first_of(&s, ShapeEnum::Vertex, |x| topods::vertex(x).unwrap());
    let (free, e1, e2) = b0::cherche_bords_libres(&ve, &v).unwrap();
    assert!(free && !e1.is_null() && !e2.is_null() && !e1.is_same(&e2));
    assert_eq!(b0::number_of_edges(&v, &ve).unwrap(), 2);
    let (mut e, mut vtx) = (Edge::default(), Vertex::default());
    b0::cherche_element(&v, &e1, &face, &mut e, &mut vtx).unwrap();
    assert!(e.is_same(&e2) && !vtx.is_same(&v));
}

#[test]
fn a_circle_is_smooth_and_approximated_in_c2() {
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(
        Circle3::new(Axis2::default(), 3.0).unwrap(),
    ))
    .cast();
    assert!(b0::is_smooth(&circle).unwrap());
    let bs = b0::approx_by_c2(&circle).unwrap();
    let bs = bs.get().unwrap();
    assert!(bs.degree() >= 3 && bs.degree() <= 8);
    let mid = bs
        .value(0.5 * (bs.first_parameter() + bs.last_parameter()))
        .unwrap();
    assert!((mid.distance(&Point3::default()) - 3.0).abs() < 1.1e-3);
}
