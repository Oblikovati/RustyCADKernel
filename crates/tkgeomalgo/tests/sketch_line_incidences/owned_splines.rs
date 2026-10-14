// SPDX-License-Identifier: GPL-2.0-only
//! Splines from the owned sketch constructor (ADR 0011/0012) through certification.
use super::corpus::{CorpusCase, mapped_line};
use super::native::{assert_covers, certify_case};
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkgeomalgo::sketch_spline::*;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

const LINES: [[[f64; 2]; 2]; 4] = [
    [[-3., 0.3], [4., 0.3]],
    [[1.1, -3.], [1.1, 3.]],
    [[-2., -2.5], [3., 2.5]],
    [[-3., -1.], [4., -0.25]],
];

fn points() -> Vec<Point2> {
    [[0.0, 0.0], [0.2, 1.0], [2.0, 1.5], [3.0, -0.5], [1.0, -2.0]]
        .into_iter()
        .map(|[x, y]| Point2::new(x, y))
        .collect()
}
fn options() -> SketchSplineOptionsV1 {
    SketchSplineOptionsV1 {
        minimum_pivot: 1e-12,
        maximum_fit_residual: 1e-10,
        maximum_seam_residuals: [1e-10, 1e-9, 1e-8],
    }
}
fn handled(closed: bool) -> SketchSplineDefinition {
    let handles = [SketchSplineHandle {
        fit_index: 1,
        end: Point2::new(1.0, 2.5),
    }];
    let limits = SketchHandleOptionsV1 {
        natural: options(),
        maximum_position_residual: 1e-10,
        maximum_derivative_residual: 1e-9,
    };
    build_handled_spline(
        &points(),
        SketchFitSpacing::Centripetal,
        closed,
        &handles,
        limits,
    )
    .unwrap()
    .definition
}
fn definitions() -> Vec<(String, SketchSplineDefinition)> {
    let mut result = Vec::new();
    let spacings = [
        SketchFitSpacing::Centripetal,
        SketchFitSpacing::ChordLength,
        SketchFitSpacing::Uniform,
    ];
    let intents = spacings
        .into_iter()
        .map(SketchSplineIntent::Fit)
        .chain([SketchSplineIntent::Control]);
    for intent in intents {
        for closed in [false, true] {
            let report = build_unhandled_spline(&points(), intent, closed, options()).unwrap();
            result.push((format!("{intent:?} closed={closed}"), report.definition));
        }
    }
    for closed in [false, true] {
        result.push((format!("handled closed={closed}"), handled(closed)));
    }
    result
}
fn incidence_curve(definition: &SketchSplineDefinition) -> SketchIncidenceCurve {
    SketchIncidenceCurve::Spline(definition.incidence_spline())
}
#[test]
fn incidence_spline_copies_the_recipe_bit_for_bit() {
    for (_, definition) in definitions() {
        let spline = definition.incidence_spline();
        let degree = definition.degree() as usize;
        assert_eq!(spline.degree, degree);
        assert_eq!(spline.flat_knots, definition.flat_knots());
        let poles: Vec<[f64; 2]> = definition
            .poles()
            .iter()
            .map(|pole| [pole.x(), pole.y()])
            .collect();
        assert_eq!(spline.poles, poles);
        assert_eq!(spline.weights, None);
        let knots = definition.flat_knots();
        assert_eq!(spline.bounds, [knots[degree], knots[poles.len()]]);
    }
}
#[test]
fn owned_sketch_splines_match_the_mapped_intersector_on_every_line() {
    let mut contacts = 0;
    for (index, (name, definition)) in definitions().into_iter().enumerate() {
        let native = Handle::new(definition.curve().unwrap()).cast();
        for (line_index, line) in LINES.iter().enumerate() {
            let case = CorpusCase {
                id: index * 10 + line_index,
                kind: name.clone(),
                source: *line,
                target: incidence_curve(&definition),
            };
            let mapped = InterCurveCurve::between(&mapped_line(*line), &native, 1e-9).unwrap();
            let points: Vec<[f64; 2]> = (1..=mapped.nb_points())
                .map(|point| {
                    let point = mapped.point(point).unwrap();
                    [point.x(), point.y()]
                })
                .collect();
            let report = certify_case(&case);
            assert_covers(&case, &report, &points);
            contacts += points.len();
        }
    }
    assert!(contacts > 60, "the corpus exercises {contacts} contacts");
}
#[test]
fn handled_spline_triple_knots_are_admitted_as_continuous() {
    let definition = handled(false);
    let interior = definition
        .flat_knots()
        .windows(3)
        .filter(|run| run[0] == run[2])
        .count();
    assert!(
        interior > 2,
        "Hermite construction has triple interior knots"
    );
    let case = CorpusCase {
        id: 0,
        kind: "handled".into(),
        source: LINES[0],
        target: incidence_curve(&definition),
    };
    assert!(!certify_case(&case).targets()[0].contacts().is_empty());
}
