// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[test]
fn native_ordinary_rational_contacts_supplement_independent_error_certificates() {
    let fixture = include_str!("incidence_assets/occt-ordinary.tsv");
    assert_eq!(fixture.lines().count(), 2);
    for row in fixture.lines() {
        verify_native_row(row);
    }
}
fn verify_native_row(row: &str) {
    let fields: Vec<&str> = row.split('\t').collect();
    let query = native_query(fields[0]);
    let report = certify_bezier_line_incidences(query, || false).unwrap();
    assert_eq!(fields[1], "1");
    assert_eq!(fields[3], "0");
    assert_eq!(report.contacts().len(), fields[2].parse::<usize>().unwrap());
    for (contact, native) in report.contacts().iter().zip(&fields[4..]) {
        verify_native_point(contact, native);
    }
}
fn native_query(name: &str) -> BezierIncidenceQuery<'static> {
    let domain = native_domain(name);
    let curve = BezierLineQuery {
        controls: &[[0., 1.], [0.5, -1.], [1., 1.]],
        weights: Some(&[1., 2., 1.]),
        line: [[0., 0.], [0.5, 0.]],
        interval: [0., 1.],
        options: RootIsolationOptions::new(0.001),
    };
    BezierIncidenceQuery {
        curve_id: 41,
        line_id: 53,
        curve,
        domain,
        domain_sign: BezierDomainSignOptions::source_basis(),
        max_world_error: 1e-8,
    }
}
fn verify_native_point(contact: &CertifiedBezierIncidence, native: &str) {
    let values: Vec<f64> = native
        .split(',')
        .map(|value| value.parse().unwrap())
        .collect();
    let (lower, upper) = contact.parameter().bounds();
    assert!(lower.compare_f64(values[0]).unwrap() != Ordering::Greater);
    assert!(upper.compare_f64(values[0]).unwrap() != Ordering::Less);
    for (axis, value) in contact.world_bounds().iter().zip(&values[1..]) {
        assert!(axis[0] <= *value && *value <= axis[1]);
    }
}

fn native_domain(name: &str) -> BezierLineDomain {
    match name {
        "rational-support" => BezierLineDomain::Support,
        "rational-segment" => BezierLineDomain::Segment,
        value => panic!("fixture {value}; expected ordinary rational row"),
    }
}
