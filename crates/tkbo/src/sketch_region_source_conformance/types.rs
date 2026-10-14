// SPDX-License-Identifier: GPL-2.0-only
use crate::sketch_region::OriginalMemberIdentity;
use crate::sketch_region_geometry_capture::{CapturedGeometryCause, CapturedMetricCurve};
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*};
use std::sync::Arc;
pub(crate) type CapturedGeometry<T> = Result<T, Arc<CapturedGeometryCause>>;
#[derive(Clone)]
pub(crate) struct OriginalCurveCapture {
    pub original: OriginalMemberIdentity,
    pub representation: CapturedGeometry<CapturedSpaceRepresentation>,
}
#[derive(Clone)]
pub(crate) struct CapturedSpaceRepresentation {
    pub curve: CapturedMetricCurve<3>,
    pub interval: [f64; 2],
    pub placement: MetricEmbedding<3>,
    pub same_parameter: bool,
}
#[derive(Clone)]
pub(crate) struct CapturedBoundaryConstruction {
    pub occurrence: u64,
    pub copied_occurrence: crate::sketch_region::PartitionOccurrenceIdentity,
    pub parents: Vec<OriginalMemberIdentity>,
    pub copied: CapturedGeometry<CapturedSpaceRepresentation>,
    pub pcurve: CapturedGeometry<CapturedMetricCurve<2>>,
    pub native_interval: [f64; 2],
    pub basis_interval: [f64; 2],
    pub captured: CapturedMetricCurve<2>,
    pub support: CapturedGeometry<CapturedPlaneSupport>,
}
#[derive(Clone)]
pub(crate) struct CapturedPlaneSupport {
    pub frame: MetricEmbedding<2>,
    pub placement: MetricEmbedding<3>,
}
#[derive(Clone, Copy)]
pub(crate) struct SourceConformancePolicy {
    pub original_plane_limit: f64,
    pub construction_limit: f64,
    pub roots: RootIsolationOptions,
}
pub(crate) struct SourceConformanceInspection {
    pub entries: Vec<OriginalConstructionEvidence>,
    pub requested: SourceConformancePolicy,
    // Only scalar coefficients and occurrence keys, never mutable topology/geometry owners.
    pub originals: Vec<OriginalCurveCapture>,
    pub constructions: Vec<CapturedBoundaryConstruction>,
    pub bisections: u64,
}
pub(crate) struct OriginalConstructionEvidence {
    pub occurrence: u64,
    pub original: OriginalMemberIdentity,
    pub stages: Vec<ConstructionStageEvidence>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConstructionStage {
    OriginalPlane,
    CopiedPlane,
    CapturedBasis,
}
pub(crate) struct ConstructionStageEvidence {
    pub stage: ConstructionStage,
    pub requested_limit: f64,
    pub effective_limit: f64,
    pub effective_roots: RootIsolationOptions,
    pub bisections: u64,
    pub decision: Result<ResidualDecision, SourceConformanceCause>,
}
#[derive(Debug)]
pub(crate) enum SourceConformanceCause {
    Capture(Arc<CapturedGeometryCause>),
    MissingOriginal(OriginalMemberIdentity),
    NativeRange {
        original: [f64; 2],
        copied: [f64; 2],
        stored: [f64; 2],
    },
    InvalidSegmentIndex {
        received: usize,
        expected_length: usize,
    },
    CurrentDescriptorMismatch {
        occurrence: u64,
    },
    MissingConstruction {
        occurrence: u64,
    },
    UnknownParameterBinding,
    Exact(ResidualError),
}
pub(crate) struct NativeParameterMembership {
    pub native_bracket: [DyadicBound; 2],
    pub supplied_parameter: DyadicBound,
    pub belongs: bool,
    pub singleton_equal: bool,
}

impl std::fmt::Display for SourceConformanceCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capture(cause) => write!(f, "source coefficients {cause:?}; expected supported stored analytic representations"),
            Self::MissingOriginal(key) => write!(f, "original member {key:?}; expected exact captured source occurrence key"),
            Self::NativeRange { original, copied, stored } => write!(f, "original={original:?}, copied={copied:?}, stored={stored:?}; expected identical full native trims for a copy binding"),
            Self::InvalidSegmentIndex { received,expected_length } => write!(f,"segment index={received}; expected index below {expected_length}"),
            Self::CurrentDescriptorMismatch { occurrence } => write!(f,"occurrence={occurrence}; expected current scalar/physical capture matching native parameter map"),
            Self::MissingConstruction { occurrence } => write!(f,"occurrence={occurrence}; expected one actual captured construction"),
            Self::UnknownParameterBinding => f.write_str("same_parameter=false; expected declared same-parameter binding before independent coefficient proof"),
            Self::Exact(cause) => cause.fmt(f),
        }
    }
}
