// SPDX-License-Identifier: GPL-2.0-only
//! Admission of ellipses, elliptic arcs and clamped B-splines (ADR 0083). Every caller value
//! is checked before geometry is built; each refusal names the curve and what is wrong.
use super::admitted::AdmittedCurve;
use super::decline::DeclineCause as Cause;
use super::ellipse::EllipseCarrier;
use super::plane_math::{length2, signed_sweep, sub2};
use super::profile::{ArcSense, ProfileCurveId, ProfileVertexId};
use super::resolution::Resolution;
use super::spline::AdmittedSpline;

type Admit<T> = Result<T, Cause>;

/// Highest admitted spline degree: the certified integrals take Bézier pieces up to 16.
const MAX_SPLINE_DEGREE: u32 = 16;

/// A caller ellipse: center, major semi-axis vector and minor radius.
pub(crate) struct EllipseInput {
    pub id: ProfileCurveId,
    pub center: [f64; 2],
    pub major_axis: [f64; 2],
    pub minor_radius: f64,
}

/// A checked carrier: finite, both radii at least `weld`, minor at most major.
pub(crate) fn ellipse_carrier(input: &EllipseInput, weld: f64) -> Admit<EllipseCarrier> {
    let id = input.id;
    let values = [input.center, input.major_axis].concat();
    if !(values.iter().all(|v| v.is_finite()) && input.minor_radius.is_finite()) {
        return Err(Cause::NonFiniteCurve { curve: id });
    }
    let carrier = EllipseCarrier::new(input.center, input.major_axis, input.minor_radius);
    let degenerate = |size| Cause::DegenerateCurve {
        curve: id,
        size,
        weld,
    };
    let carrier = carrier.ok_or(degenerate(0.0))?;
    if carrier.minor < weld || carrier.major < weld {
        return Err(degenerate(carrier.minor.min(carrier.major)));
    }
    if carrier.minor > carrier.major {
        let reason = "minor radius above the major radius";
        return Err(Cause::InvalidCurve { curve: id, reason });
    }
    Ok(carrier)
}

/// A full ellipse.
pub(crate) fn admit_ellipse(input: &EllipseInput, resolution: &Resolution) -> Admit<AdmittedCurve> {
    let carrier = ellipse_carrier(input, resolution.weld)?;
    Ok(AdmittedCurve::Ellipse {
        id: input.id,
        carrier,
    })
}

/// An elliptic arc: ends on the carrier within `sew`, distinct, sweeping more than the
/// angular tolerance and longer than the weld tolerance.
pub(crate) fn admit_elliptic_arc(
    input: &EllipseInput,
    ends: [(ProfileVertexId, [f64; 2]); 2],
    sense: ArcSense,
    resolution: &Resolution,
) -> Admit<AdmittedCurve> {
    let carrier = ellipse_carrier(input, resolution.weld)?;
    for (vertex, point) in ends {
        require_on_curve(input.id, vertex, carrier.gap(point), resolution.sew)?;
    }
    let [t0, t1] = ends.map(|(_, point)| carrier.parameter(point));
    let sweep = signed_sweep(t0, t1, sense).abs();
    let size = carrier.minor * sweep.min(1.0);
    let degenerate = ends[0].0 == ends[1].0 || size < resolution.weld;
    if degenerate || sweep < resolution.angular {
        let weld = resolution.weld;
        return Err(Cause::DegenerateCurve {
            curve: input.id,
            size,
            weld,
        });
    }
    let (id, ends) = (input.id, ends.map(|(vertex, _)| vertex));
    Ok(AdmittedCurve::EllipticArc {
        id,
        ends,
        carrier,
        sense,
    })
}

fn require_on_curve(
    curve: ProfileCurveId,
    vertex: ProfileVertexId,
    distance: f64,
    sew: f64,
) -> Admit<()> {
    match distance <= sew {
        true => Ok(()),
        false => Err(Cause::EndOffCurve {
            curve,
            vertex,
            distance,
            sew,
        }),
    }
}

/// A caller spline before admission.
pub(crate) struct SplineInput<'a> {
    pub id: ProfileCurveId,
    pub degree: u32,
    pub poles: &'a [[f64; 2]],
    pub weights: Option<&'a [f64]>,
    pub knots: &'a [f64],
}

/// A clamped spline whose end poles lie within `sew` of its vertices; the admitted end
/// poles are the vertex points.
pub(crate) fn admit_spline(
    input: &SplineInput<'_>,
    ends: [(ProfileVertexId, [f64; 2]); 2],
    resolution: &Resolution,
) -> Admit<AdmittedCurve> {
    require_degree(input)?;
    require_poles(input)?;
    require_knots(input)?;
    let mut poles = input.poles.to_vec();
    let last = poles.len() - 1;
    for ((vertex, point), index) in ends.into_iter().zip([0, last]) {
        let distance = length2(sub2(point, poles[index]));
        require_on_curve(input.id, vertex, distance, resolution.sew)?;
        poles[index] = point;
    }
    let size: f64 = poles.windows(2).map(|w| length2(sub2(w[1], w[0]))).sum();
    if size < resolution.weld {
        let (curve, weld) = (input.id, resolution.weld);
        return Err(Cause::DegenerateCurve { curve, size, weld });
    }
    let spline = AdmittedSpline {
        degree: input.degree as usize,
        poles,
        weights: input.weights.map(<[f64]>::to_vec),
        knots: input.knots.to_vec(),
    };
    let (id, ends) = (input.id, ends.map(|(vertex, _)| vertex));
    Ok(AdmittedCurve::Spline { id, ends, spline })
}

fn invalid(curve: ProfileCurveId, reason: &'static str) -> Admit<()> {
    Err(Cause::InvalidCurve { curve, reason })
}

fn require_degree(input: &SplineInput<'_>) -> Admit<()> {
    match input.degree {
        0 => invalid(input.id, "spline degree 0"),
        d if d > MAX_SPLINE_DEGREE => Err(Cause::UnsupportedCurve {
            curve: input.id,
            kind: "spline above degree 16",
        }),
        _ => Ok(()),
    }
}

fn require_poles(input: &SplineInput<'_>) -> Admit<()> {
    let id = input.id;
    if !input.poles.iter().flatten().all(|v| v.is_finite()) {
        return Err(Cause::NonFiniteCurve { curve: id });
    }
    if input.poles.len() <= input.degree as usize {
        return invalid(id, "fewer than degree + 1 poles");
    }
    let Some(weights) = input.weights else {
        return Ok(());
    };
    if weights.len() != input.poles.len() {
        return invalid(id, "weight count differs from pole count");
    }
    match weights.iter().all(|w| w.is_finite() && *w > 0.0) {
        true => Ok(()),
        false => invalid(id, "nonpositive or nonfinite weight"),
    }
}

fn require_knots(input: &SplineInput<'_>) -> Admit<()> {
    let (id, knots, p) = (input.id, input.knots, input.degree as usize);
    if knots.len() != input.poles.len() + p + 1 {
        return invalid(id, "knot count is not poles + degree + 1");
    }
    if !knots.iter().all(|k| k.is_finite()) {
        return Err(Cause::NonFiniteCurve { curve: id });
    }
    if knots.windows(2).any(|w| w[1] < w[0]) || knots[0] == knots[knots.len() - 1] {
        return invalid(id, "knots decrease or span an empty range");
    }
    let clamped = |end: &[f64]| end.iter().all(|k| *k == end[0]);
    if !(clamped(&knots[..=p]) && clamped(&knots[knots.len() - p - 1..])) {
        let kind = "unclamped spline";
        return Err(Cause::UnsupportedCurve { curve: id, kind });
    }
    // Ends of multiplicity exactly p + 1, interior knots of multiplicity at most p.
    let n = knots.len();
    let ends_exact = knots[p + 1] > knots[p] && knots[n - p - 2] < knots[n - p - 1];
    let any_equal = |w: &[f64]| w.iter().all(|k| *k == w[0]);
    let interior = &knots[p + 1..n - p - 1];
    if !ends_exact || interior.windows(p + 1).any(any_equal) {
        return invalid(id, "knot multiplicity above the degree");
    }
    Ok(())
}
