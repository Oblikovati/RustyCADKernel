//! Pinned protected `ShapeFix_ComposeShell::BreakWires` behavior.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_fix::{ShapeFixComposeShell, ShapeFixWireSegment, WirePatchIndex};

#[test]
fn external_joint_vertices_partition_closed_wire_and_keep_patch_indices() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    let vertices = [
        make_vertex(0.0, 0.0),
        make_vertex(2.0, 0.0),
        make_vertex(2.0, 2.0),
        make_vertex(0.0, 2.0),
    ];
    let boundary = (0..4)
        .map(|index| edge_from(&vertices[index], &vertices[(index + 1) % 4]))
        .collect::<Vec<_>>();
    let mut segments = wire_segments(&boundary, &vertices);
    ShapeFixComposeShell::new()
        .break_wires(&mut segments)
        .unwrap();
    assert_eq!(
        format_segments(&segments),
        expected
            .lines()
            .find(|line| line.starts_with("break_wires "))
            .unwrap()
    );
}

#[test]
fn internal_wire_external_edge_becomes_external_segment() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    let vertices = [
        make_vertex(0.0, 0.0),
        make_vertex(1.0, 0.0),
        make_vertex(2.0, 0.0),
    ];
    let mut external_edge = edge_from(&vertices[0], &vertices[1]);
    external_edge.set_orientation(Orientation::External);
    let ordinary_edge = edge_from(&vertices[1], &vertices[2]);
    let mut segment = ShapeFixWireSegment::new();
    segment.add_edge(0, &external_edge).unwrap();
    segment.add_edge(0, &ordinary_edge).unwrap();
    segment.set_orientation(Orientation::Internal);
    let mut segments = Sequence::default();
    segments.append(segment);
    ShapeFixComposeShell::new()
        .break_wires(&mut segments)
        .unwrap();
    assert_eq!(
        format_orientations(&segments),
        expected
            .lines()
            .find(|line| line.starts_with("break_wires_internal_external "))
            .unwrap()
    );
    assert_eq!(
        segments.value(1).unwrap().edge(1).unwrap().orientation(),
        Orientation::Forward
    );
    assert_eq!(
        segments.value(2).unwrap().edge(1).unwrap().orientation(),
        Orientation::Forward
    );
}

fn format_orientations(segments: &Sequence<ShapeFixWireSegment>) -> String {
    let mut output = format!("break_wires_internal_external {}", segments.len());
    for segment in segments.iter() {
        output.push_str(&format!(
            " {}:{}",
            segment.nb_edges(),
            orientation_code(segment.orientation())
        ));
    }
    output
}

fn wire_segments(boundary: &[Edge], vertices: &[Vertex; 4]) -> Sequence<ShapeFixWireSegment> {
    let outer = ShapeFixWireSegment::new();
    for (index, edge) in boundary.iter().enumerate() {
        outer
            .add_edge_with_patch(
                0,
                edge,
                WirePatchIndex {
                    u_min: index as i32 + 3,
                    ..WirePatchIndex::default()
                },
            )
            .unwrap();
    }
    let diagonal = edge_from(&vertices[1], &vertices[3]);
    let mut diagonal = diagonal;
    diagonal.set_orientation(Orientation::External);
    let mut cutter = ShapeFixWireSegment::new();
    cutter.set_orientation(Orientation::External);
    cutter.add_edge(0, &diagonal).unwrap();
    let mut segments = Sequence::default();
    segments.append(outer);
    segments.append(cutter);
    segments
}

fn format_segments(segments: &Sequence<ShapeFixWireSegment>) -> String {
    let mut output = format!("break_wires {}", segments.len());
    for segment in segments.iter() {
        output.push_str(&format!(
            " {}:{}:",
            segment.nb_edges(),
            orientation_code(segment.orientation())
        ));
        for edge_index in 1..=segment.nb_edges() {
            if edge_index > 1 {
                output.push(',');
            }
            output.push_str(&segment.patch_index(edge_index).unwrap().u_min.to_string());
        }
    }
    output
}

fn orientation_code(orientation: Orientation) -> i32 {
    match orientation {
        Orientation::Forward => 0,
        Orientation::Reversed => 1,
        Orientation::Internal => 2,
        Orientation::External => 3,
    }
}

fn make_vertex(x: f64, y: f64) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, y, 0.0), 1.0e-7)
        .unwrap();
    vertex
}

fn edge_from(first: &Vertex, last: &Vertex) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let mut shape = edge.into_shape();
    rustycad_tkbrep::topods::Builder
        .add(&mut shape, &first.oriented(Orientation::Forward))
        .unwrap();
    rustycad_tkbrep::topods::Builder
        .add(&mut shape, &last.oriented(Orientation::Reversed))
        .unwrap();
    topods::edge(&shape).unwrap()
}
