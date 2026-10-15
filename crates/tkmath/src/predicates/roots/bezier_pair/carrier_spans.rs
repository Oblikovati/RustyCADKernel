// SPDX-License-Identifier: GPL-2.0-only
//! Exact projective-arc intersection for ADR 0067 common-carrier certificates.
//! The second source's interval maps to the arc from its start preimage to its end
//! preimage through its midpoint preimage. Meeting the first interval in one point is a
//! contact; a positive-length meet is an overlap and refuses.
use super::super::certificate_arithmetic::{ExactRatio, rounding::outward_binary64};
use super::carrier::ProjectiveParameter;
use super::operation::PairOperationFailure;
use super::source::OwnedBezierSource;

/// Real span with `None` for an infinite end.
struct Span {
    lower: Option<ExactRatio>,
    upper: Option<ExactRatio>,
}
/// Ordered `(t, u)` contacts; both parameters are exact source interval endpoints.
pub(super) fn touches(
    preimages: &[ProjectiveParameter; 3],
    sources: &[OwnedBezierSource; 2],
) -> Result<Vec<(f64, f64)>, PairOperationFailure> {
    let [start, middle, end] = preimages;
    let spans = arc_spans(start, middle, end)?;
    let first = sources[0].interval.map(ExactRatio::from_finite);
    if first[0] > first[1] {
        return Err(PairOperationFailure::CarrierDegenerate);
    }
    let mut found = Vec::new();
    for span in spans {
        if let Some(t) = meet(&span, &first)? {
            found.push((t.clone(), paired(&t, preimages, sources[1].interval)?));
        }
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));
    found
        .into_iter()
        .map(|(t, u)| Ok((endpoint(&t, sources[0].interval)?, u)))
        .collect()
}
fn arc_spans(
    start: &ProjectiveParameter,
    middle: &ProjectiveParameter,
    end: &ProjectiveParameter,
) -> Result<Vec<Span>, PairOperationFailure> {
    use ProjectiveParameter::{Finite, Infinite};
    Ok(match (start, middle, end) {
        (Finite(p), Finite(m), Finite(q)) if p != q => finite_arc(p, m, q),
        (Finite(p), Infinite, Finite(q)) if p != q => outside(p, q),
        (Infinite, Finite(m), Finite(q)) | (Finite(q), Finite(m), Infinite) => {
            vec![half_line(m, q)]
        }
        _ => return Err(PairOperationFailure::CarrierDegenerate),
    })
}
fn finite_arc(p: &ExactRatio, m: &ExactRatio, q: &ExactRatio) -> Vec<Span> {
    let (low, high) = if p < q { (p, q) } else { (q, p) };
    if low < m && m < high {
        return vec![bounded(low, high)];
    }
    outside(low, high)
}
fn outside(p: &ExactRatio, q: &ExactRatio) -> Vec<Span> {
    let (low, high) = if p < q { (p, q) } else { (q, p) };
    vec![
        Span {
            lower: None,
            upper: Some(low.clone()),
        },
        Span {
            lower: Some(high.clone()),
            upper: None,
        },
    ]
}
/// The arc from infinity to `q` through finite `m` is the half line on `m`'s side.
fn half_line(m: &ExactRatio, q: &ExactRatio) -> Span {
    if m < q {
        return Span {
            lower: None,
            upper: Some(q.clone()),
        };
    }
    Span {
        lower: Some(q.clone()),
        upper: None,
    }
}
fn bounded(low: &ExactRatio, high: &ExactRatio) -> Span {
    Span {
        lower: Some(low.clone()),
        upper: Some(high.clone()),
    }
}
/// Single meeting point, `None` when disjoint; positive length refuses as overlap.
fn meet(span: &Span, first: &[ExactRatio; 2]) -> Result<Option<ExactRatio>, PairOperationFailure> {
    let lower = span
        .lower
        .clone()
        .map_or(first[0].clone(), |v| v.max(first[0].clone()));
    let upper = span
        .upper
        .clone()
        .map_or(first[1].clone(), |v| v.min(first[1].clone()));
    if lower > upper {
        return Ok(None);
    }
    if lower < upper {
        let low = outward_binary64(&lower).map_or(f64::NAN, |b| b[0]);
        let high = outward_binary64(&upper).map_or(f64::NAN, |b| b[1]);
        return Err(PairOperationFailure::CarrierOverlap {
            first_range: [low, high],
        });
    }
    Ok(Some(lower))
}
/// The meeting point is the preimage of the second source's start or end.
fn paired(
    t: &ExactRatio,
    preimages: &[ProjectiveParameter; 3],
    interval: [f64; 2],
) -> Result<f64, PairOperationFailure> {
    if preimages[0] == ProjectiveParameter::Finite(t.clone()) {
        return Ok(interval[0]);
    }
    if preimages[2] == ProjectiveParameter::Finite(t.clone()) {
        return Ok(interval[1]);
    }
    Err(PairOperationFailure::CarrierDegenerate)
}
/// A single meeting point is an endpoint of the first closed interval.
fn endpoint(t: &ExactRatio, interval: [f64; 2]) -> Result<f64, PairOperationFailure> {
    let matched = interval
        .into_iter()
        .find(|end| *t == ExactRatio::from_finite(*end));
    matched.ok_or(PairOperationFailure::CarrierDegenerate)
}
