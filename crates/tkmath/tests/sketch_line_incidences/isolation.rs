// SPDX-License-Identifier: GPL-2.0-only
//! Ownership, thread independence and repeatable output of complete reports.
use super::*;

fn mixed_targets() -> Vec<SketchIncidenceTarget> {
    let sector = SketchConicSector {
        start: [1., 0.],
        end: [-1., 1.],
        counterclockwise: true,
    };
    vec![
        target(1, circle([0.25, 0.], 1.)),
        target(
            2,
            SketchIncidenceCurve::Ellipse {
                center: [0.5, 0.2],
                major_axis: [2., 1.],
                radii: [1.5, 0.5],
                sector: Some(sector),
            },
        ),
        target(
            3,
            spline(
                3,
                &[[-2., -1.], [-1., 2.], [0., -2.], [1., 2.], [2., -1.]],
                &[0., 0., 0., 0., 0.5, 1., 1., 1., 1.],
                [0., 1.],
            ),
        ),
        target(4, segment([0.3, -2.], [0.4, 2.])),
    ]
}
fn mixed_source() -> SketchIncidenceSource {
    source([[-3., 0.1], [3., 0.3]], SketchLineDomain::Support)
}
/// Stable text of every certified value; `Debug` output covers the whole report.
pub fn digest(report: &SketchLineIncidenceReport) -> String {
    format!("{report:?}")
}
#[test]
fn reports_are_identical_across_threads() {
    let expected = certify(mixed_source(), &mixed_targets());
    assert!(expected.ordered().len() >= 6, "{expected:?}");
    let reports: Vec<SketchLineIncidenceReport> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| certify(mixed_source(), &mixed_targets())))
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
#[test]
fn report_retains_its_inputs_after_the_caller_mutates_them() {
    let mut targets = mixed_targets();
    let report = certify(mixed_source(), &targets);
    let before = digest(&report);
    targets[0].curve = circle([9., 9.], 2.);
    targets.clear();
    assert_eq!(digest(&report), before);
    assert_eq!(report.targets()[0].target(), &mixed_targets()[0]);
}
#[test]
fn print_report_digest_for_process_comparison() {
    // Compared across two separate test processes in the ADR 0060 receipt.
    let report = certify(mixed_source(), &mixed_targets());
    println!("sketch-incidence-digest {}", digest(&report).len());
    println!("sketch-incidence-report {}", digest(&report));
}
