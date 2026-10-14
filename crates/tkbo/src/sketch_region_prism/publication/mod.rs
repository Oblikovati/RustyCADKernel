// SPDX-License-Identifier: GPL-2.0-only
//! ADR0041 complete-operation publication seam; legacy diagnostic receipts remain unchanged.
#[cfg(test)]
mod baseline_tests;
mod progress;
mod source;
mod stage_error;
mod stages;

mod failure;
mod history;
mod operation;
mod replay;
mod request;
mod result;
pub(crate) use operation::{publish_named_prism, publish_named_prism_v2};
pub(crate) use progress::{
    PrismPublicationCheckpoint, PrismPublicationPhase, PrismPublicationProgress,
};
pub(crate) use request::*;
pub(crate) use result::*;

#[cfg(test)]
mod fixture;
#[cfg(test)]
mod operation_tests;

#[cfg(test)]
mod domain_tests;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod mutation_tests;
#[cfg(test)]
mod replay_tests;
pub(crate) use progress::{PublicationCallbackCause, PublicationCallbackStop};
pub(crate) use source::PrismPublicationSource;
pub(crate) use stage_error::{
    PrismPublicationClass, PublicationStageCause, PublicationStageFailure,
};
#[cfg(test)]
mod diagnostic_baseline_tests;
#[cfg(test)]
mod operation_v2_tests;
#[cfg(test)]
mod refusal_tests;

#[cfg(test)]
#[path = "../owned_request/names/body_map/publication_tests.rs"]
mod semantic_map_tests;
