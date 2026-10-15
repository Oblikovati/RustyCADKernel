// SPDX-License-Identifier: GPL-2.0-only
//! Published planar regions: immutable faces with their loops, measures and the complete
//! source-curve to edge mapping.
use super::admitted::WorldFrame;
use super::decline::OperationDecline;
use super::loops::ProfileLoop;
use super::profile::{ProfileCurveId, ProfileVertexId};
use super::resolution::{AchievedTolerance, Resolution};
use super::validation::RegionValidation;
use crate::measured::Measured;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};

/// Index of a region within its `PlanarRegionSet`: the caller order of the first curve of
/// its outer loop decides it, never coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionId(pub u32);

/// The single native edge generated from one caller curve and the regions bounded by it.
#[derive(Clone, Debug)]
pub struct CurveEdge {
    /// The caller curve.
    pub curve: ProfileCurveId,
    /// Its native edge, shared by every region it bounds.
    pub edge: Edge,
    /// Regions whose boundary uses this edge, ascending.
    pub regions: Vec<RegionId>,
}

/// One validated planar face: its outer loop and holes.
#[derive(Clone, Debug)]
pub struct PlanarRegion {
    pub(crate) id: RegionId,
    pub(crate) face: Face,
    pub(crate) outer: ProfileLoop,
    pub(crate) holes: Vec<ProfileLoop>,
    pub(crate) area: Measured,
    pub(crate) edges: Vec<(ProfileCurveId, Edge)>,
    pub(crate) vertices: Vec<(ProfileVertexId, Vertex)>,
    pub(crate) frame: WorldFrame,
    pub(crate) validation: RegionValidation,
    pub(crate) achieved: AchievedTolerance,
}

impl PlanarRegion {
    /// Region identity; `region.id()`.
    pub fn id(&self) -> RegionId {
        self.id
    }
    /// The native face, read-only; `region.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }
    /// The outer loop, counterclockwise about the plane normal in the face.
    pub fn outer(&self) -> &ProfileLoop {
        &self.outer
    }
    /// The hole loops, in caller order.
    pub fn holes(&self) -> &[ProfileLoop] {
        &self.holes
    }
    /// Face area: outer area minus hole areas.
    pub fn area(&self) -> Measured {
        self.area
    }
    /// Every boundary edge with its caller curve, outer loop first, in loop order.
    pub fn edges(&self) -> &[(ProfileCurveId, Edge)] {
        &self.edges
    }
    /// Every authored vertex of the boundary with its native vertex, ascending by id.
    pub fn vertices(&self) -> &[(ProfileVertexId, Vertex)] {
        &self.vertices
    }
    /// Unit plane normal of the face orientation.
    pub fn normal(&self) -> [f64; 3] {
        self.frame.normal
    }
    /// Ordered validation of the face.
    pub fn validation(&self) -> RegionValidation {
        self.validation
    }
    /// Largest stored tolerances of the face.
    pub fn achieved_tolerance(&self) -> AchievedTolerance {
        self.achieved
    }
}

/// Certified evidence counts of a region set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionCertificate {
    /// Pair and self domains of Bézier pieces certified by the loop region proof.
    pub pair_domains: usize,
    /// Of those, the pairs certified by the common-carrier route (pieces of one circle or
    /// one line).
    pub carrier_pairs: usize,
    /// Closed loops whose nesting was certified.
    pub loops: usize,
}

/// All bounded faces of a solved profile and the complete curve to edge mapping.
#[derive(Clone, Debug)]
pub struct PlanarRegionSet {
    pub(crate) regions: Vec<PlanarRegion>,
    pub(crate) curve_edges: Vec<CurveEdge>,
    pub(crate) resolution: Resolution,
    pub(crate) certificate: RegionCertificate,
}

impl PlanarRegionSet {
    /// Every region, ordered by `RegionId`.
    pub fn regions(&self) -> &[PlanarRegion] {
        &self.regions
    }
    /// The region `id`, if present.
    pub fn region(&self, id: RegionId) -> Option<&PlanarRegion> {
        self.regions.get(id.0 as usize)
    }
    /// One entry per caller curve, in caller order.
    pub fn curve_edges(&self) -> &[CurveEdge] {
        &self.curve_edges
    }
    /// The resolution the set was built with.
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }
    /// Certified evidence counts.
    pub fn certificate(&self) -> RegionCertificate {
        self.certificate
    }
}

/// Result of `build_planar_regions`.
pub type PlanarRegionResult = Result<PlanarRegionSet, OperationDecline>;
