// SPDX-License-Identifier: GPL-2.0-only
//! Exact offset pieces (ADR 0080). A basis piece `(x, y) / w` with tangent
//! `V = (x' w - x w', y' w - y w')` and squared speed `G = |V|^2 > 0` has the
//! offset point `(x sqrt(G) + d w V_y, y sqrt(G) - d w V_x) / (w sqrt(G))`, i.e.
//! `C + d (C'_y, -C'_x) / |C'|` exactly, with surd coordinates.
//!
//! An ellipse `C + ra cos(t) a + rb sin(t) n` (unit `a = axis / |axis|`, which is
//! rational when admitted) uses two rational charts `u = tan((t - t0) / 2)` on
//! `[-1, 1]`, around `t0 = 0` and `t0 = pi`; both travel counterclockwise.
use super::super::certificate_arithmetic::ExactRatio;
use super::input::{SketchIncidenceCurve, SketchIncidenceOffset};
use super::offset_geometry::{outward_distance, rational_length, speed_squared, tangent};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::spans::{HomogeneousPoint, ParametricSpan, line_span, spline_spans};
use super::surd::Surd;
use std::sync::Arc;

/// One offset piece with its basis point (whose places and sectors it keeps).
#[derive(Clone, Debug)]
pub(super) struct OffsetSpan {
    pub span: ParametricSpan<Surd>,
    pub basis: HomogeneousPoint,
}
/// Offset pieces of an admitted, canonical, non-circle offset, in basis order.
pub(super) fn offset_spans(offset: &SketchIncidenceOffset) -> Vec<OffsetSpan> {
    let distance = ratio(offset.distance);
    match offset.basis.as_ref() {
        SketchIncidenceCurve::Line { points, bounded } => {
            vec![offset_span(line_span(*points, *bounded), &distance)]
        }
        SketchIncidenceCurve::Spline(spline) => spline_spans(spline)
            .into_iter()
            .map(|span| offset_span(span, &distance))
            .collect(),
        ellipse => ellipse_spans(offset, ellipse),
    }
}
// Both rational charts of an ellipse, offset along the outward normal of its sense.
fn ellipse_spans(
    offset: &SketchIncidenceOffset,
    ellipse: &SketchIncidenceCurve,
) -> Vec<OffsetSpan> {
    let SketchIncidenceCurve::Ellipse {
        center,
        major_axis,
        radii,
        sector,
    } = ellipse
    else {
        unreachable!("offset basis {ellipse:?}; expected a canonical non-circle basis");
    };
    let outward = outward_distance(offset, sector.as_ref());
    let frame = EllipseFrame::new(*center, *major_axis, *radii);
    [false, true]
        .map(|opposite| offset_span(frame.chart(opposite), &outward))
        .to_vec()
}
/// The offset of one basis piece at an exact distance.
pub(super) fn offset_span(basis: ParametricSpan, distance: &ExactRatio) -> OffsetSpan {
    let velocity = tangent(&basis.point);
    let radicand = Arc::new(speed_squared(&velocity));
    let shift = basis.point.w.scale(distance);
    let surd = |rational: RationalPolynomial, radical: &RationalPolynomial| Surd {
        rational,
        radical: radical.clone(),
        radicand: Arc::clone(&radicand),
    };
    let [vx, vy] = &velocity;
    let point = HomogeneousPoint {
        x: surd(shift.multiply(vy), &basis.point.x),
        y: surd(shift.multiply(vx).scale(&ratio(-1.)), &basis.point.y),
        w: surd(RationalPolynomial(Vec::new()), &basis.point.w),
    };
    OffsetSpan {
        span: ParametricSpan {
            point,
            range: basis.range,
        },
        basis: basis.point,
    }
}
/// Center and the scaled axes `ra a`, `rb n` of an ellipse with a rational axis length.
struct EllipseFrame {
    center: [ExactRatio; 2],
    major: [ExactRatio; 2],
    minor: [ExactRatio; 2],
}
impl EllipseFrame {
    fn new(center: [f64; 2], axis: [f64; 2], radii: [f64; 2]) -> Self {
        let length = rational_length(axis).expect("admitted rational axis length");
        let unit = axis.map(|value| ratio(value).divide(&length));
        let [major, minor] = radii.map(ratio);
        Self {
            center: center.map(ratio),
            major: [unit[0].multiply(&major), unit[1].multiply(&major)],
            minor: [unit[1].negated().multiply(&minor), unit[0].multiply(&minor)],
        }
    }
    // `C (1 + u^2) +- (ra a (1 - u^2) + rb n 2u)` over `w = 1 + u^2`, u in [-1, 1].
    fn chart(&self, opposite: bool) -> ParametricSpan {
        let side = ratio(if opposite { -1. } else { 1. });
        let coordinate = |axis: usize| {
            let center = &self.center[axis];
            let major = self.major[axis].multiply(&side);
            let minor = self.minor[axis].multiply(&side);
            RationalPolynomial(vec![
                center.add(&major),
                minor.multiply(&ratio(2.)),
                center.subtract(&major),
            ])
        };
        let w = RationalPolynomial(vec![ratio(1.), ratio(0.), ratio(1.)]);
        ParametricSpan {
            point: HomogeneousPoint {
                x: coordinate(0),
                y: coordinate(1),
                w,
            },
            range: Some([-1., 1.]),
        }
    }
}
