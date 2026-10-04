use super::super::fixture;
use rustycad_tkbo::bop_tools::algo_tools3d;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Vector2;
#[test]
fn native_split_seam_pair_and_order_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_split_seam.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = observe(&row, false);
        assert_eq!(actual.len(), row.len() - 4);
        for (index, (actual, expected)) in actual.iter().zip(&row[4..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}: {line}; actual={actual}"
            );
        }
    }
}
fn observe(row: &[f64], periodic: bool) -> Vec<f64> {
    let mut face = fixture(row[0] as usize).face;
    let mode = row[1] as usize;
    let (mut origin, ordinary) = original_edges(&face);
    origin.set_orientation(if mode == 1 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    });
    let mut split = split_edge(&origin, &face, mode);
    if mode == 5 {
        origin = ordinary;
    }
    let orientations = [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ];
    split.set_orientation(orientations[row[2] as usize]);
    face.set_orientation(orientations[row[3] as usize]);
    let result = if periodic {
        algo_tools3d::split_seam_on_face(&split, &face)
    } else {
        algo_tools3d::split_seam_from_origin(&origin, &split, &face)
    }
    .unwrap();
    let mut output = vec![
        f64::from(result),
        f64::from(BRepTool::is_closed_on_face(&split, &face).unwrap()),
    ];
    face.set_orientation(Orientation::Forward);
    for side in &orientations[..2] {
        split.set_orientation(*side);
        append_curve(&mut output, &split, &face);
    }
    output
}
pub(super) fn original_edges(face: &Face) -> (Edge, Edge) {
    let mut seam = Edge::default();
    let mut ordinary = Edge::default();
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let edge = topods::edge(&shape).unwrap();
        if BRepTool::is_closed_on_face(&edge, face).unwrap() {
            if seam.is_null() {
                seam = edge;
            }
        } else {
            ordinary = edge;
        }
    }
    (seam, ordinary)
}
pub(super) fn split_edge(origin: &Edge, face: &Face, mode: usize) -> Edge {
    if mode == 4 {
        return topods::edge(&origin.empty_copied().unwrap()).unwrap();
    }
    let (curve, first, last) = BRepTool::curve(origin).unwrap();
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &curve, 1e-7)
        .unwrap();
    builder.range(&edge, first, last, false).unwrap();
    if mode != 3 {
        store_curve(&edge, origin, face, mode);
    }
    edge
}
fn store_curve(edge: &Edge, origin: &Edge, face: &Face, mode: usize) {
    let found = BRepTool::curve_on_face(origin, face).unwrap();
    let curve: Handle<dyn Geom2dCurve> = found.pcurve.copy().unwrap().cast();
    let (mut first, mut last) = (found.first, found.last);
    if mode == 2 {
        curve.translate(Vector2::new(0.1, 0.1)).unwrap();
    }
    if mode == 6 {
        (first, last) = (
            curve.reversed_parameter(last).unwrap(),
            curve.reversed_parameter(first).unwrap(),
        );
        curve.reverse().unwrap();
    }
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(edge, &curve, face, 1e-7)
        .unwrap();
    builder.range_on_face(edge, face, first, last).unwrap();
}
fn append_curve(output: &mut Vec<f64>, edge: &Edge, face: &Face) {
    let curve = BRepTool::curve_on_face(edge, face).unwrap();
    output.extend([f64::from(curve.pcurve.is_null()), curve.first, curve.last]);
    if !curve.pcurve.is_null() {
        let point = curve
            .pcurve
            .value(0.5 * (curve.first + curve.last))
            .unwrap();
        output.extend([point.x(), point.y()]);
    }
}

#[test]
fn native_periodic_split_seam_pair_and_order_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_split_seam_periodic.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = observe(&row, true);
        assert_eq!(actual.len(), row.len() - 4);
        for (index, (actual, expected)) in actual.iter().zip(&row[4..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "column {index}: {line}; actual={actual}"
            );
        }
    }
}
