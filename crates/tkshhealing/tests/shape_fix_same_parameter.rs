//! Native package same-parameter traversal, flags, warning and cancellation rows.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{self, Compound, Edge, Shape},
};
use rustycad_tkernel::{
    message::{ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope},
    standard::Handle,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tkshhealing::{shape_extend::ShapeExtendMsgRegistrator, shape_fix::ShapeFix};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
struct CancelledProgress;
impl ProgressIndicatorHooks for CancelledProgress {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        true
    }
}
fn fixture(name: &str) -> Shape {
    match name {
        "displaced" => displaced_edge_face(),
        "box" => box_shape(),
        "cylinder" => cylinder_shape(),
        "empty" | "empty_cancel" => empty_compound(),
        "no_curve" => no_curve(),
        _ => straight_edge(0.).into_shape(),
    }
}
fn box_shape() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
fn cylinder_shape() -> Shape {
    BRepPrimAPIMakeCylinder::new(2., 5.)
        .shape()
        .unwrap()
        .clone()
}
fn straight_edge(height: f64) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., height), Point3::new(1., 0., height))
        .unwrap()
        .edge()
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
fn observe(name: &str) -> Vec<f64> {
    let shape = fixture(name);
    let progress = ProgressIndicator::new(Box::new(CancelledProgress));
    let range = if name.ends_with("cancel") {
        progress.start()
    } else {
        ProgressRange::default()
    };
    let messages = Handle::new(ShapeExtendMsgRegistrator::new());
    let enforce = !matches!(name, "empty" | "empty_cancel" | "line" | "displaced");
    let changed =
        ShapeFix::same_parameter(&shape, enforce, 0.125, &range, &messages.cast()).unwrap();
    let [same, ranges, tolerance] = edge_observations(&shape);
    let count = messages.map_shape(|map| map.values().map(|list| list.len()).sum::<usize>());
    vec![f64::from(changed), same, ranges, tolerance, count as f64]
}
fn edge_observations(shape: &Shape) -> [f64; 3] {
    let mut edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut row = [0.; 3];
    while edges.more() {
        let edge = topods::edge(edges.current()).unwrap();
        row[0] += f64::from(BRepTool::same_parameter(&edge).unwrap());
        row[1] += f64::from(BRepTool::same_range(&edge).unwrap());
        row[2] += BRepTool::edge_tolerance(&edge).unwrap();
        edges.next().unwrap();
    }
    row
}
#[test]
fn native_same_parameter_rows_match() {
    for line in include_str!("../../../tools/oracle/tkshhealing_same_parameter.expected").lines() {
        let mut words = line.split_whitespace();
        let name = words.next().unwrap();
        let expected: Vec<f64> = words.map(|word| word.parse().unwrap()).collect();
        let actual = observe(name);
        for (index, (left, right)) in actual.into_iter().zip(expected).enumerate() {
            assert!(
                (left - right).abs() <= 1e-14,
                "{name}[{index}]: {left} != {right}"
            );
        }
    }
}

fn displaced_edge_face() -> Shape {
    use rustycad_tkbrep::topods::{Face, Wire};
    use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
    let builder = BRepBuilder::default();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::new(rustycad_tkmath::gp::Axis3::default())).cast();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let edge = straight_edge(1.);
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    builder.add(&mut face, &wire).unwrap();
    face.into_shape()
}
