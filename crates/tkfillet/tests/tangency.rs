//! Exact native connection classifications.
mod support;
use rustycad_tkfillet::chfi3d::is_tangent_faces;
use support::{fixture_edge, fixture_face};
#[test]
fn native_tangent_face_cases_match() {
    for line in include_str!("../../../tools/oracle/tkfillet_tangency.expected").lines() {
        let fields: Vec<i32> = line
            .split_whitespace()
            .map(|field| field.parse().unwrap())
            .collect();
        assert_eq!(classify_case(&fields), fields[6], "{line}");
    }
}
fn classify_case(fields: &[i32]) -> i32 {
    let mut edge = fixture_edge(fields[0]);
    if fields[4] != 0 {
        edge.reverse();
    }
    let normals = [
        (0., 0., 1.),
        (0., 0., -1.),
        (0., 1., 0.),
        (0., -1., 0.),
        (0., 0., 1.),
    ];
    let first = fixture_face(&edge, (0., 0., 1.), fields[2] != 0, false);
    let second = if fields[1] == 5 {
        first.clone()
    } else {
        fixture_face(
            &edge,
            normals[fields[1] as usize],
            fields[3] != 0,
            fields[1] == 4,
        )
    };
    is_tangent_faces(
        &edge,
        &first,
        &second,
        [
            rustycad_tkmath::geom_abs::Shape::G1,
            rustycad_tkmath::geom_abs::Shape::C2,
            rustycad_tkmath::geom_abs::Shape::G2,
        ][fields[5] as usize],
    )
    .unwrap() as i32
}
