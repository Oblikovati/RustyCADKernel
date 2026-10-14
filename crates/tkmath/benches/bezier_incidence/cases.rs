// SPDX-License-Identifier: GPL-2.0-only
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};

pub(super) struct IncidenceWorkload {
    pub name: String,
    pub expected: usize,
    controls: Vec<[f64; 2]>,
    weights: Option<Vec<f64>>,
    line: [[f64; 2]; 2],
    interval: [f64; 2],
    domain: BezierLineDomain,
}
pub(super) fn accepted_workloads() -> Vec<IncidenceWorkload> {
    include_str!("../../tests/exact_roots/bezier_line/incidence_assets/accepted.tsv")
        .lines()
        .map(parse_workload)
        .collect()
}
fn parse_workload(row: &str) -> IncidenceWorkload {
    let fields: Vec<&str> = row.split('\t').collect();
    let domain = match fields[5] {
        "support" => BezierLineDomain::Support,
        "segment" => BezierLineDomain::Segment,
        value => panic!("domain {value}; expected support or segment"),
    };
    IncidenceWorkload {
        name: fields[0].into(),
        controls: pairs(fields[1]),
        weights: (fields[2] != "-").then(|| numbers(fields[2])),
        line: pairs(fields[3]).try_into().unwrap(),
        interval: numbers(fields[4]).try_into().unwrap(),
        domain,
        expected: fields[6].parse().unwrap(),
    }
}
fn pairs(text: &str) -> Vec<[f64; 2]> {
    text.split(';')
        .map(|point| {
            let pair = numbers(point);
            pair.try_into()
                .expect("fixture point; expected two finite coordinates")
        })
        .collect()
}
fn numbers(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|value| {
            value
                .parse()
                .expect("fixture scalar; expected binary64 number")
        })
        .collect()
}
impl IncidenceWorkload {
    pub(super) fn verify(&self) {
        let report = certify_bezier_line_incidences(self.query(), || false)
            .expect("accepted complete workload");
        assert_eq!(report.completion(), BezierIncidenceCompletion::Complete);
        assert_eq!(report.contacts().len(), self.expected);
        assert!(report.denominator_isolation().roots().is_empty());
        assert!(
            report
                .contacts()
                .iter()
                .all(|contact| contact.achieved_world_error() <= 1e-8)
        );
        std::hint::black_box(report);
    }
    fn query(&self) -> BezierIncidenceQuery<'_> {
        let curve = BezierLineQuery {
            controls: &self.controls,
            weights: self.weights.as_deref(),
            line: self.line,
            interval: self.interval,
            options: RootIsolationOptions::new(0.001),
        };
        BezierIncidenceQuery {
            curve_id: 17,
            line_id: 23,
            curve,
            domain: self.domain,
            domain_sign: BezierDomainSignOptions::source_basis(),
            max_world_error: 1e-8,
        }
    }
}
