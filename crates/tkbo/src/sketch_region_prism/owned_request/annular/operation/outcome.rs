// SPDX-License-Identifier: GPL-2.0-only
//! The owned one-call outcome: a published named annular prism with its validation coverage,
//! history counts, per-face certified areas and physical volume, or a typed refusal. No live shape,
//! operation-local failure or process address is reachable from either branch.
use super::*;
use crate::sketch_region_prism::frozen::FrozenSweepRole;
use crate::sketch_region_prism::frozen::ordered_validation::OrderedPrismStageStatus;

pub(in crate::sketch_region_prism) enum OwnedAnnularPrismOutcome {
    Published(Box<OwnedAnnularPrism>),
    Refused(Box<OwnedAnnularPrismRefusal>),
}
/// Caller-qualified face name; derived from the issued dictionary, never from face order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularFaceName {
    /// A cap of the caller's annulus region at the first or last sweep end.
    Cap { region: u64, sweep: FrozenSweepRole },
    /// The side swept from one caller boundary and its caller annulus wire.
    Side {
        boundary: CallerBoundaryId,
        wire: u64,
    },
}
/// A certified enclosure with its certified absolute error; never a sampled or mesh estimate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct CertifiedQuantity {
    pub enclosure: [f64; 2],
    pub absolute_error_bound: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct OwnedAnnularFace {
    pub name: AnnularFaceName,
    pub area: CertifiedQuantity,
}
/// Physical volume from the exact-sign transport of the certified sweep integral.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct OwnedAnnularVolume {
    pub volume: CertifiedQuantity,
    pub representative: f64,
    pub jacobian_sign: i8,
}
/// Counts of the complete issued history; full relations are in `publication_replay`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct OwnedAnnularHistorySummary {
    pub continued_originals: usize,
    pub retired_wires: usize,
    pub deleted_originals: usize,
}
pub(in crate::sketch_region_prism) struct OwnedAnnularPrism {
    pub(super) request: OwnedAnnularPrismRequest,
    pub(super) faces: Vec<OwnedAnnularFace>,
    pub(super) total_area: CertifiedQuantity,
    pub(super) volume: OwnedAnnularVolume,
    pub(super) validation: [OrderedPrismStageStatus; 4],
    pub(super) history: OwnedAnnularHistorySummary,
    pub(super) dictionary: AnnularNameDictionary,
    pub(super) publication_replay: Vec<u8>,
    pub(super) volume_replay: Vec<u8>,
}
/// Typed refusal diagnostic; borrowed child families that cannot leave the call are explicit gaps.
pub(in crate::sketch_region_prism) enum AnnularOperationDiagnostic {
    Preparation(Box<OwnedAnnularPreparationDiagnostic>),
    Publication(Box<OwnedAnnularPublicationDiagnostic>),
    Volume(Box<AnnularVolumeFailure>),
    Body(AnnularBodyCause),
    /// The child refusal borrows operation-local state; only its stage and class are owned.
    Unprojected,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum AnnularBodyCause {
    UnresolvedFace {
        source: u64,
    },
    UnmeasuredFace {
        source: u64,
    },
    Count {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularOperationGap {
    /// The stage's original child failure was dropped with the call's local state.
    StageFailure(AnnularOperationStage),
    /// The owned child diagnostic itself records explicit gaps.
    ChildDiagnostic(AnnularOperationStage),
}
pub(in crate::sketch_region_prism) struct OwnedAnnularPrismRefusal {
    pub(super) request: OwnedAnnularPrismRequest,
    pub(super) stage: AnnularOperationStage,
    pub(super) class: PrismDiagnosticClass,
    pub(super) diagnostic: AnnularOperationDiagnostic,
    pub(super) stop: Option<AnnularOperationStop>,
    pub(super) gaps: Vec<AnnularOperationGap>,
}
impl OwnedAnnularPrism {
    /// Read the original owned request; `prism.request().regions.annulus`.
    pub(in crate::sketch_region_prism) fn request(&self) -> &OwnedAnnularPrismRequest {
        &self.request
    }
    /// Read every named face with its certified area; `prism.faces().len() == 4`.
    pub(in crate::sketch_region_prism) fn faces(&self) -> &[OwnedAnnularFace] {
        &self.faces
    }
    /// Read the certified total boundary area; `prism.total_area()`.
    pub(in crate::sketch_region_prism) fn total_area(&self) -> CertifiedQuantity {
        self.total_area
    }
    /// Read the certified physical volume; `prism.volume().volume.enclosure`.
    pub(in crate::sketch_region_prism) fn volume(&self) -> OwnedAnnularVolume {
        self.volume
    }
    /// Read ordered validation coverage (topology, geometry, self-intersection, tolerance).
    pub(in crate::sketch_region_prism) fn validation(&self) -> [OrderedPrismStageStatus; 4] {
        self.validation
    }
    /// Read complete history counts; `prism.history().continued_originals`.
    pub(in crate::sketch_region_prism) fn history(&self) -> OwnedAnnularHistorySummary {
        self.history
    }
    /// Read the caller dictionary of every issued token; `prism.dictionary().meaning(token)`.
    pub(in crate::sketch_region_prism) fn dictionary(&self) -> &AnnularNameDictionary {
        &self.dictionary
    }
    /// Full publication receipt (join, validation, history, measures); no addresses.
    pub(in crate::sketch_region_prism) fn publication_replay(&self) -> &[u8] {
        &self.publication_replay
    }
    /// Full volume receipt; `prism.volume_replay()`.
    pub(in crate::sketch_region_prism) fn volume_replay(&self) -> &[u8] {
        &self.volume_replay
    }
}
impl OwnedAnnularPrismRefusal {
    /// Read the original owned request; `refusal.request()`.
    pub(in crate::sketch_region_prism) fn request(&self) -> &OwnedAnnularPrismRequest {
        &self.request
    }
    /// Read the refusing stage; `refusal.stage()`.
    pub(in crate::sketch_region_prism) fn stage(&self) -> AnnularOperationStage {
        self.stage
    }
    /// Read the unchanged child class; `refusal.class()`.
    pub(in crate::sketch_region_prism) fn class(&self) -> PrismDiagnosticClass {
        self.class
    }
    /// Read the owned typed diagnostic; `refusal.diagnostic()`.
    pub(in crate::sketch_region_prism) fn diagnostic(&self) -> &AnnularOperationDiagnostic {
        &self.diagnostic
    }
    /// Read the recorded caller stop, if any; `refusal.stop()`.
    pub(in crate::sketch_region_prism) fn stop(&self) -> Option<&AnnularOperationStop> {
        self.stop.as_ref()
    }
    /// Read explicit projection gaps; empty means the refusal is owned losslessly.
    pub(in crate::sketch_region_prism) fn gaps(&self) -> &[AnnularOperationGap] {
        &self.gaps
    }
}
