// SPDX-License-Identifier: GPL-2.0-only
//! Crate-level entry to the general loop region proof (ADR 0066 part C) for the public
//! profile extrusion seam (ADR 0068): owned plain inputs in, the certificate or a rendered
//! typed refusal out. It adds no proof step of its own.
use super::admission::GRAPH_PROFILE_VERSION;
use super::*;
use crate::sketch_region_prism::region_certificate::loop_region::graph::GraphRegionCertificate;
use crate::sketch_region_prism::region_certificate::loop_region::{
    LoopRegionCause, LoopRegionCertificate, LoopRegionFailure, LoopRegionOptions,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
use rustycad_tkmath::predicates::roots::metric_bounds::RationalIntegralOptions;

/// One Bézier edge of the proof request in world coordinates.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProofEdge {
    pub id: u64,
    pub world_poles: Vec<[f64; 3]>,
    pub weights: Option<Vec<f64>>,
    pub ends: [u64; 2],
}

/// Owned inputs: edges, loops of `(edge, reversed)` uses, `(name, chart seed)` regions.
pub(crate) struct ProfileProofInput {
    pub edges: Vec<ProofEdge>,
    pub loops: Vec<(u64, Vec<(u64, bool)>)>,
    pub regions: Vec<(u64, [f64; 2])>,
    pub plane: Plane3,
    /// Spatial limit for census contacts and transported endpoints.
    pub spatial_limit: f64,
}

/// A refusal of the proof, typed where the public seam distinguishes it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ProfileProofRefusal {
    /// Two edges meet away from an authored join.
    UnexpectedContact { edges: [u64; 2], point: [f64; 2] },
    /// ADR 0084: branches leave `vertex` in one direction, so their order is undecided.
    TangentBranches { vertex: u64 },
    /// ADR 0084: a face or hole boundary revisits `vertex`.
    Pinched { vertex: u64, hole: bool },
    /// Any other refused or incomplete obligation, rendered with its stage.
    Incomplete(String),
}

struct NeverStop;
impl OwnedPrismProgress for NeverStop {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}

/// Prepare and certify; `certify_profile(input)?` returns the complete certificate.
pub(crate) fn certify_profile(
    input: ProfileProofInput,
) -> Result<LoopRegionCertificate, ProfileProofRefusal> {
    let prepared =
        prepare_owned_profile(request(input), &mut NeverStop).map_err(preparation_refusal)?;
    prepared.inspect_regions(|| false).map_err(region_refusal)
}

/// Plane graph inputs (ADR 0084): edges with their vertex ids, no loops or seeds.
pub(crate) struct ProfileGraphInput {
    pub edges: Vec<ProofEdge>,
    pub plane: Plane3,
    /// Spatial limit for census contacts and transported endpoints.
    pub spatial_limit: f64,
}

/// Prepare and certify a plane graph; `certify_profile_graph(input)?` returns every
/// bounded face with its holes and certified area.
pub(crate) fn certify_profile_graph(
    input: ProfileGraphInput,
) -> Result<GraphRegionCertificate, ProfileProofRefusal> {
    let request = OwnedProfileRequest {
        version: GRAPH_PROFILE_VERSION,
        operation: CallerOperationId(1),
        edges: input.edges.into_iter().map(owned_edge).collect(),
        loops: Vec::new(),
        regions: Vec::new(),
        declared_plane: input.plane,
        policy: policy(input.spatial_limit),
    };
    let prepared = prepare_owned_profile(request, &mut NeverStop).map_err(preparation_refusal)?;
    prepared
        .inspect_plane_graph(|| false)
        .map_err(region_refusal)
}

fn preparation_refusal(failure: Box<ProfilePreparationFailure>) -> ProfileProofRefusal {
    let cause = match &failure.cause {
        ProfilePreparationCause::Admission(cause) => format!("{cause:?}"),
        ProfilePreparationCause::Stage(_) => "stage refusal".to_string(),
    };
    let (class, phase) = (failure.class, failure.phase);
    ProfileProofRefusal::Incomplete(format!(
        "profile preparation {class:?} at {phase:?}: {cause}"
    ))
}

fn region_refusal(failure: LoopRegionFailure) -> ProfileProofRefusal {
    match failure.cause {
        LoopRegionCause::UnexpectedContact { sources, point } => {
            let edges = sources.map(|source| source.0);
            ProfileProofRefusal::UnexpectedContact { edges, point }
        }
        LoopRegionCause::TangentBranches { vertex } => {
            ProfileProofRefusal::TangentBranches { vertex }
        }
        LoopRegionCause::PinchedFace { vertex } => ProfileProofRefusal::Pinched {
            vertex,
            hole: false,
        },
        LoopRegionCause::PinchedHole { vertex } => {
            ProfileProofRefusal::Pinched { vertex, hole: true }
        }
        cause => ProfileProofRefusal::Incomplete(format!("{:?}: {cause:?}", failure.stage)),
    }
}

fn request(input: ProfileProofInput) -> OwnedProfileRequest {
    let policy = policy(input.spatial_limit);
    OwnedProfileRequest {
        version: 1,
        operation: CallerOperationId(1),
        edges: input.edges.into_iter().map(owned_edge).collect(),
        loops: input.loops.into_iter().map(owned_loop).collect(),
        regions: input
            .regions
            .into_iter()
            .map(|(name, seed)| OwnedProfileRegion { name, seed })
            .collect(),
        declared_plane: input.plane,
        policy,
    }
}

fn owned_edge(edge: ProofEdge) -> OwnedProfileEdge {
    OwnedProfileEdge {
        source: CallerBoundaryId(edge.id),
        revision: CallerSourceRevision(1),
        world_poles: edge.world_poles,
        world_weights: edge.weights,
        start: CallerVertexId(edge.ends[0]),
        end: CallerVertexId(edge.ends[1]),
    }
}

fn owned_loop((name, uses): (u64, Vec<(u64, bool)>)) -> OwnedProfileLoop {
    let uses = uses.into_iter().map(|(edge, reversed)| OwnedProfileUse {
        edge: CallerBoundaryId(edge),
        reversed,
    });
    OwnedProfileLoop {
        name,
        uses: uses.collect(),
    }
}

/// The ADR 0066 version-1 child policies with the caller's spatial limit.
fn policy(limit: f64) -> OwnedProfilePolicy {
    let roots = RootIsolationOptions::new(0.001);
    let (original_plane_limit, construction_limit) = (0., 0.);
    let area = RationalIntegralOptions::default();
    OwnedProfilePolicy {
        census: CensusPolicy {
            roots,
            max_world_error: limit,
        },
        source: SourceConformancePolicy {
            roots,
            original_plane_limit,
            construction_limit,
        },
        contacts: contact_policy(limit, roots),
        region: LoopRegionOptions {
            version: 1,
            winding: roots,
            area,
        },
    }
}

fn contact_policy(limit: f64, roots: RootIsolationOptions) -> ContactTransportPolicy {
    ContactTransportPolicy {
        native_parameter_limits: [limit; 2],
        world_endpoint_limit: limit,
        roots,
    }
}
