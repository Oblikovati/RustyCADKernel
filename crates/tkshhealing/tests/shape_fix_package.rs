//! Package utility results compared with pinned OCCT.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{self, Compound, Edge, Shape},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCircle, GeomCurve},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tkshhealing::shape_fix::ShapeFix;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};

fn expected(name: &str) -> f64 {
    include_str!("../../../tools/oracle/tkshhealing_package.expected")
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(' ')?;
            (key == name).then(|| value.parse().unwrap())
        })
        .unwrap()
}
fn empty_compound() -> Shape {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    compound.into_shape()
}
fn no_curve() -> Shape {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    edge.into_shape()
}
fn line() -> Shape {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(1., 2., 3.), Point3::new(4., 6., 3.))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
fn circle() -> Shape {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 2.).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
#[test]
fn least_edge_uses_native_three_samples_and_empty_sentinel() {
    for (name, shape) in [
        ("empty", empty_compound()),
        ("no_curve", no_curve()),
        ("line", line()),
        ("circle", circle()),
        ("box", box_shape()),
    ] {
        assert_eq!(
            ShapeFix::least_edge_size(&shape).unwrap(),
            expected(name),
            "{name}"
        );
    }
}
#[test]
fn encode_regularity_preserves_native_cylinder_edge_observations() {
    let cylinder = BRepPrimAPIMakeCylinder::new(2., 5.)
        .shape()
        .unwrap()
        .clone();
    ShapeFix::encode_regularity(&cylinder, 1e-10).unwrap();
    let mut edges = Explorer::new(&cylinder, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while edges.more() {
        let edge = topods::edge(edges.current()).unwrap();
        count += i32::from(BRepTool::has_continuity(&edge).unwrap());
        edges.next().unwrap();
    }
    assert_eq!(f64::from(count), expected("regularity"));
}

fn box_shape() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
