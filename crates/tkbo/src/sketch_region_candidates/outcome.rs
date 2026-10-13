// SPDX-License-Identifier: GPL-2.0-only
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity};
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::bezier_line::BezierWireError;
#[derive(Clone)]
pub(crate) struct CandidateSegment {
    pub occurrence: u64,
    pub edge: Shape,
    pub parents: Vec<OriginalMemberIdentity>,
    pub controls: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
    pub interval: [f64; 2],
    pub original_interval: [f64; 2],
    pub reversed: bool,
}
#[derive(Clone)]
pub(crate) struct CandidateWire {
    pub segments: Vec<CandidateSegment>,
}
pub(crate) struct RegionCandidate {
    pub face: Face,
    pub wires: Vec<CandidateWire>,
    pub relations: Vec<ArrangementRelation>,
}
pub(crate) struct CandidateBatch {
    pub plane: rustycad_tkmath::gp::Plane3,
    pub candidates: Vec<RegionCandidate>,
    pub input_deleted: Vec<OriginalMemberIdentity>,
    pub unrepresented: Vec<ArrangementRelation>,
}
pub(crate) enum CandidateCause {
    Mapped(Failure),
    MappedReport(std::sync::Arc<rustycad_tkernel::message::Report>),
    Incomplete(CandidateGap),
    Cancelled(CandidatePhase),
    Numerical(BezierWireError),
}
pub(crate) enum CandidateGap {
    WorkBudget {
        requested: u16,
        used: u32,
    },
    UnattributedMember {
        member: Shape,
    },
    UnsupportedCurve {
        occurrence: u64,
        dynamic_type: String,
    },
    MissingParents {
        occurrence: u64,
    },
    IncompleteWire {
        omitted: Vec<Shape>,
        unexpected: Vec<Shape>,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidatePhase {
    Copy,
    Generation,
    Capture,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MissingCertificate {
    Construction,
    SharedIncidenceReuse,
    SearchCompleteness,
    SelfIntersection,
    Tolerance,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SeedSelection {
    Empty,
    Boundary { occurrences: Vec<u64> },
    Unique { candidate: usize },
    Ambiguous { candidates: Vec<usize> },
}
pub(crate) enum CandidateClassification {
    Incomplete {
        selection: SeedSelection,
        missing: Vec<MissingCertificate>,
        bisections: u32,
        requested: rustycad_tkmath::predicates::roots::RootIsolationOptions,
        remaining: rustycad_tkmath::predicates::roots::RootIsolationOptions,
        achieved_error: Option<f64>,
    },
}
