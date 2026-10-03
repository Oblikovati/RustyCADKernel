//! Native aggregate intersection and complete analyzer call-order statuses.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Face, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::{shape_analysis::ShapeAnalysisWire, shape_extend::ShapeExtendStatus};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_perform.expected");

#[test]
fn aggregate_and_perform_match_native_wire_statuses() {
    for (mode, name) in ["square", "bowtie", "reversed", "empty", "open"]
        .into_iter()
        .enumerate()
    {
        let analyzer =
            ShapeAnalysisWire::from_wire(&fixture_wire(mode), &planar_face(), 1e-7).unwrap();
        let observed = observations(&analyzer);
        let row = EXPECTED
            .lines()
            .find(|row| row.starts_with(&format!("{name} ")))
            .unwrap();
        let expected: Vec<u8> = row
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(observed, expected, "native aggregate case {name}");
    }
}

fn observations(analyzer: &ShapeAnalysisWire) -> Vec<u8> {
    use ShapeExtendStatus::*;
    let mut observed = vec![u8::from(analyzer.check_self_intersection().unwrap())];
    for flag in [Done1, Done2, Done3, Fail1, Fail2, Fail3] {
        observed.push(u8::from(analyzer.status_self_intersection(flag)));
    }
    observed.extend([
        u8::from(analyzer.last_check_status(Done)),
        u8::from(analyzer.last_check_status(Fail)),
        u8::from(analyzer.perform().unwrap()),
    ]);
    for flag in [Done, Fail] {
        append_perform_statuses(analyzer, flag, &mut observed);
    }
    analyzer.clear_statuses();
    observed.push(u8::from(analyzer.status_self_intersection(Ok)));
    observed
}

fn append_perform_statuses(
    analyzer: &ShapeAnalysisWire,
    flag: ShapeExtendStatus,
    observed: &mut Vec<u8>,
) {
    observed.extend(
        [
            analyzer.status_order(flag),
            analyzer.status_small(flag),
            analyzer.status_connected(flag),
            analyzer.status_edge_curves(flag),
            analyzer.status_degenerated(flag),
            analyzer.status_self_intersection(flag),
            analyzer.status_lacking(flag),
            analyzer.status_closed(flag),
            analyzer.last_check_status(flag),
        ]
        .map(u8::from),
    );
}

fn planar_face() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn fixture_wire(mode: usize) -> Wire {
    if mode == 3 {
        let mut wire = Wire::default();
        BRepBuilder::default().make_wire(&mut wire);
        return wire;
    }
    let points = if mode == 1 {
        [(0., 0.), (2., 2.), (0., 2.), (2., 0.)]
    } else {
        [(0., 0.), (2., 0.), (2., 2.), (0., 2.)]
    };
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in points {
        polygon.add_point(Point3::new(x, y, 0.)).unwrap();
    }
    if mode != 4 {
        polygon.close().unwrap();
    }
    let mut wire = polygon.wire().unwrap();
    if mode == 2 {
        wire.reverse();
    }
    wire
}
