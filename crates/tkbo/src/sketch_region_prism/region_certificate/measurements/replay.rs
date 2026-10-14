// SPDX-License-Identifier: GPL-2.0-only
//! External callbacks finish before this fresh, callback-free binding pass.
use super::super::ClosurePointEvidence;
use super::super::{
    boundary::{same_boundary_geometry, same_support},
    identity_multiset::same_multiset,
    types::holds,
};
use super::*;
use crate::sketch_region_source_conformance::{
    same_embedding, same_metric_curve, same_space_representation,
};

pub(super) fn rebind(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    region: &RegionCertificate,
    charts: &[MeasuredPrismChart],
) -> MeasureCheck {
    let (current, _) = binding::current_region(bridge, sources, receipt, &mut || false)?;
    holds(
        "measured original/contact/traversal still current",
        same_region(region, &current),
    )?;
    let actual = charts::current_charts(receipt, &mut || false)?;
    same_charts(charts, &actual)
}
pub(super) fn same_charts(
    charts: &[MeasuredPrismChart],
    actual: &[MeasuredPrismChart],
) -> MeasureCheck {
    for measured in charts {
        let current = actual
            .iter()
            .find(|row| row.role == measured.role)
            .expect("unique complete chart roles");
        holds(
            "measured actual chart coefficients and identities still current",
            same_chart(measured, current),
        )?;
    }
    Ok(())
}
fn same_region(a: &RegionCertificate, b: &RegionCertificate) -> bool {
    a.original == b.original
        && a.census_occurrence == b.census_occurrence
        && a.contact == b.contact
        && a.selected_winding == b.selected_winding
        && a.boundary_reversed == b.boundary_reversed
        && same_boundary(&a.source_candidate, &b.source_candidate)
        && same_boundary(&a.complementary_candidate, &b.complementary_candidate)
        && same_boundary(&a.copied, &b.copied)
        && same_closure_points(&a.closure_points, &b.closure_points)
}
fn same_boundary(a: &CapturedPrismSources, b: &CapturedPrismSources) -> bool {
    let first = &a.boundaries[0];
    let second = &b.boundaries[0];
    a.face == b.face
        && first.occurrence == second.occurrence
        && same_multiset(&first.endpoints, &second.endpoints)
        && same_boundary_geometry(a, b)
}
fn same_chart(a: &MeasuredPrismChart, b: &MeasuredPrismChart) -> bool {
    a.source == b.source
        && a.output == b.output
        && a.role == b.role
        && same_multiset(&a.parents, &b.parents)
        && a.captured.extrusion.map(f64::to_bits) == b.captured.extrusion.map(f64::to_bits)
        && same_chart_source(&a.captured.source, &b.captured.source)
        && same_surface(&a.captured.actual, &b.captured.actual)
        && same_trims(&a.captured.trims, &b.captured.trims)
}
fn same_chart_source(a: &CapturedChartSource, b: &CapturedChartSource) -> bool {
    match (a, b) {
        (CapturedChartSource::Cap(a), CapturedChartSource::Cap(b)) => same_support(a, b),
        (CapturedChartSource::Side(a), CapturedChartSource::Side(b)) => {
            same_space_representation(a, b)
        }
        _ => false,
    }
}
fn same_surface(a: &CapturedPrismSurface, b: &CapturedPrismSurface) -> bool {
    if let (CapturedPrismSurface::Plane(a), CapturedPrismSurface::Plane(b)) = (a, b) {
        return same_support(a, b);
    }
    same_extrusion(a, b)
}
fn same_extrusion(a: &CapturedPrismSurface, b: &CapturedPrismSurface) -> bool {
    let (
        CapturedPrismSurface::Extrusion {
            basis: a,
            direction: b,
            placement: c,
        },
        CapturedPrismSurface::Extrusion {
            basis: d,
            direction: e,
            placement: f,
        },
    ) = (a, b)
    else {
        return false;
    };
    same_metric_curve(a, d) && b.map(f64::to_bits) == e.map(f64::to_bits) && same_embedding(c, f)
}
fn same_trims(a: &[CapturedPrismTrim], b: &[CapturedPrismTrim]) -> bool {
    a.len() == b.len()
        && a.iter()
            .all(|first| b.iter().filter(|second| same_trim(first, second)).count() == 1)
        && b.iter()
            .all(|second| a.iter().filter(|first| same_trim(first, second)).count() == 1)
}
fn same_trim(a: &CapturedPrismTrim, b: &CapturedPrismTrim) -> bool {
    a.output == b.output
        && a.source == b.source
        && a.role == b.role
        && same_multiset(&a.parents, &b.parents)
        && a.interval.map(f64::to_bits) == b.interval.map(f64::to_bits)
        && same_metric_curve(&a.pcurve, &b.pcurve)
        && same_space_representation(&a.space, &b.space)
        && same_expectation(&a.expected, &b.expected)
}

fn same_closure_points(a: &[ClosurePointEvidence], b: &[ClosurePointEvidence]) -> bool {
    let rows = |points: &[ClosurePointEvidence]| {
        points
            .iter()
            .flat_map(|point| {
                point.contexts.iter().map(|context| {
                    (
                        context.stage,
                        context.occurrence.clone(),
                        point.point.map(f64::to_bits),
                        point.original.clone(),
                    )
                })
            })
            .collect::<Vec<_>>()
    };
    same_multiset(&rows(a), &rows(b))
}
fn same_expectation(a: &CapturedTrimExpectation, b: &CapturedTrimExpectation) -> bool {
    match (a, b) {
        (CapturedTrimExpectation::SideGenerating, CapturedTrimExpectation::SideGenerating) => true,
        (
            CapturedTrimExpectation::SideDirecting { parameter: a },
            CapturedTrimExpectation::SideDirecting { parameter: b },
        ) => a.to_bits() == b.to_bits(),
        _ => same_cap_expectation(a, b),
    }
}
fn same_cap_expectation(a: &CapturedTrimExpectation, b: &CapturedTrimExpectation) -> bool {
    let (
        CapturedTrimExpectation::Cap {
            pcurve: a,
            interval: b,
            space: c,
        },
        CapturedTrimExpectation::Cap {
            pcurve: d,
            interval: e,
            space: f,
        },
    ) = (a, b)
    else {
        return false;
    };
    same_metric_curve(a, d)
        && b.map(f64::to_bits) == e.map(f64::to_bits)
        && same_space_representation(c, f)
}
