//! Typed replay of compact native buffer and oriented-text records.
use super::color_scale_recording::{ScaleDrawRequest, record_scale_vertices};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
pub(super) const GEOMETRY: &str =
    include_str!("../../tests/fixtures/view-cube-native-geometry.txt");
pub(super) const PRESENTATIONS: &str =
    include_str!("../../tests/fixtures/view-cube-native-presentations.txt");
pub(super) fn fixture_rows<'a>(
    fixture: &'a str,
    kind: &'a str,
) -> impl Iterator<Item = Vec<&'a str>> + 'a {
    fixture
        .lines()
        .filter(move |line| line.split_whitespace().next() == Some(kind))
        .map(|line| line.split_whitespace().skip(1).collect())
}
pub(super) fn parse_cube_array(parts: &[&str]) -> ScaleDrawRequest {
    let count = parts[2].parse::<usize>().unwrap();
    let width = parts[4].parse::<usize>().unwrap();
    let vertex_end = 5 + count * width;
    let vertices = parts[5..vertex_end]
        .chunks(width)
        .map(|vertex| vertex.iter().map(|value| value.parse().unwrap()).collect())
        .collect();
    let indices = parts[vertex_end..]
        .iter()
        .map(|value| value.parse().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), parts[3].parse::<usize>().unwrap());
    ScaleDrawRequest::Array {
        kind: parts[1].parse().unwrap(),
        vertices,
        indices,
    }
}
pub(super) fn parse_cube_text(parts: &[&str]) -> ScaleDrawRequest {
    ScaleDrawRequest::Text3d {
        frame: std::array::from_fn(|i| parts[i + 1].parse().unwrap()),
        height: parts[10].parse().unwrap(),
        horizontal: parts[11].parse().unwrap(),
        vertical: parts[12].parse().unwrap(),
        anchor: parts[13] == "1",
        plane: parts[14] == "1",
        text: parts[15].trim_matches('"').to_owned(),
    }
}
pub(super) fn actual_cube_array(array: &Handle<Graphic3dArrayOfTriangles>) -> ScaleDrawRequest {
    let indices = array.indices();
    ScaleDrawRequest::Array {
        kind: array.primitive_type() as i32,
        vertices: record_scale_vertices(&array.attributes()),
        indices: (0..indices.nb_elements())
            .map(|i| indices.index(i).unwrap())
            .collect(),
    }
}
pub(super) fn assert_cube_request(actual: &ScaleDrawRequest, expected: &ScaleDrawRequest) {
    if let (Some((a, av, ai)), Some((e, ev, ei))) =
        (cube_array_parts(actual), cube_array_parts(expected))
    {
        assert_eq!(a, e);
        assert_eq!(ai, ei);
        assert_cube_vertices(av, ev);
        return;
    }
    if let (
        ScaleDrawRequest::Text3d { frame: af, .. },
        ScaleDrawRequest::Text3d { frame: ef, .. },
    ) = (actual, expected)
    {
        assert_cube_coordinates(af, ef);
        assert_cube_text_metadata(actual, expected);
        return;
    }
    panic!("ViewCube request shape mismatch: {actual:?}; expected {expected:?}");
}
type CubeArrayChannels<'a> = (i32, &'a [Vec<f64>], &'a [i32]);
fn cube_array_parts(request: &ScaleDrawRequest) -> Option<CubeArrayChannels<'_>> {
    if let ScaleDrawRequest::Array {
        kind,
        vertices,
        indices,
    } = request
    {
        return Some((*kind, vertices, indices));
    }
    None
}

fn assert_cube_vertices(actual: &[Vec<f64>], expected: &[Vec<f64>]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_cube_coordinates(actual, expected);
    }
}

pub(super) fn assert_cube_coordinates(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a - b).abs() <= 2e-5,
            "ViewCube channel {index}: actual={a}; expected={b}"
        );
    }
}
fn assert_cube_text_metadata(actual: &ScaleDrawRequest, expected: &ScaleDrawRequest) {
    let mut actual = actual.clone();
    let mut expected = expected.clone();
    if let ScaleDrawRequest::Text3d { frame, .. } = &mut actual {
        *frame = [0.0; 9];
    }
    if let ScaleDrawRequest::Text3d { frame, .. } = &mut expected {
        *frame = [0.0; 9];
    }
    assert_eq!(actual, expected);
}
