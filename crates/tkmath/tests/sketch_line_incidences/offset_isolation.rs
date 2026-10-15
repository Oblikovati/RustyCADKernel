// SPDX-License-Identifier: GPL-2.0-only
//! Thread independence, input ownership and repeatable output for offset and
//! periodic-spline incidences (ADR 0080).
use super::isolation::digest;
use super::offset_circles::offset;
use super::*;

fn arch() -> SketchIncidenceCurve {
    spline(
        2,
        &[[0., 0.], [1., 2.], [2., 0.]],
        &[0., 0., 0., 1., 1., 1.],
        [0., 1.],
    )
}
fn periodic_hexagon() -> SketchIncidenceCurve {
    SketchIncidenceCurve::PeriodicSpline(SketchPeriodicSpline {
        degree: 3,
        poles: vec![
            [2., 0.25],
            [1., 2.],
            [-1., 1.75],
            [-2., -0.25],
            [-1., -2.],
            [1., -1.75],
        ],
        weights: None,
        knots: vec![0., 1., 2., 3., 4., 5., 6.],
        multiplicities: vec![1; 7],
        bounds: [0., 6.],
    })
}
fn offset_targets() -> Vec<SketchIncidenceTarget> {
    vec![
        target(1, offset(ellipse([0.25, 0.5], [3., 4.], [2., 1.]), 0.5)),
        target(2, offset(segment([-3., -1.], [3., 1.25]), 0.75)),
        target(3, offset(arch(), -0.25)),
        target(4, offset(circle([0.5, -0.25], 1.25), 0.5)),
        target(5, periodic_hexagon()),
    ]
}
fn offset_sources() -> Vec<SketchIncidenceSource> {
    vec![
        source([[-3., 0.1], [3., 0.4]], SketchLineDomain::Support),
        curve_source(circle([0.1, 0.2], 1.75), SketchLineDomain::Segment),
        curve_source(
            offset(segment([-3., -1.], [3., -0.5]), -1.),
            SketchLineDomain::Segment,
        ),
    ]
}
#[test]
fn offset_reports_are_identical_across_threads() {
    for source in offset_sources() {
        let expected = certify(source.clone(), &offset_targets());
        assert!(expected.ordered().len() >= 6, "{expected:#?}");
        let reports: Vec<SketchLineIncidenceReport> = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..8)
                .map(|_| {
                    let source = source.clone();
                    scope.spawn(move || certify(source, &offset_targets()))
                })
                .collect();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect()
        });
        for report in &reports {
            assert_eq!(report, &expected);
            assert_eq!(digest(report), digest(&expected));
        }
    }
}
#[test]
fn offset_reports_keep_their_inputs_after_the_caller_mutates_them() {
    let mut targets = offset_targets();
    let report = certify(offset_sources().remove(0), &targets);
    let before = digest(&report);
    targets[0].curve = circle([9., 9.], 2.);
    targets.clear();
    assert_eq!(digest(&report), before);
    assert_eq!(report.targets()[0].target(), &offset_targets()[0]);
}
#[test]
fn print_offset_report_digest_for_process_comparison() {
    // Compared across two separate test processes in the ADR 0080 receipt.
    let text: String = offset_sources()
        .into_iter()
        .map(|source| digest(&certify(source, &offset_targets())))
        .collect();
    println!("sketch-offset-incidence-digest {}", text.len());
    println!("sketch-offset-incidence-report {text}");
}
