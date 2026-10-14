// SPDX-License-Identifier: GPL-2.0-only
//! Actual sweep charts are checked inside the retained history pipeline; body completion stays open.
mod cap;
mod capture;
mod coverage;
mod identity;
mod inspection;
mod progress;
mod side;
mod side_trims;
mod sources;
mod trim_binding;
mod trim_capture;
mod types;
pub(super) use capture::capture_bezier_edge;
pub(super) use inspection::retain_surface_charts;
pub(super) use sources::capture_sources;
pub(crate) use types::*;
#[cfg(test)]
mod degenerate_tests;
#[cfg(test)]
mod expected_tests;
#[cfg(test)]
mod mutation_tests;
#[cfg(test)]
mod progress_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod tests;

fn prove_chart(
    chart: &CapturedSurfaceChart,
    role: super::PrismImageRole,
    cancelled: &mut impl FnMut() -> bool,
    proof: &mut PrismSurfaceProof,
) -> Result<(), PrismSurfaceCause> {
    let mut context = identity::ProofContext { cancelled, proof };
    if role == super::PrismImageRole::Generated {
        return side::prove_side(chart, &mut context);
    }
    prove_cap_chart(chart, role, &mut context)
}
fn prove_cap_chart(
    chart: &CapturedSurfaceChart,
    role: super::PrismImageRole,
    context: &mut identity::ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let (CapturedChartSource::Cap(source), CapturedPrismSurface::Plane(actual)) =
        (&chart.source, &chart.actual)
    else {
        return Err(PrismSurfaceCause::MissingHistory {
            entity: "planar cap source and output",
        });
    };
    let offset = if role == super::PrismImageRole::Last {
        chart.extrusion
    } else {
        [0.; 3]
    };
    cap::prove_cap((source, actual), chart, offset, context)
}
pub(super) use trim_capture::pcurve_on;
