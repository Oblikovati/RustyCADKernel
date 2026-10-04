use super::*;
use rustycad_tkbo::{bop_tools, int_tools::Curve};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{ShapeIterator, Vertex},
};
fn check_observation(line: &str, actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len(), "{line}: {actual:?}");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() < 1e-12,
            "{line}: column {index}, actual {actual}"
        );
    }
}
fn parse_row(line: &str) -> Vec<f64> {
    line.split_whitespace()
        .map(|token| token.parse().unwrap())
        .collect()
}
#[test]
fn native_edge_copy_and_split_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_edge_copy_split.expected").lines() {
        let row = parse_row(line);
        let actual = copy_split_observation(row[0] as usize, row[1] as usize, row[2] as usize);
        check_observation(line, &actual, &row[3..]);
    }
}
fn copy_split_observation(kind: usize, mode: usize, orientation: usize) -> Vec<f64> {
    let (_, mut source) = edge_fixture(if kind == 0 { 0 } else { 2 });
    source.set_orientation(
        [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ][orientation],
    );
    let (first, last) = topexp::vertices(&source, false);
    let output = copy_or_split(&source, &first, &last, mode);
    edge_observation(&output, &source, &first, &last)
}
fn copy_or_split(source: &Edge, first: &Vertex, last: &Vertex, mode: usize) -> Edge {
    if mode == 0 {
        return bop_tools::copy_edge(source).unwrap();
    }
    let mut result = Edge::default();
    let empty = Vertex::default();
    let first = if mode >= 3 { &empty } else { first };
    let last = if mode == 4 { &empty } else { last };
    let (start, end) = if mode == 2 { (0.8, 0.2) } else { (0.2, 0.8) };
    bop_tools::make_split_edge(source, first, start, last, end, &mut result).unwrap();
    result
}
fn edge_observation(output: &Edge, source: &Edge, first: &Vertex, last: &Vertex) -> Vec<f64> {
    let (curve, start, end) = BRepTool::curve(output).unwrap();
    let (original, _, _) = BRepTool::curve(source).unwrap();
    let children: Vec<_> = ShapeIterator::new(output, false, true).collect();
    let mut actual = vec![
        f64::from(output.is_same(source)),
        f64::from(curve == original),
        output.orientation() as u8 as f64,
        start,
        end,
        BRepTool::edge_tolerance(output).unwrap(),
        f64::from(BRepTool::same_parameter(output).unwrap()),
        f64::from(BRepTool::same_range(output).unwrap()),
        children.len() as f64,
    ];
    for child in children {
        actual.extend([
            f64::from(child.is_same(first)),
            f64::from(child.is_same(last)),
            child.orientation() as u8 as f64,
        ]);
    }
    actual
}
#[test]
fn native_section_edge_and_endpoint_mutation_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_section_edge.expected").lines() {
        let row = parse_row(line);
        let actual = section_observation(row[0] as usize, row[1] as usize, row[2] != 0.);
        check_observation(line, &actual, &row[3..]);
    }
}
fn section_inputs(kind: usize, mode: usize) -> (Curve, Vertex, Vertex, f64, f64) {
    let fixture = fixture(if kind == 0 { 0 } else { 2 });
    let (start, end) = match mode {
        1 => (1., 0.),
        2 => (0.5, 0.5),
        _ => (0., 1.),
    };
    let (mut first, mut last) = (Vertex::default(), Vertex::default());
    let builder = BRepBuilder::default();
    builder
        .make_vertex_at(&mut first, fixture.curve.value(start).unwrap(), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, fixture.curve.value(end).unwrap(), 1e-7)
        .unwrap();
    let mut curve = Curve::default();
    curve.set_curve(&fixture.curve);
    (curve, first, last, start, end)
}
fn section_observation(kind: usize, mode: usize, make: bool) -> Vec<f64> {
    let (curve, first, last, start, end) = section_inputs(kind, mode);
    let mut output = Edge::default();
    let result = if make {
        bop_tools::make_edge(&curve, &first, start, &last, end, 1e-4, &mut output)
    } else {
        bop_tools::make_section_edge(&curve, &first, start, &last, end, &mut output)
    };
    let status = match result {
        Ok(()) => 0.,
        Err(error) => {
            assert_eq!(error.exception_type(), "StdFail_NotDone");
            -1.
        }
    };
    let mut actual = vec![
        status,
        BRepTool::vertex_tolerance(&first).unwrap(),
        BRepTool::vertex_tolerance(&last).unwrap(),
        f64::from(output.is_null()),
    ];
    if !output.is_null() {
        let (start, end) = BRepTool::range(&output).unwrap();
        actual.extend([
            start,
            end,
            BRepTool::edge_tolerance(&output).unwrap(),
            output.orientation() as u8 as f64,
        ]);
    }
    actual
}
