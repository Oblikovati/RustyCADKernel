//! Native seam ordering and output-reference preservation.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCylindricalSurface, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
    shape_fix::ShapeFixWire,
};
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_seam.expected");

#[test]
fn seam_checks_match_native_independently_of_topological_orientation() {
    inspect("swapped", 2, false, false);
    inspect("ordered", 3, false, false);
    inspect("reversed_face", 2, true, false);
    inspect("reversed_edge", 2, false, true);
    inspect("reversed_both", 3, true, true);
}

#[test]
fn unready_and_ordinary_edges_preserve_output_sentinels() {
    inspect("unloaded", 0, false, false);
    inspect("ordinary", 1, false, false);
}

fn inspect(name: &str, mode: u8, reverse_face: bool, reverse_edge: bool) {
    let analyzer = Handle::new(seam_analyzer(mode, reverse_face, reverse_edge));
    let (mut first, mut second, mut range) = (line(77.), line(88.), (17., 19.));
    let found = analyzer
        .check_seam_into(0, &mut first, &mut second, &mut range)
        .unwrap();
    let mut observed = vec![
        f64::from(found),
        f64::from(analyzer.last_check_status(ShapeExtendStatus::Done1)),
        first.value(0.).unwrap().x(),
        second.value(0.).unwrap().x(),
        range.0,
        range.1,
        f64::from(analyzer.check_seam_at(0).unwrap()),
    ];
    append_seam_repair(&analyzer, mode, &mut observed);
    let row = EXPECTED
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(
        observed.as_slice(),
        expected.as_slice(),
        "native seam case {name}"
    );
}

fn append_seam_repair(analyzer: &Handle<ShapeAnalysisWire>, mode: u8, observed: &mut Vec<f64>) {
    let fixer = ShapeFixWire::new();
    fixer.init(analyzer);
    observed.extend([
        f64::from(fixer.fix_seam(0).unwrap()),
        f64::from(fixer.last_fix_status(ShapeExtendStatus::Done1)),
    ]);
    if mode == 0 {
        observed.extend([77., 88., 17., 19., 0.]);
    } else {
        append_repaired_pcurves(analyzer, observed);
    }
    observed.extend([
        f64::from(fixer.fix_seam(0).unwrap()),
        f64::from(fixer.last_fix_status(ShapeExtendStatus::Ok)),
    ]);
}

fn append_repaired_pcurves(analyzer: &ShapeAnalysisWire, observed: &mut Vec<f64>) {
    let mut face = analyzer.face();
    let mut edge = analyzer.wire_data().edge(1).unwrap();
    face.set_orientation(Orientation::Forward);
    edge.set_orientation(Orientation::Forward);
    let forward = BRepTool::curve_on_face(&edge, &face).unwrap();
    observed.push(forward.pcurve.value(0.).unwrap().x());
    edge.set_orientation(Orientation::Reversed);
    let reverse = BRepTool::curve_on_face(&edge, &face).unwrap();
    observed.extend([
        reverse.pcurve.value(0.).unwrap().x(),
        reverse.first,
        reverse.last,
        BRepTool::edge_tolerance(&edge).unwrap(),
    ]);
}

fn seam_analyzer(mode: u8, reverse_face: bool, reverse_edge: bool) -> ShapeAnalysisWire {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let mut edge = seam_edge(mode, &face);
    if reverse_edge {
        edge.set_orientation(Orientation::Reversed);
    }
    if reverse_face {
        face.set_orientation(Orientation::Reversed);
    }
    let wire = Handle::new(ShapeExtendWireData::new());
    if mode != 0 {
        wire.add_edge(&edge, 0).unwrap();
    }
    ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap()
}

fn seam_edge(mode: u8, face: &Face) -> Edge {
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    if mode == 1 {
        builder
            .update_edge_pcurve_on_face(&edge, &line(0.), face, 1e-7)
            .unwrap();
    }
    if mode >= 2 {
        let (first, second) = if mode == 2 { (0., 6.) } else { (6., 0.) };
        builder
            .update_edge_pcurves_on_face(&edge, (&line(first), &line(second)), face, 1e-7)
            .unwrap();
    }
    if mode != 0 {
        builder.range_on_face(&edge, face, 2., 4.).unwrap();
    }
    edge
}

fn line(x: f64) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(x, 0.),
        Direction2::new(0., 1.).unwrap(),
    )))
    .cast()
}
