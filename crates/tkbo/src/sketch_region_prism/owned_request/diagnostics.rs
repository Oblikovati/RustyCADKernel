// SPDX-License-Identifier: GPL-2.0-only
//! Private retained state, including mutable local diagnostics; never an application result.
use super::{admission::*, construction::*, names::*, progress::*, request::*};
use crate::sketch_region_certification::{BoundaryCensus, BoundaryInputs, CensusCause};
use crate::sketch_region_contact_transport::{ContactTransportStop, PairedContactTransaction};
use crate::sketch_region_partition_bindings::PartitionBoundOutputs;
use crate::sketch_region_prism::{PrismReceipt, region_certificate::RegionPrismUnavailable};
use rustycad_tkernel::standard::Failure;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OwnedPreparationClass {
    Declined,
    Incomplete,
    Failed,
    Cancelled,
}
pub(crate) enum OwnedPreparationCause {
    Admission(OwnedPrismAdmissionCause),
    Native(Failure),
    Census(Box<CensusCause>),
    Contact(ContactTransportStop),
    PrismUnavailable(Box<RegionPrismUnavailable>),
    StageUnavailable { class: OwnedPreparationClass },
    MissingLocalState { expected: &'static str },
    Callback,
}
pub(super) struct PreparationRetained {
    pub request: OwnedPolynomialPrismRequest,
    pub plan: Option<SingleBoundaryNamePlan>,
    pub source: Option<ConstructedPolynomialSource>,
    pub inputs: Option<BoundaryInputs>,
    pub census: Option<BoundaryCensus>,
    pub partition: Option<PartitionBoundOutputs>,
    pub bridge: Option<PairedContactTransaction>,
    pub prism: Option<PrismReceipt>,
    pub phase: OwnedPrismPhase,
}
/// A prepared diagnostic stack is not a published body; no successful operation accessor exists.
pub(crate) struct PreparedPolynomialPrism {
    pub(super) retained: PreparationRetained,
}
/// Actual child causes are retained privately for later owned refusal projection.
pub(crate) struct OwnedPrismPreparationFailure {
    pub class: OwnedPreparationClass,
    pub cause: OwnedPreparationCause,
    pub callback: Option<OwnedPrismCallbackStop>,
    pub(super) retained: PreparationRetained,
}
impl PreparationRetained {
    pub(super) fn new(request: OwnedPolynomialPrismRequest) -> Self {
        Self {
            request,
            plan: None,
            source: None,
            inputs: None,
            census: None,
            partition: None,
            bridge: None,
            prism: None,
            phase: OwnedPrismPhase::Admission,
        }
    }
    pub(super) fn source(&self) -> Result<&ConstructedPolynomialSource, OwnedPreparationCause> {
        local(self.source.as_ref(), "constructed polynomial source")
    }
}

pub(super) fn local<T>(
    value: Option<T>,
    expected: &'static str,
) -> Result<T, OwnedPreparationCause> {
    value.ok_or(OwnedPreparationCause::MissingLocalState { expected })
}

impl From<CensusCause> for OwnedPreparationCause {
    fn from(cause: CensusCause) -> Self {
        Self::Census(Box::new(cause))
    }
}
