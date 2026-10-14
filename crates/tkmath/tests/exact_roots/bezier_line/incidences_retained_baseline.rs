// SPDX-License-Identifier: GPL-2.0-only
use super::incidences::request;
use super::*;

// Captured from 38a5edf6633ff3683d536a191f241497f8e3077d before the shared preparation refactor.
// This pins owned behavior; it is not independent numerical acceptance evidence.
#[test]
fn owned_incidence_preparation_baseline_is_unchanged() {
    let controls = [[0., -3.], [0., -1.], [0., 1.], [4., 3.]];
    let mut query = request(&controls);
    query.curve.options.max_degree = 1;
    query.domain_sign.degree_limit = BezierDomainDegreeLimit::Explicit(1);
    let mut actual = String::new();
    for domain in [BezierLineDomain::Support, BezierLineDomain::Segment] {
        query.domain = domain;
        actual.push_str(&format!(
            "{:#?}\n",
            certify_bezier_line_incidences(query, || false)
        ));
    }
    assert_eq!(
        actual,
        include_str!("incidence_assets/owned-preparation-baseline.txt")
    );
}
