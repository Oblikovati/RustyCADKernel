// SPDX-License-Identifier: GPL-2.0-only
//! Consume sealed zero reports only after exact raw curve, support and point frame admission.
use super::{progress::AnnularExactWork, *};
use crate::sketch_region_prism::exact_tolerance::{frames::admit_tolerance_frame, scalar::*};
use crate::sketch_region_source_conformance::{
    CapturedSpaceRepresentation, captured_endpoint_root_options,
};
use rustycad_tkmath::predicates::roots::metric_bounds::{
    MetricEmbedding, TransportedResidualReport,
};
pub(super) fn inspect(
    binding: &AnnularPrismBinding,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    count(
        "annular source closure owners",
        binding.region().sources().len(),
        2,
    )?;
    for source in binding.region().sources() {
        original_and_census(source, work, progress)?;
        super::source_reports::retain(source, work)?;
    }
    candidates(binding, work, progress)?;
    super::source_copy::inspect(binding, work, progress)?;
    complete_sources(work)
}
fn original_and_census(
    source: &TwoLoopSourceProof,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    for (stage, curve, endpoints) in source_groups(source) {
        let context = AnnularSourceContext {
            source: source.name,
            stage,
            native: None,
        };
        group(
            context,
            curve,
            source.support.placement,
            endpoints,
            work,
            progress,
        )?;
    }
    Ok(())
}
fn candidates(
    binding: &AnnularPrismBinding,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    count(
        "annular complete candidate proof universe",
        binding.region().candidates().len(),
        3,
    )?;
    let mut wires = 0;
    for candidate in binding.region().candidates() {
        for wire in &candidate.wires {
            candidate_wire(candidate, wire, work, progress)?;
            wires += 1;
        }
    }
    count("all source-keyed actual candidate wires", wires, 4)?;
    Ok(())
}
pub(super) fn group(
    context: AnnularSourceContext,
    curve: &CapturedSpaceRepresentation,
    support: MetricEmbedding<3>,
    endpoints: &[TwoLoopEndpointProof],
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    work.source_context = Some(context);
    work.body.poll(progress)?;
    unique_group(context, &work.upstream.supports)?;
    count(
        "two native endpoints per upstream semantic context",
        endpoints.len(),
        2,
    )?;
    let frame = admit_tolerance_frame(support).map_err(AnnularConformanceCause::SourceFrame)?;
    work.upstream
        .supports
        .push(AnnularSupportEvidence { context, frame });
    native_endpoints(context, curve, endpoints, work, progress)
}
fn unique_group(
    context: AnnularSourceContext,
    rows: &[AnnularSupportEvidence],
) -> AnnularExactCheck {
    if rows.iter().any(|row| row.context == context) {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "unique source/stage/candidate/wire group",
        }
        .into());
    }
    Ok(())
}
fn append(
    context: AnnularSourceContext,
    curve: &CapturedSpaceRepresentation,
    row: &TwoLoopEndpointProof,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    work.source_context = Some(context);
    work.body.poll(progress)?;
    let certificate = retained_endpoint(curve, row)
        .map_err(|cause| AnnularConformanceCause::SourceEndpoint(Box::new(cause)))?;
    work.upstream.closures.push(AnnularClosureEvidence {
        context,
        reused_from: reused_origin(context),
        parameter: row.parameter,
        orientation: row.orientation,
        local_point: row.local_point,
        source_raw_tolerance: row.raw_tolerance,
        certificate,
    });
    Ok(())
}
pub(super) fn retained_endpoint(
    curve: &CapturedSpaceRepresentation,
    row: &TwoLoopEndpointProof,
) -> Result<ExactEndpointCertificate, ExactEndpointFailure> {
    let curve_frame = endpoint_frame(curve.placement, EndpointFrameRole::CurveRepresentation)?;
    let vertex_frame = endpoint_frame(row.placement, EndpointFrameRole::VertexOccurrence)?;
    // This report is already sealed to actual original operands; no numerical reevaluation occurs.
    let report = TransportedResidualReport {
        decision: Ok(row.residual.clone()),
        bisections: row.bisections,
    };
    complete_zero_report(
        curve_frame,
        vertex_frame,
        captured_endpoint_root_options(),
        report,
    )
}

fn reused_origin(context: AnnularSourceContext) -> Option<AnnularSourceContext> {
    if let AnnularClosureStage::PrismCopy { candidate, wire } = context.stage {
        return Some(AnnularSourceContext {
            stage: AnnularClosureStage::Candidate { candidate, wire },
            ..context
        });
    }
    None
}

fn complete_sources(work: &AnnularExactWork) -> AnnularExactCheck {
    count(
        "all source/census/candidate/copied native endpoints",
        work.upstream.closures.len(),
        20,
    )?;
    count(
        "all upstream source conformance reports",
        work.upstream.source_reports.len(),
        6,
    )?;
    count(
        "all source-group support frames",
        work.upstream.supports.len(),
        10,
    )?;
    Ok(())
}

fn source_groups(
    source: &TwoLoopSourceProof,
) -> [(
    AnnularClosureStage,
    &CapturedSpaceRepresentation,
    &[TwoLoopEndpointProof],
); 2] {
    [
        (
            AnnularClosureStage::Original,
            &source.original,
            &source.closure,
        ),
        (
            AnnularClosureStage::Census,
            &source.copied,
            &source.copied_closure,
        ),
    ]
}

fn candidate_wire(
    candidate: &TwoLoopCandidateProof,
    wire: &TwoLoopWireProof,
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    let context = AnnularSourceContext {
        source: wire.source,
        stage: AnnularClosureStage::Candidate {
            candidate: candidate.name,
            wire: wire.name,
        },
        native: None,
    };
    group(
        context,
        &wire.space,
        candidate.support.placement,
        &wire.closure,
        work,
        progress,
    )?;
    Ok(())
}

fn native_endpoints(
    context: AnnularSourceContext,
    curve: &CapturedSpaceRepresentation,
    endpoints: &[TwoLoopEndpointProof],
    work: &mut AnnularExactWork,
    progress: &mut impl JoinedConformanceProgress,
) -> AnnularExactCheck {
    for (native, parameter) in [
        (CanonicalEndpointSide::First, 0_f64),
        (CanonicalEndpointSide::Last, 1_f64),
    ] {
        let row = native_endpoint(endpoints, parameter)?;
        append(
            AnnularSourceContext {
                native: Some(native),
                ..context
            },
            curve,
            row,
            work,
            progress,
        )?;
    }
    Ok(())
}

fn native_endpoint(
    endpoints: &[TwoLoopEndpointProof],
    parameter: f64,
) -> AnnularExactCheck<&TwoLoopEndpointProof> {
    let rows: Vec<_> = endpoints
        .iter()
        .filter(|row| row.parameter.to_bits() == parameter.to_bits())
        .collect();
    one(
        &rows,
        "one exact native parameter role per upstream endpoint",
    )
    .copied()
    .map_err(Into::into)
}
