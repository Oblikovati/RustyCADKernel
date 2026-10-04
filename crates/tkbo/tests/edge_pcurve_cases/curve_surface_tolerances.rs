use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{Face, Vertex},
    toptools::IndexedMapOfShape,
};
use rustycad_tkg3d::{geom::GeomCurve, top_abs::ShapeEnum};
use rustycad_tkmath::gp::{Vector2, Vector3};
#[test]
fn native_curve_surface_tolerances_match() {
    for line in
        include_str!("../../../../tools/oracle/tkbo_curve_surface_tolerances.expected").lines()
    {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let actual = observe(&row);
        assert_eq!(actual.len(), row.len() - 5, "{line}");
        for (index, (actual, expected)) in actual.iter().zip(&row[5..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-10,
                "{line}: column {index}, actual {actual}"
            );
        }
    }
}
fn observe(row: &[f64]) -> Vec<f64> {
    let face = fixture(row[0] as usize).face;
    let (edges, vertices) = mapped_shapes(&face);
    let edge = Edge::unchecked(edges.find_key(1).unwrap().clone());
    perturb(&face, &edge, row[1] as usize);
    let avoided = avoidance(&edges, &vertices, row[2] as usize);
    let maximum = if row[3] == 0. { 1e-4 } else { 0.1 };
    let result = if row[4] == 0. {
        bop_tools::correct_curve_on_surface(&face, &avoided, maximum, false)
    } else {
        bop_tools::correct_tolerances(&face, &avoided, maximum, false)
    };
    let mut actual = vec![
        if result.is_ok() { 0. } else { -1. },
        edges.extent() as f64,
        vertices.extent() as f64,
    ];
    actual.extend((1..=edges.extent()).map(|i| {
        BRepTool::edge_tolerance(&Edge::unchecked(edges.find_key(i).unwrap().clone())).unwrap()
    }));
    actual.extend((1..=vertices.extent()).map(|i| {
        BRepTool::vertex_tolerance(&Vertex::unchecked(vertices.find_key(i).unwrap().clone()))
            .unwrap()
    }));
    actual
}
fn mapped_shapes(face: &Face) -> (IndexedMapOfShape, IndexedMapOfShape) {
    let (mut edges, mut vertices) = (IndexedMapOfShape::default(), IndexedMapOfShape::default());
    topexp::map_shapes_of_type(face, ShapeEnum::Edge, &mut edges).unwrap();
    topexp::map_shapes_of_type(face, ShapeEnum::Vertex, &mut vertices).unwrap();
    (edges, vertices)
}
fn perturb(face: &Face, edge: &Edge, mode: usize) {
    let builder = BRepBuilder::default();
    if mode == 1 {
        let pc: Handle<dyn Geom2dCurve> = BRepTool::curve_on_face(edge, face)
            .unwrap()
            .pcurve
            .copy()
            .unwrap()
            .cast();
        pc.translate(Vector2::new(0.01, 0.02)).unwrap();
        builder
            .update_edge_pcurve_on_face(edge, &pc, face, 1e-7)
            .unwrap();
    }
    if mode == 2 {
        shift_curve(edge);
    }
    if mode >= 3 {
        builder.same_range(edge, false).unwrap();
    }
    if mode == 4 {
        builder.same_parameter(edge, false).unwrap();
    }
}
fn shift_curve(edge: &Edge) {
    let (curve, _, _) = BRepTool::curve(edge).unwrap();
    if curve.is_null() {
        return;
    }
    let curve: Handle<dyn GeomCurve> = curve.copy().unwrap().cast();
    curve.translate(Vector3::new(0., 0., 0.02)).unwrap();
    BRepBuilder::default()
        .update_edge_curve(edge, &curve, 1e-7)
        .unwrap();
}
fn avoidance(
    edges: &IndexedMapOfShape,
    vertices: &IndexedMapOfShape,
    mode: usize,
) -> IndexedMapOfShape {
    let mut avoided = IndexedMapOfShape::default();
    if mode == 1 {
        avoided.add(edges.find_key(1).unwrap().clone());
    }
    if mode >= 2 {
        for i in 1..=vertices.extent() {
            avoided.add(vertices.find_key(i).unwrap().clone());
        }
    }
    if mode == 3 {
        for i in 1..=edges.extent() {
            avoided.add(edges.find_key(i).unwrap().clone());
        }
    }
    avoided
}
