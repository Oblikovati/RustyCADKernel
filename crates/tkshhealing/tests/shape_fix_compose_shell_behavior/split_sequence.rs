//! Sequence splitting must create interior seams, not merely split input edges.
use super::*;
use rustycad_tkbrep::topods::{Face, Wire};
const EXPECTED: &str = include_str!("../../../../tools/oracle/tkshhealing_split_sequence.expected");

#[test]
fn joint_sequence_matches_native_seams_patches_and_parity() {
    for (mode, row) in EXPECTED.lines().enumerate() {
        let face = sequence_face(mode);
        let context = Handle::new(ShapeBuildReShape::new());
        let composer = initialized_composer(&face, &context);
        composer.set_closed_mode(mode == 4);
        let mut wires = composer.load_wires().unwrap();
        if mode == 5 {
            wires.value(1).unwrap().wire_data().remove(3).unwrap();
        }
        let along_u = mode != 1;
        composer
            .split_segments_by_line(&mut wires, cut_line(mode), along_u, 1)
            .unwrap();
        let observed = sequence_snapshot(&composer, &wires, &face);
        let expected: Vec<f64> = row
            .split_whitespace()
            .skip(1)
            .map(|x| x.parse().unwrap())
            .collect();
        assert_eq!(observed, expected, "native joint sequence {row}");
    }
}

fn sequence_face(mode: usize) -> Face {
    let mut face = face_fixture();
    if mode == 6 {
        BRepBuilder::default()
            .add(&mut face, &upper_rectangle().into_shape())
            .unwrap();
    }
    face
}

fn upper_rectangle() -> Wire {
    let points = [
        Point3::new(0., 3., 0.),
        Point3::new(2., 3., 0.),
        Point3::new(2., 5., 0.),
        Point3::new(0., 5., 0.),
    ];
    let mut builder = BRepBuilderAPIMakeWire::new();
    for index in 0..4 {
        let edge = BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
            .unwrap()
            .edge()
            .unwrap();
        builder.add_edge(&edge).unwrap();
    }
    builder.wire().unwrap()
}

fn cut_line(mode: usize) -> Line2 {
    let position = if mode == 2 {
        3.
    } else if mode == 3 {
        0.
    } else {
        1.
    };
    if mode == 1 {
        Line2::new(Point2::new(0., position), Direction2::new(1., 0.).unwrap())
    } else {
        Line2::new(Point2::new(position, 0.), Direction2::new(0., 1.).unwrap())
    }
}

fn sequence_snapshot(
    composer: &ShapeFixComposeShell,
    wires: &Sequence<ShapeFixWireSegment>,
    face: &Face,
) -> Vec<f64> {
    let mut observed = vec![
        f64::from(wires.length()),
        f64::from(composer.status(ShapeExtendStatus::Fail4)),
    ];
    for index in 1..=wires.len() {
        let wire = wires.value(index).unwrap();
        observed.extend([wire.orientation() as u8 as f64, f64::from(wire.nb_edges())]);
        for edge_index in 1..=wire.nb_edges() {
            append_edge(&mut observed, wire, edge_index, face);
        }
    }
    observed
}

fn append_edge(observed: &mut Vec<f64>, wire: &ShapeFixWireSegment, index: i32, face: &Face) {
    let patch = wire.patch_index(index).unwrap();
    let edge = wire.edge(index).unwrap();
    let found = BRepTool::curve_on_face(&edge, face).unwrap();
    let start = found.pcurve.value(found.first).unwrap();
    let end = found.pcurve.value(found.last).unwrap();
    let spatial = !rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge::new()
        .curve_3d(&edge, false)
        .unwrap()
        .0
        .is_null();
    observed.extend([
        f64::from(patch.u_min),
        f64::from(patch.u_max),
        f64::from(patch.v_min),
        f64::from(patch.v_max),
        found.first,
        found.last,
        start.x(),
        start.y(),
        end.x(),
        end.y(),
        f64::from(spatial),
        f64::from(BRepTool::same_parameter(&edge).unwrap()),
        f64::from(BRepTool::same_range(&edge).unwrap()),
    ]);
}
