// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0066 part C request, certificate and refusal types of the general loop region proof.
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use crate::sketch_region_contact_transport::PairedContactTransaction;
use rustycad_tkmath::predicates::roots::metric_bounds::{
    CertifiedRationalMeasure, RationalIntegralOptions,
};
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    bezier_line::{BezierWireError, BezierWireLocation},
};

/// Version 1 is the only behavior; winding work is bounded per query.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LoopRegionOptions {
    pub version: u32,
    pub winding: RootIsolationOptions,
    /// Certified face-area policy (ADR 0066 parts A and C); the cap is in chart area units.
    pub area: RationalIntegralOptions,
}
/// One directed use of an arrangement source; `reversed` is relative to native `[0,1]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AuthoredLoopUse {
    pub source: RegionSourceId,
    pub reversed: bool,
}
/// Authored closed loop hypothesis; the proof certifies or refuses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuthoredLoop {
    pub name: u64,
    pub uses: Vec<AuthoredLoopUse>,
}
/// Authored region selection; the seed is in the census plane chart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AuthoredRegion {
    pub name: u64,
    pub seed: [f64; 2],
}
/// Borrowed inputs: the actual contact transaction and the authored hypotheses.
pub(crate) struct LoopRegionRequest<'a, 'b> {
    pub bridge: &'a PairedContactTransaction,
    pub sources: &'b [ArrangementSource<'a>],
    pub loops: &'b [AuthoredLoop],
    pub regions: &'b [AuthoredRegion],
    pub options: LoopRegionOptions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LoopRegionStage {
    Admission,
    Census,
    Joins,
    Nesting,
    Regions,
    /// ADR 0084: rotation system, orbits and their orientation.
    Faces,
}
/// Typed refusal; no partial certificate accompanies it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LoopRegionCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Count {
        quantity: &'static str,
        actual: usize,
        expected: usize,
    },
    Binding {
        quantity: &'static str,
        source: RegionSourceId,
    },
    Obligation {
        quantity: &'static str,
    },
    PairUnavailable {
        sources: [RegionSourceId; 2],
    },
    UnexpectedContact {
        sources: [RegionSourceId; 2],
        point: [f64; 2],
    },
    MissingJoin {
        sources: [RegionSourceId; 2],
        parameter: [u8; 2],
    },
    Winding {
        loop_name: u64,
        cause: BezierWireError,
    },
    NonJordanWinding {
        loop_name: u64,
        point: [f64; 2],
        actual: BezierWireLocation,
    },
    SeedOnBoundary {
        region: u64,
        loop_name: u64,
    },
    NestingContradiction {
        loops: [u64; 2],
    },
    UnboundedSelection {
        region: u64,
    },
    DuplicateSelection {
        regions: [u64; 2],
    },
    /// The face area was not certified; the numerical cause keeps its typed display.
    Area {
        region: u64,
        curve_index: Option<usize>,
        cause: String,
    },
    Cancelled,
    /// ADR 0084: two darts leave a vertex of degree three or more in one direction.
    TangentBranches {
        vertex: u64,
    },
    /// ADR 0084: darts at one vertex start at different chart points.
    VertexChart {
        vertex: u64,
    },
    /// ADR 0084: a source with all controls equal.
    DegenerateSource {
        vertex: u64,
    },
    /// ADR 0084: an orbit's certified Green integral contains zero.
    OrbitOrientation {
        orbit: usize,
        enclosure: [f64; 2],
    },
    /// ADR 0084: a component without exactly one negative (outer) orbit.
    OrbitCount {
        component: usize,
        outer: usize,
    },
    /// ADR 0084: a bounded face whose boundary walk revisits `vertex`.
    PinchedFace {
        vertex: u64,
    },
    /// ADR 0084: a nested component whose outer walk revisits `vertex`.
    PinchedHole {
        vertex: u64,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoopRegionFailure {
    pub stage: LoopRegionStage,
    pub cause: LoopRegionCause,
}
/// One certified exact join contact between consecutive uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct JoinProof {
    pub sources: [RegionSourceId; 2],
    /// Exact native parameters of the contact on each source, each 0 or 1.
    pub parameter: [u8; 2],
}
/// One certified Jordan loop and its place in the nesting forest.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoopProof {
    pub name: u64,
    pub uses: Vec<AuthoredLoopUse>,
    /// Innermost authored loop strictly containing this loop, if any.
    pub parent: Option<u64>,
    /// Exact start point of the first use, in the census plane chart.
    pub witness: [f64; 2],
}
/// One certified selected face: the open set inside `outer` and outside every hole.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RegionProof {
    pub name: u64,
    pub seed: [f64; 2],
    pub outer: u64,
    pub holes: Vec<u64>,
    /// Exact signed winding of the seed for every loop, in request order.
    pub windings: Vec<(u64, i64)>,
    /// Certified chart area of the face: outer area minus hole areas.
    pub area: Option<CertifiedRationalMeasure>,
}
/// Complete loop region certificate; it publishes no body, history or names.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoopRegionCertificate {
    pub options: LoopRegionOptions,
    pub loops: Vec<LoopProof>,
    pub joins: Vec<JoinProof>,
    pub regions: Vec<RegionProof>,
    /// Pair domains certified by the ADR 0066 part B common-carrier route.
    pub carrier_pairs: usize,
    /// Pair domains certified in total (all distinct pairs and all self domains).
    pub pair_domains: usize,
}
pub(crate) type LoopRegionCheck<T> = Result<T, LoopRegionFailure>;
pub(crate) fn refuse<T>(stage: LoopRegionStage, cause: LoopRegionCause) -> LoopRegionCheck<T> {
    Err(LoopRegionFailure { stage, cause })
}
