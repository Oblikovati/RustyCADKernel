// SPDX-License-Identifier: GPL-2.0-only
//! Every actual face and trim is keyed by its full ancestry, including repeated seam uses.
use super::super::super::{contexts::body_contexts, surface_charts::PrismTrimRole};
use super::super::current_binding::{
    BoundPrismChart, PrismBindingFailure, capture_actual_charts_guarded,
};
use super::*;

pub(super) fn capture(
    receipt: &PrismReceipt,
    copied: &AnnularCopyCapture,
    boundaries: &[AnnularBoundaryBinding],
    cancelled: &mut impl FnMut() -> bool,
) -> AnnularCheck<'static, (Vec<AnnularBoundChart>, Vec<RegionObservedContext>)> {
    capture_guarded(receipt, copied, boundaries, cancelled, &mut || Ok(()))
}
pub(super) fn capture_guarded(
    receipt: &PrismReceipt,
    copied: &AnnularCopyCapture,
    boundaries: &[AnnularBoundaryBinding],
    cancelled: &mut impl FnMut() -> bool,
    preflight: &mut impl FnMut() -> AnnularCheck<'static>,
) -> AnnularCheck<'static, (Vec<AnnularBoundChart>, Vec<RegionObservedContext>)> {
    let actual = capture_native(receipt, cancelled, preflight)?;
    preflight()?;
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .expect("existing actual chart capture checked body");
    let contexts = body_contexts(&diagnostic.body)
        .map_err(|cause| Box::new(AnnularBindingCause::Body(Box::new(cause))))?;
    let snapshots = contexts
        .iter()
        .map(correspondence::snapshot)
        .collect::<AnnularCheck<'_, Vec<_>>>()?;
    let mut charts = Vec::new();
    for chart in actual {
        charts.push(bind_chart(chart, &snapshots, copied, boundaries)?);
    }
    complete_roles(&charts, boundaries)?;
    charts.sort_by_key(|c| (c.identity.role as u8, c.source_boundary));
    Ok((charts, snapshots))
}
fn admit_faces(faces: &[PrismBodyContext]) -> Result<(), PrismBindingFailure> {
    if faces.len() == 4 {
        return Ok(());
    }
    Err(super::super::RegionCertificateCause::Hypothesis {
        name: "two-boundary affine prism actual faces",
        actual: faces.len(),
        expected: 4,
    }
    .into())
}
fn bind_chart(
    chart: BoundPrismChart,
    contexts: &[RegionObservedContext],
    copied: &AnnularCopyCapture,
    boundaries: &[AnnularBoundaryBinding],
) -> AnnularCheck<'static, AnnularBoundChart> {
    let source = source_context(copied, &chart.source)?;
    let outputs: Vec<_> = contexts
        .iter()
        .filter(|c| c.kind == ShapeEnum::Face && c.occurrence == chart.output)
        .collect();
    require("unique actual full chart output", outputs.len(), 1)?;
    let output = outputs[0];
    let source_boundary = chart_boundary(&chart, boundaries, copied)?;
    let trims = bind_trims(&chart, output, contexts, copied)?;
    let identity = AnnularChartIdentity {
        source: source.clone(),
        output: output.clone(),
        role: chart.role,
    };
    Ok(AnnularBoundChart {
        identity,
        source_boundary,
        chart,
        trims,
    })
}
fn source_context<'a>(
    copied: &'a AnnularCopyCapture,
    identity: &PartitionOccurrenceIdentity,
) -> AnnularCheck<'static, &'a RegionObservedContext> {
    let matches: Vec<_> = copied
        .members
        .iter()
        .filter(|m| &m.copied.occurrence == identity)
        .collect();
    require("unique copied full chart or trim source", matches.len(), 1)?;
    Ok(&matches[0].copied)
}
fn chart_boundary(
    chart: &BoundPrismChart,
    boundaries: &[AnnularBoundaryBinding],
    copied: &AnnularCopyCapture,
) -> AnnularCheck<'static, Option<RegionSourceId>> {
    if chart.role != PrismImageRole::Generated {
        holds(
            "cap image of actual copied face",
            chart.source == copied.copied.face,
        )?;
        return Ok(None);
    }
    let matches: Vec<_> = boundaries
        .iter()
        .filter(|b| b.copied.occurrence == chart.source)
        .collect();
    require("one authored source for generated side", matches.len(), 1)?;
    Ok(Some(matches[0].source))
}
fn bind_trims(
    chart: &BoundPrismChart,
    face: &RegionObservedContext,
    contexts: &[RegionObservedContext],
    copied: &AnnularCopyCapture,
) -> AnnularCheck<'static, Vec<AnnularTrimIdentity>> {
    let actual: Vec<_> = contexts
        .iter()
        .filter(|c| c.kind == ShapeEnum::Edge && below(c, face))
        .collect();
    require(
        "complete actual directed trim census",
        chart.captured.trims.len(),
        actual.len(),
    )?;
    let mut trims = Vec::new();
    for trim in &chart.captured.trims {
        trims.push(bind_trim(trim, &actual, copied)?);
    }
    complete_trim_uses(&actual, &trims)?;
    Ok(trims)
}
fn below(child: &RegionObservedContext, parent: &RegionObservedContext) -> bool {
    child.child_slots.len() == parent.child_slots.len() + 2
        && child.child_slots.starts_with(&parent.child_slots)
        && child.ancestors.get(parent.ancestors.len()) == Some(&parent.occurrence)
        && child.ancestors.starts_with(&parent.ancestors)
}
fn complete_roles(
    charts: &[AnnularBoundChart],
    boundaries: &[AnnularBoundaryBinding],
) -> AnnularCheck<'static> {
    complete_caps(charts)?;
    for boundary in boundaries {
        require(
            "one generated chart per authored source",
            charts
                .iter()
                .filter(|c| c.source_boundary == Some(boundary.source))
                .count(),
            1,
        )?;
    }
    require(
        "twelve directed annular trim uses",
        charts.iter().map(|c| c.trims.len()).sum(),
        12,
    )
}

fn capture_native(
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
    preflight: &mut impl FnMut() -> AnnularCheck<'static>,
) -> AnnularCheck<'static, Vec<BoundPrismChart>> {
    let mut refusal = None;
    let actual = capture_actual_charts_guarded(receipt, cancelled, admit_faces, &mut || {
        if let Err(cause) = preflight() {
            refusal = Some(cause);
            return Err(PrismBindingFailure::Cancelled);
        }
        Ok(())
    });
    if let Some(cause) = refusal {
        return Err(cause);
    }
    actual.map_err(|cause| Box::new(AnnularBindingCause::Chart(Box::new(cause))))
}

fn bind_trim(
    trim: &super::super::super::surface_charts::CapturedPrismTrim,
    actual: &[&RegionObservedContext],
    copied: &AnnularCopyCapture,
) -> AnnularCheck<'static, AnnularTrimIdentity> {
    let matches: Vec<_> = actual
        .iter()
        .filter(|c| c.occurrence == trim.output)
        .collect();
    require("unique full directed trim occurrence", matches.len(), 1)?;
    Ok(AnnularTrimIdentity {
        source: source_context(copied, &trim.source)?.clone(),
        output: (*matches[0]).clone(),
        role: trim.role,
    })
}

fn complete_caps(charts: &[AnnularBoundChart]) -> AnnularCheck<'static> {
    for role in [PrismImageRole::First, PrismImageRole::Last] {
        require(
            "one source-face cap image per role",
            charts.iter().filter(|c| c.identity.role == role).count(),
            1,
        )?;
    }
    Ok(())
}

fn complete_trim_uses(
    actual: &[&RegionObservedContext],
    trims: &[AnnularTrimIdentity],
) -> AnnularCheck<'static> {
    for actual in actual {
        require(
            "one retained identity per actual trim",
            trims.iter().filter(|t| t.output == **actual).count(),
            1,
        )?;
    }
    Ok(())
}
