// SPDX-License-Identifier: GPL-2.0-only
//! Typed owned summary plus original local diagnostic when its owners cannot be projected.
use super::request::*;
use crate::sketch_region::RegionSourceId;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::{
    bezier_line::BezierWireError, bezier_pair::*, metric_bounds::ResidualError,
};

pub(crate) struct TwoLoopRegionFailure<'a> {
    pub operation: u64,
    pub stage: TwoLoopRegionStage,
    pub entities: Vec<TwoLoopEntity>,
    pub requested: TwoLoopRegionOptions,
    pub retained_units: usize,
    pub cause: TwoLoopRegionCause,
    pub local: Option<TwoLoopLocalDiagnostic<'a>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TwoLoopEntity {
    Source(RegionSourceId),
    Candidate(u64),
    Wire(u64),
}
#[derive(Debug)]
pub(crate) enum TwoLoopRegionCause {
    Missing {
        hypothesis: &'static str,
    },
    Count {
        hypothesis: &'static str,
        actual: usize,
        expected: usize,
    },
    Limit {
        operand: &'static str,
        requested: usize,
        measured: usize,
    },
    Overflow {
        operand: &'static str,
    },
    Stale {
        operand: &'static str,
    },
    Native(Failure),
    Capture(std::sync::Arc<super::super::super::surface_charts::PrismSurfaceCause>),
    Numerical(BezierWireError),
    CandidateSignature {
        actual: [i64; 2],
        wires: usize,
    },
    Winding {
        cause: BezierWireError,
        work: rustycad_tkmath::predicates::roots::bezier_line::CaptureWorkReceipt,
    },
    WindingLocation {
        source: RegionSourceId,
        seed: [f64; 2],
        actual: rustycad_tkmath::predicates::roots::bezier_line::BezierWireLocation,
        expected: &'static str,
    },
    Residual(ResidualError),
    PairCapture(Box<BezierPairCaptureFailure>),
    PairRefusal {
        completion: BezierPairCompletion,
        policy: BezierPairPolicy,
        work: BezierPairWork,
    },
    UnsupportedDiagnostic {
        stage: &'static str,
    },
    Cancelled,
    Geometry(std::sync::Arc<crate::sketch_region_geometry_capture::CapturedGeometryCause>),
    SourceRefusal {
        stage: crate::sketch_region_source_conformance::ConstructionStage,
    },
    Kind {
        actual: rustycad_tkg3d::top_abs::ShapeEnum,
        expected: rustycad_tkg3d::top_abs::ShapeEnum,
    },
    Scalar {
        operand: &'static str,
        actual: f64,
        expected: &'static str,
    },
    ResidualMismatch(rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision),
}
pub(crate) enum TwoLoopLocalDiagnostic<'a> {
    Arrangement(&'a crate::sketch_region::ArrangementCompletion),
    PartitionObservation(&'a crate::sketch_region::PartitionObservation),
    Copy(&'a crate::sketch_region_candidates::CandidateCopyInspection),
    Report {
        gravity: rustycad_tkernel::message::Gravity,
        report: &'a rustycad_tkernel::message::Report,
    },
    Census(&'a crate::sketch_region_certification::BoundaryCensus),
    Partition {
        cause: Box<super::super::RegionCertificateCause>,
        inspection: Box<crate::sketch_region_partition_bindings::PartitionTransactionInspection>,
    },
    Pair(&'a BezierPairReport),
    Candidate(&'a crate::sketch_region_candidates::CandidateCause),
    Region(Box<super::super::RegionCertificateCause>),
    Source(&'a crate::sketch_region_source_conformance::SourceConformanceCause),
    SourceBinding(Vec<crate::sketch_region_source_conformance::SourceBindingFault>),
}

impl<'a> TwoLoopRegionFailure<'a> {
    pub(super) fn at(mut self: Box<Self>, entity: TwoLoopEntity) -> Box<Self> {
        if !self.entities.contains(&entity) {
            self.entities.push(entity);
        }
        self
    }
}
