// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0068 corpus fixtures: exact rational circles and polygons as owned profiles.
use super::*;
use crate::sketch_region_prism::region_certificate::loop_region::*;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
use rustycad_tkmath::predicates::roots::metric_bounds::RationalIntegralOptions;

mod corpus_tests;
mod refusal_tests;

const LIMIT: f64 = 1e-8;
/// Exact unit-circle quarters x = (1-t^2)/(1+t^2), y = 2t/(1+t^2), counterclockwise.
const QUARTERS: [[[f64; 2]; 3]; 4] = [
    [[1., 0.], [1., 1.], [0., 1.]],
    [[0., 1.], [-1., 1.], [-1., 0.]],
    [[-1., 0.], [-1., -1.], [0., -1.]],
    [[0., -1.], [1., -1.], [1., 0.]],
];

/// A profile under construction: edges, loops and regions with explicit ids.
#[derive(Default)]
pub(super) struct Profile {
    pub edges: Vec<OwnedProfileEdge>,
    pub loops: Vec<OwnedProfileLoop>,
    pub regions: Vec<OwnedProfileRegion>,
}
impl Profile {
    /// Add a circle as four exact rational quarter edges; ids `base..base+4`.
    pub(super) fn circle(&mut self, base: u64, center: [f64; 2], radius: f64, clockwise: bool) {
        let place = |[x, y]: [f64; 2]| [center[0] + radius * x, center[1] + radius * y, 0.];
        for (index, quarter) in QUARTERS.iter().enumerate() {
            let next = (index as u64 + 1) % 4;
            self.edges.push(OwnedProfileEdge {
                source: CallerBoundaryId(base + index as u64),
                revision: CallerSourceRevision(1),
                world_poles: quarter.iter().copied().map(place).collect(),
                world_weights: Some(vec![1., 1., 2.]),
                start: CallerVertexId(base + index as u64),
                end: CallerVertexId(base + next),
            });
        }
        let mut uses: Vec<_> = (0..4)
            .map(|i| OwnedProfileUse {
                edge: CallerBoundaryId(base + i),
                reversed: clockwise,
            })
            .collect();
        if clockwise {
            uses.reverse();
        }
        self.loops.push(OwnedProfileLoop {
            name: base + 1000,
            uses,
        });
    }
    /// Add a closed polygon of straight polynomial edges; ids `base..base+n`.
    pub(super) fn polygon(&mut self, base: u64, corners: &[[f64; 2]]) {
        let count = corners.len() as u64;
        for (index, corner) in corners.iter().enumerate() {
            let next = corners[(index + 1) % corners.len()];
            self.edges.push(OwnedProfileEdge {
                source: CallerBoundaryId(base + index as u64),
                revision: CallerSourceRevision(1),
                world_poles: vec![[corner[0], corner[1], 0.], [next[0], next[1], 0.]],
                world_weights: None,
                start: CallerVertexId(base + index as u64),
                end: CallerVertexId(base + (index as u64 + 1) % count),
            });
        }
        let uses = (0..count)
            .map(|i| OwnedProfileUse {
                edge: CallerBoundaryId(base + i),
                reversed: false,
            })
            .collect();
        self.loops.push(OwnedProfileLoop {
            name: base + 1000,
            uses,
        });
    }
    /// Add one planar edge between caller vertices `ends`.
    pub(super) fn edge(
        &mut self,
        id: u64,
        poles: &[[f64; 2]],
        weights: Option<&[f64]>,
        ends: [u64; 2],
    ) {
        self.edges.push(OwnedProfileEdge {
            source: CallerBoundaryId(id),
            revision: CallerSourceRevision(1),
            world_poles: poles.iter().map(|p| [p[0], p[1], 0.]).collect(),
            world_weights: weights.map(<[f64]>::to_vec),
            start: CallerVertexId(ends[0]),
            end: CallerVertexId(ends[1]),
        });
    }
    /// Add a loop of forward uses of `edges`, in order.
    pub(super) fn chain(&mut self, name: u64, edges: &[u64]) {
        let uses = edges
            .iter()
            .map(|id| OwnedProfileUse {
                edge: CallerBoundaryId(*id),
                reversed: false,
            })
            .collect();
        self.loops.push(OwnedProfileLoop { name, uses });
    }
    pub(super) fn select(&mut self, name: u64, seed: [f64; 2]) {
        self.regions.push(OwnedProfileRegion { name, seed });
    }
    pub(super) fn request(self) -> OwnedProfileRequest {
        OwnedProfileRequest {
            version: 1,
            operation: CallerOperationId(91),
            edges: self.edges,
            loops: self.loops,
            regions: self.regions,
            declared_plane: Plane3::default(),
            policy: policy(),
        }
    }
}
pub(super) fn policy() -> OwnedProfilePolicy {
    let roots = RootIsolationOptions::new(0.001);
    OwnedProfilePolicy {
        census: CensusPolicy {
            roots,
            max_world_error: LIMIT,
        },
        source: SourceConformancePolicy {
            roots,
            original_plane_limit: 0.,
            construction_limit: 0.,
        },
        contacts: ContactTransportPolicy {
            native_parameter_limits: [LIMIT; 2],
            world_endpoint_limit: LIMIT,
            roots,
        },
        region: LoopRegionOptions {
            version: 1,
            winding: roots,
            area: RationalIntegralOptions::default(),
        },
    }
}
pub(super) struct Continue;
impl OwnedPrismProgress for Continue {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}
/// Prepare and certify; panics with the typed refusal for diagnosis.
pub(super) fn certified(profile: Profile) -> LoopRegionCertificate {
    let prepared = prepared(profile);
    prepared
        .inspect_regions(|| false)
        .unwrap_or_else(|failure| panic!("{failure:?}"))
}
pub(super) fn prepared(profile: Profile) -> PreparedProfile {
    prepare_owned_profile(profile.request(), &mut Continue)
        .unwrap_or_else(|failure| panic!("preparation {:?} at {:?}", failure.class, failure.phase))
}
/// The certified face area contains the closed form; the allowance covers only the
/// binary64 rounding of the closed form itself (a few ulp of the expected value).
pub(super) fn assert_area(region: &RegionProof, expected: f64) {
    let area = region.area.as_ref().expect("certified face area");
    let allowance = area.absolute_error_bound() + 4. * f64::EPSILON * expected.abs();
    assert!(
        (area.representative() - expected).abs() <= allowance,
        "{area:?} vs {expected}"
    );
    assert!(area.absolute_error_bound() <= area.requested_options().absolute_error_limit);
}
