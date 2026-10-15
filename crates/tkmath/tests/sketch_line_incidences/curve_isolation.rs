// SPDX-License-Identifier: GPL-2.0-only
//! Thread independence, input ownership and repeatable output for curve sources.
use super::isolation::digest;
use super::*;

fn mixed_targets() -> Vec<SketchIncidenceTarget> {
    vec![
        target(1, segment([-3., 0.1], [3., 0.3])),
        target(2, circle([0.75, 0.25], 1.25)),
        target(3, ellipse([-0.25, -0.5], [2., 1.], [1.75, 0.75])),
        target(
            4,
            spline(
                3,
                &[[-2., -1.], [-1., 2.], [0., -2.], [1., 2.], [2., -1.]],
                &[0., 0., 0., 0., 0.5, 1., 1., 1., 1.],
                [0., 1.],
            ),
        ),
    ]
}
fn sources() -> Vec<SketchIncidenceSource> {
    let arc = arc([0., 0.], 1.5, [1., 0.25], [-0.5, 1.], true);
    let ellipse_source = ellipse([0.1, 0.2], [1., 0.5], [2., 1.]);
    let curve = spline(
        3,
        &[
            [-2.5, 0.5],
            [-1., -1.5],
            [0.5, 1.5],
            [1.5, -1.],
            [2.5, 0.75],
        ],
        &[0., 0., 0., 0., 0.5, 1., 1., 1., 1.],
        [0., 1.],
    );
    vec![
        curve_source(arc, SketchLineDomain::Support),
        curve_source(ellipse_source, SketchLineDomain::Segment),
        curve_source(curve, SketchLineDomain::Segment),
    ]
}
#[test]
fn curve_source_reports_are_identical_across_threads() {
    for source in sources() {
        let expected = certify(source.clone(), &mixed_targets());
        assert!(expected.ordered().len() >= 4, "{expected:#?}");
        let reports: Vec<SketchLineIncidenceReport> = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..8)
                .map(|_| {
                    let source = source.clone();
                    scope.spawn(move || certify(source, &mixed_targets()))
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
fn curve_source_report_retains_its_inputs_after_the_caller_mutates_them() {
    let mut targets = mixed_targets();
    let mut source = sources().remove(0);
    let report = certify(source.clone(), &targets);
    let before = digest(&report);
    targets[0].curve = circle([9., 9.], 2.);
    source.curve = circle([5., 5.], 1.);
    assert_eq!(digest(&report), before);
    assert_eq!(report.source(), &sources()[0]);
}
#[test]
fn print_curve_report_digest_for_process_comparison() {
    // Compared across two separate test processes in the ADR 0067 receipt.
    let text: String = sources()
        .into_iter()
        .map(|source| digest(&certify(source, &mixed_targets())))
        .collect();
    println!("sketch-curve-incidence-digest {}", text.len());
    println!("sketch-curve-incidence-report {text}");
}
