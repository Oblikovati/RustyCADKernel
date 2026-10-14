// SPDX-License-Identifier: GPL-2.0-only
//! Actual copied closure operands reuse selected proof only through the sealed exact correspondence.
use super::{progress::AnnularExactWork, *};
use crate::sketch_region_prism::surface_charts::CapturedSourceBoundary;
use crate::sketch_region_source_conformance::{same_embedding, same_space_representation};
pub(super) fn inspect(
    binding: &AnnularPrismBinding,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    count(
        "actual selected annular copied boundaries",
        binding.boundaries().len(),
        2,
    )?;
    let candidates: Vec<_> = binding
        .region()
        .candidates()
        .iter()
        .filter(|row| row.name == binding.region().selected_candidate_id())
        .collect();
    let selected = *one(&candidates, "one selected annulus for copied closure")?;
    for boundary in binding.boundaries() {
        let wires: Vec<_> = selected
            .wires
            .iter()
            .filter(|wire| wire.name == boundary.wire && wire.source == boundary.source)
            .collect();
        let wire = *one(&wires, "one source-keyed selected wire for copied closure")?;
        copied_group(binding, boundary, selected.name, wire, work, progress)?;
    }
    Ok(())
}
fn copied_group(
    binding: &AnnularPrismBinding,
    boundary: &AnnularBoundaryBinding,
    candidate: u64,
    wire: &TwoLoopWireProof,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    let context = AnnularSourceContext {
        source: wire.source,
        stage: AnnularClosureStage::PrismCopy {
            candidate,
            wire: wire.name,
        },
        native: None,
    };
    work.source_context = Some(context);
    let curve = actual_curve(binding, boundary, wire)?;
    exact_endpoints(boundary, wire)?;
    super::source_closures::group(
        context,
        &curve.space,
        binding.copied_sources().support.placement,
        &wire.closure,
        work,
        progress,
    )?;
    Ok(())
}
fn actual_curve<'a>(
    binding: &'a AnnularPrismBinding,
    boundary: &AnnularBoundaryBinding,
    wire: &TwoLoopWireProof,
) -> AnnularExactCheck<&'a CapturedSourceBoundary> {
    let curves: Vec<_> = binding
        .copied_sources()
        .boundaries
        .iter()
        .filter(|row| row.occurrence == boundary.copied.occurrence)
        .collect();
    let curve = *one(&curves, "one actual copied full directed curve occurrence")?;
    if !same_space_representation(&curve.space, &wire.space) {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "bit-identical copied closure curve before proof reuse",
        }
        .into());
    }
    Ok(curve)
}
fn exact_endpoints(
    boundary: &AnnularBoundaryBinding,
    wire: &TwoLoopWireProof,
) -> AnnularExactCheck {
    count(
        "actual copied native endpoint contexts",
        boundary.endpoints.len(),
        2,
    )?;
    for parameter in [0_f64, 1_f64] {
        let actual: Vec<_> = boundary
            .endpoints
            .iter()
            .filter(|row| row.parameter.to_bits() == parameter.to_bits())
            .collect();
        let proof: Vec<_> = wire
            .closure
            .iter()
            .filter(|row| row.parameter.to_bits() == parameter.to_bits())
            .collect();
        same_endpoint(
            one(&actual, "one copied native endpoint")?,
            one(&proof, "one selected native endpoint proof")?,
        )?;
    }
    Ok(())
}
fn same_endpoint(
    actual: &AnnularNativeEndpoint,
    proof: &TwoLoopEndpointProof,
) -> AnnularExactCheck {
    if actual.local_point.map(f64::to_bits) != proof.local_point.map(f64::to_bits)
        || !same_embedding(&actual.placement, &proof.placement)
        || actual.context.occurrence.orientation != proof.orientation
    {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "exact copied native raw operands before source proof reuse",
        }
        .into());
    }
    Ok(())
}
