// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularMeasurePhase {
    Admission,
    Caps,
    Sweep,
    Sides,
    Total,
    Commit,
}
#[derive(Debug)]
pub(in crate::sketch_region_prism) enum AnnularMeasureCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Count {
        quantity: &'static str,
        actual: usize,
        expected: usize,
    },
    Operand {
        required: &'static str,
    },
    Direction(AnnularDirectionCause),
    OriginalSidePolicy(PolynomialSideFailure),
    Cap(PolynomialPlaneError),
    Sweep(PolynomialIntegralError),
    Side(Box<PolynomialSideError>),
    Total(EnclosureSumError),
    WorkOverflow {
        completed: u32,
        additional: u32,
    },
    Cancelled,
}
#[derive(Debug)]
pub(in crate::sketch_region_prism) struct AnnularMeasureFailure {
    pub operation: AuthoredPrismId,
    pub phase: AnnularMeasurePhase,
    pub entity: Option<FrozenUseKey>,
    pub requested: AnnularMeasureOptions,
    pub binding: Box<[u8]>,
    pub partial: AnnularMeasurePartial,
    pub cause: AnnularMeasureCause,
}
impl AnnularMeasureFailure {
    /// Preserve actual arithmetic cancellation; `failure.classification()` grants no validity.
    pub(in crate::sketch_region_prism) fn classification(&self) -> JoinedMeasureFailureClass {
        if cancelled(&self.cause) {
            return JoinedMeasureFailureClass::Cancelled;
        }
        JoinedMeasureFailureClass::Incomplete
    }
}
fn cancelled(cause: &AnnularMeasureCause) -> bool {
    match cause {
        AnnularMeasureCause::Cancelled => true,
        AnnularMeasureCause::Cap(error) => plane_cancelled(&error.cause),
        AnnularMeasureCause::Sweep(error) => integral_cancelled(&error.cause),
        AnnularMeasureCause::Side(error) => {
            matches!(*error.cause, PolynomialSideFailure::Cancelled)
        }
        AnnularMeasureCause::Total(error) => matches!(error.cause, EnclosureSumCause::Cancelled),
        _ => false,
    }
}
pub(super) fn count(quantity: &'static str, actual: usize, expected: usize) -> MeasureCheck<()> {
    if actual == expected {
        return Ok(());
    }
    Err(AnnularMeasureCause::Count {
        quantity,
        actual,
        expected,
    })
}
pub(super) fn operand(required: &'static str) -> AnnularMeasureCause {
    AnnularMeasureCause::Operand { required }
}
