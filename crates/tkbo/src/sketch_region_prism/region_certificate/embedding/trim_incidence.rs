// SPDX-License-Identifier: GPL-2.0-only
//! Shared trim correspondence preserves the V1 checks; callers supply bound geometric domains.
use super::super::{
    history::same_physical,
    hypothesis::{HypothesisCheck as RegionCheck, holds},
};
use super::*;

/// Check one supplied seam pair; `seam_conclusion(start,end)` grants no region authority.
pub(in crate::sketch_region_prism) fn seam_conclusion(
    start: &PrismAuthoredTrimUse,
    end: &PrismAuthoredTrimUse,
) -> RegionCheck<PrismIncidenceConclusion> {
    holds(
        "embedding authored seam physical correspondence",
        same_physical(&start.output, &end.output) && same_physical(&start.source, &end.source),
    )?;
    Ok(PrismIncidenceConclusion::InjectiveModuloSeam {
        start: start.clone(),
        end: end.clone(),
        native_endpoints: [0., 1.],
        normalized_sweep: [0., 1.],
    })
}

/// Check one supplied rim pair; `rim_conclusion(cap,side,sweep)` assumes bound chart operands.
pub(in crate::sketch_region_prism) fn rim_conclusion(
    cap_trim: &PrismAuthoredTrimUse,
    side_trim: &PrismAuthoredTrimUse,
    sweep: f64,
) -> RegionCheck<PrismIncidenceConclusion> {
    holds(
        "embedding authored rim physical correspondence",
        same_physical(&cap_trim.output, &side_trim.output)
            && same_physical(&cap_trim.source, &side_trim.source),
    )?;
    Ok(PrismIncidenceConclusion::ExactlyRim {
        cap: cap_trim.clone(),
        side: side_trim.clone(),
        native: [0., 1.],
        normalized_sweep: sweep,
    })
}

/// Retain every supplied directed trim field; `retain_trim(chart,trim)` adds no acceptance.
pub(in crate::sketch_region_prism) fn retain_trim(
    chart: &BoundPrismChart,
    trim: &CapturedPrismTrim,
) -> PrismAuthoredTrimUse {
    PrismAuthoredTrimUse {
        face: chart.output.clone(),
        output: trim.output.clone(),
        source: trim.source.clone(),
        parents: trim.parents.clone(),
        role: trim.role,
        interval: trim.interval,
    }
}
