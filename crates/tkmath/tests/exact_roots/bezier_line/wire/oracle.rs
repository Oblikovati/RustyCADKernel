// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[path = "../../../fixtures/bezier_wire.rs"]
mod fixture;
#[test]
fn wire_full_location_matches_exact_equation_corpus_and_native_faces() {
    let fixtures = fixture::corpus();
    let receipts: Vec<&str> = include_str!("occt-8.0.0.tsv").lines().collect();
    assert_eq!(fixtures.len(), 48);
    assert_eq!(receipts.len(), 48);
    let mut native_vertex_misses = Vec::new();
    for (fixture, receipt) in fixtures.iter().zip(receipts) {
        let result = fixture.with_query(|query| classify_bezier_wire(query, || false).unwrap());
        assert_eq!(result.location(), &fixture.expected, "{}", fixture.name);
        if check_native(&fixture.name, result.location(), receipt) {
            native_vertex_misses.push(fixture.name.as_str());
        }
        fixture.with_query(|query| check_ray_frames(query, result.location()));
    }
    assert_eq!(native_vertex_misses, ["lens_4", "lens_5", "cubic_2"]);
}
fn check_native(name: &str, location: &BezierWireLocation, receipt: &str) -> bool {
    let fields: Vec<&str> = receipt.split('\t').collect();
    assert_eq!(fields[0], name);
    assert_eq!(fields[2], "valid");
    let kind = match location {
        BezierWireLocation::Outside => "outside",
        BezierWireLocation::Inside { .. } => "inside",
        BezierWireLocation::Boundary { .. } => "boundary",
    };
    if fields[1] == kind {
        return false;
    }
    // ADR 0010 retains native classifier failures at exact curved vertices.
    assert_eq!(
        (fields[1], kind, fields[3]),
        ("inside", "boundary", "vertex"),
        "{name}"
    );
    true
}
fn check_ray_frames(mut query: BezierWireQuery<'_>, expected: &BezierWireLocation) {
    let origin = query.line[0];
    for direction in [[0., 1.], [-1., 0.], [1., 1.], [-2., 1.]] {
        query.line[1] = [origin[0] + direction[0], origin[1] + direction[1]];
        let result = classify_bezier_wire(query, || false).unwrap();
        assert_eq!(result.location(), expected);
    }
}
