use super::*;
use rustycad_tkbo::{bop_tools, int_tools::Range};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::gp::Point3;
fn range_curve(kind: usize) -> Handle<dyn GeomCurve> {
    if kind < 2 {
        return fixture(if kind == 0 { 0 } else { 2 }).curve;
    }
    let middle = if kind == 2 {
        Point3::default()
    } else {
        Point3::new(0.5, 2., 0.)
    };
    let poles = [Point3::default(), middle, Point3::new(1., 2., 0.)];
    Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}
fn range_edge(kind: usize, tolerance: f64) -> Edge {
    let edge = BRepBuilderAPIMakeEdge::from_curve_params(&range_curve(kind), 0., 1.)
        .unwrap()
        .edge()
        .unwrap();
    BRepBuilder::default()
        .update_edge_tolerance(&edge, tolerance)
        .unwrap();
    edge
}
#[test]
fn native_edge_and_face_range_corrections_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_correct_range.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let tolerance = [1e-7, 0.1, 1.][row[2] as usize];
        let edge = range_edge(row[0] as usize, tolerance);
        let ranges = [(0., 1.), (0., 1e-10), (1., 0.), (0.2, 0.8)];
        let (first, last) = ranges[row[1] as usize];
        let range = Range::new(first, last);
        let mut corrected = Range::new(9., 8.);
        if row[3] == 0. {
            bop_tools::correct_range_edges(
                &edge,
                &range_edge(0, 2. * tolerance),
                &range,
                &mut corrected,
            )
            .unwrap();
        } else {
            let face = fixture(0).face;
            BRepBuilder::default()
                .update_face_tolerance(&face, 3. * tolerance)
                .unwrap();
            bop_tools::correct_range_face(&edge, &face, &range, &mut corrected).unwrap();
        }
        assert!(
            (corrected.first() - row[4]).abs() < 1e-12,
            "{line}: first={}",
            corrected.first()
        );
        assert!(
            (corrected.last() - row[5]).abs() < 1e-12,
            "{line}: last={}",
            corrected.last()
        );
    }
}
