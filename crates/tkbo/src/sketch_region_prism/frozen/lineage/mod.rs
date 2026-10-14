// SPDX-License-Identifier: GPL-2.0-only
//! ADR0032 observed historical lineage; no region proof or successful body is published.
mod capture;
mod directed;
mod members;
mod names;
mod sweep;
mod types;
mod views;
use super::super::{PrismBodyContext, PrismCopyObservation, PrismReceipt};
use super::*;
pub(crate) use capture::inspect_prism_lineage;
use rustycad_tkbrep::topods::Shape;
pub(crate) use types::*;

pub(super) struct LineageInputs<'a> {
    receipt: &'a PrismReceipt,
    publication: &'a PrismPublicationBindings,
    authored: &'a PrismCopyBindings,
}
impl LineageInputs<'_> {
    fn failure(&self, phase: CopyLineagePhase, cause: CopyLineageCause) -> CopyLineageFinding {
        CopyLineageFinding {
            operation: self.publication.operation,
            phase,
            entity: None,
            cause,
        }
    }
    fn count(
        &self,
        phase: CopyLineagePhase,
        required: &'static str,
        actual: usize,
        expected: usize,
    ) -> CopyLineageFinding {
        self.failure(
            phase,
            CopyLineageCause::Count {
                required,
                actual,
                expected,
            },
        )
    }
    fn require(
        &self,
        phase: CopyLineagePhase,
        required: &'static str,
        actual: usize,
        expected: usize,
    ) -> LineageCheck<()> {
        if actual != expected {
            return Err(self.count(phase, required, actual, expected));
        }
        Ok(())
    }
    fn frozen(&self, phase: CopyLineagePhase, cause: PrismFreezeCause) -> CopyLineageFinding {
        self.failure(phase, CopyLineageCause::Frozen(Box::new(cause)))
    }
    fn limit(&self, quantity: &'static str, actual: usize) -> LineageCheck<()> {
        super::domain::check_count(actual).map_err(|cause| {
            let cause = match cause {
                PrismFreezeCause::LimitExceeded {
                    actual, maximum, ..
                } => PrismFreezeCause::LimitExceeded {
                    quantity,
                    actual,
                    maximum,
                },
                other => other,
            };
            self.frozen(CopyLineagePhase::Members, cause)
        })
    }
    fn observations(&self) -> LineageCheck<&[PrismCopyObservation]> {
        self.receipt
            .copied
            .as_ref()
            .map(|c| c.observations.as_slice())
            .ok_or_else(|| self.count(CopyLineagePhase::Members, "copied selection", 0, 1))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod isolation_tests;
#[cfg(test)]
mod refusal_tests;
#[cfg(test)]
mod replay_tests;

#[cfg(test)]
mod member_identity_tests;
