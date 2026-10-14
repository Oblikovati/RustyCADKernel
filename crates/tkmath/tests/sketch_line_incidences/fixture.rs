// SPDX-License-Identifier: GPL-2.0-only
//! Named builders and checks shared by the analytic incidence fixtures.
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

pub const ERROR: f64 = 1e-12;

pub fn source(points: [[f64; 2]; 2], domain: SketchLineDomain) -> SketchIncidenceSource {
    SketchIncidenceSource {
        id: 1,
        points,
        domain,
    }
}
pub fn target(id: u64, curve: SketchIncidenceCurve) -> SketchIncidenceTarget {
    SketchIncidenceTarget { id, curve }
}
pub fn circle(center: [f64; 2], radius: f64) -> SketchIncidenceCurve {
    SketchIncidenceCurve::Circle {
        center,
        radius,
        sector: None,
    }
}
pub fn arc(
    center: [f64; 2],
    radius: f64,
    start: [f64; 2],
    end: [f64; 2],
    counterclockwise: bool,
) -> SketchIncidenceCurve {
    let sector = SketchConicSector {
        start,
        end,
        counterclockwise,
    };
    SketchIncidenceCurve::Circle {
        center,
        radius,
        sector: Some(sector),
    }
}
pub fn ellipse(center: [f64; 2], major_axis: [f64; 2], radii: [f64; 2]) -> SketchIncidenceCurve {
    SketchIncidenceCurve::Ellipse {
        center,
        major_axis,
        radii,
        sector: None,
    }
}
pub fn segment(start: [f64; 2], end: [f64; 2]) -> SketchIncidenceCurve {
    SketchIncidenceCurve::Line {
        points: [start, end],
        bounded: true,
    }
}
pub fn infinite(start: [f64; 2], end: [f64; 2]) -> SketchIncidenceCurve {
    SketchIncidenceCurve::Line {
        points: [start, end],
        bounded: false,
    }
}
pub fn spline(
    degree: usize,
    poles: &[[f64; 2]],
    flat_knots: &[f64],
    bounds: [f64; 2],
) -> SketchIncidenceCurve {
    SketchIncidenceCurve::Spline(SketchIncidenceSpline {
        degree,
        poles: poles.to_vec(),
        weights: None,
        flat_knots: flat_knots.to_vec(),
        bounds,
    })
}
pub fn certify(
    source: SketchIncidenceSource,
    targets: &[SketchIncidenceTarget],
) -> SketchLineIncidenceReport {
    certify_sketch_line_incidences(source, targets, SketchIncidenceOptions::v1(ERROR), || false)
        .unwrap_or_else(|failure| panic!("{failure}"))
}
/// Contacts of the ordered groups, flattened in order.
pub fn ordered(report: &SketchLineIncidenceReport) -> Vec<&SketchLineContact> {
    report
        .ordered()
        .iter()
        .flat_map(|group| group.members().iter().map(|member| report.contact(*member)))
        .collect()
}
/// The certified point lies within its achieved error of an exactly representable expectation.
pub fn assert_point(contact: &SketchLineContact, expected: [f64; 2]) {
    assert!(contact.achieved_world_error() <= ERROR, "{contact:?}");
    for axis in 0..2 {
        let [lower, upper] = contact.world_bounds()[axis];
        assert!(
            lower <= expected[axis] && expected[axis] <= upper,
            "{contact:?} bounds miss {expected:?}"
        );
        let offset = (contact.point()[axis] - expected[axis]).abs();
        assert!(
            offset <= contact.achieved_world_error(),
            "{contact:?} vs {expected:?}"
        );
    }
}
/// The constructed point is the exact expectation with zero certified error.
pub fn assert_exact_point(contact: &SketchLineContact, expected: [f64; 2]) {
    assert_eq!(contact.point(), expected, "{contact:?}");
    assert_eq!(contact.achieved_world_error(), 0., "{contact:?}");
}
pub fn summary(
    contact: &SketchLineContact,
) -> (
    u64,
    SketchLinePosition,
    SketchLinePosition,
    SketchContactTransition,
    [Option<usize>; 2],
) {
    (
        contact.target_id(),
        contact.line_position(),
        contact.target_position(),
        contact.transition(),
        contact.multiplicity(),
    )
}
