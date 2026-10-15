// SPDX-License-Identifier: GPL-2.0-only
//! Conic sources against conic targets.
//!
//! Directions from the source center are charted by `d(K) = -K r + o r_perp`
//! (`o det(r, d) = |r|^2 > 0`), so a source point is `C + lambda d(K)` with the
//! upper half turn at `lambda > 0`, and `K` is exactly the sweep cotangent key.
//! On the source, `f = Q_S(d) lambda^2 - c_S = 0`; on the target,
//! `g = Q_T(d) lambda^2 + 2 B(d) lambda + G = 0` with `B(d) = d.M_T (C_S - C_T)`.
//! Their resultant in `lambda`, `R(K) = E^2 - 4 c_S Q_S B^2` with
//! `E = Q_S G + c_S Q_T`, equals `Q_S^2 g(lambda+) g(lambda-)`; its first
//! subresultant gives the single fibre point `lambda = -E / (2 Q_S B)` wherever
//! `B != 0`. `R` changes sign exactly where the source crosses the target.
//! Where `B = 0` both half turns meet the target on one line through the
//! center (an exceptional fibre, `K` rational); those points are certified
//! with `sqrt(c_S / Q_S(d))` as an exact quadratic root. Contacts on the
//! reference line itself (`K = infinity`) are read in the chart of `r_perp`.
//! Concentric conics give an even system handled by `conic_concentric`.
use super::super::certificate_arithmetic::ExactRatio;
use super::budget::Work;
use super::conic_chart::Chart;
use super::conic_concentric;
use super::conic_point::ChartContact;
use super::conic_source::ConicSource;
use super::contact::TargetKey;
use super::failure::{SketchIncidenceCause as Cause, SketchIncidenceStage};
use super::implicit::ConicImplicit;
use super::place::SketchSweepChart;
use super::report::{SketchLineContact, SketchLineOverlap};
use super::sector::ArcBounds;
use super::source_frame::SourceFrame;
use std::cmp::Ordering;

/// One conic target against a conic source.
pub(super) struct ConicPair<'a> {
    pub source: &'a ConicSource,
    pub target: &'a ConicImplicit,
    pub sector: Option<ArcBounds<'a>>,
    pub owner: TargetKey,
}
pub(super) fn incidences(
    pair: &ConicPair<'_>,
    frame: &SourceFrame,
    work: &mut Work<'_>,
) -> Result<(Vec<SketchLineContact>, Vec<SketchLineOverlap>), Cause> {
    work.enter(SketchIncidenceStage::Equation);
    work.check()?;
    if pair.source.implicit.center == pair.target.center {
        return conic_concentric::incidences(pair, frame, work);
    }
    let primary = Chart::primary(&pair.source.frame);
    let mut found = primary.contacts(pair, work)?;
    found.extend(reference_line_contacts(pair, work)?);
    let contacts = found
        .into_iter()
        .map(|contact| contact.finish(pair, frame, work))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((contacts.into_iter().flatten().collect(), Vec::new()))
}
// The reference line `C + mu r` is the point at infinity of the primary chart; in
// the chart of `r_perp` it is `K' = 0`, where its contacts are regular roots.
fn reference_line_contacts(
    pair: &ConicPair<'_>,
    work: &mut Work<'_>,
) -> Result<Vec<ChartContact>, Cause> {
    let secondary = Chart::secondary(&pair.source.frame);
    let resultant = secondary.equations(pair).resultant();
    if resultant.evaluate(&ExactRatio::from_finite(0.)).sign() != Ordering::Equal {
        return Ok(Vec::new());
    }
    secondary.contacts_at_zero(pair, work)
}
/// Chart of the source place of a point at signed distance along `d`.
pub(super) fn half_turn(positive: bool) -> SketchSweepChart {
    if positive {
        SketchSweepChart::Upper
    } else {
        SketchSweepChart::Lower
    }
}
