// SPDX-License-Identifier: GPL-2.0-only
//! The captured scalar kernel cannot return live geometry or mutable diagnostics.
use super::*;

pub(crate) use super::super::hypothesis::RegionHypothesisCause as ScalarOperandCause;
pub(super) use super::super::hypothesis::{holds as scalar_holds, need as scalar_need};

#[derive(Debug)]
pub(crate) enum PrismScalarFailure {
    Operand(ScalarOperandCause),
    Cap {
        role: PrismImageRole,
        cause: PolynomialPlaneError,
    },
    Sweep(PolynomialIntegralError),
}
impl From<ScalarOperandCause> for PrismScalarFailure {
    fn from(cause: ScalarOperandCause) -> Self {
        Self::Operand(cause)
    }
}
impl From<PrismScalarFailure> for PrismMeasureFailure {
    fn from(cause: PrismScalarFailure) -> Self {
        match cause {
            PrismScalarFailure::Operand(cause) => Self::Region(cause.into()),
            PrismScalarFailure::Cap { role, cause } => Self::Cap { role, cause },
            PrismScalarFailure::Sweep(cause) => Self::Sweep(cause),
        }
    }
}
pub(super) type ScalarCheck<T> = Result<T, PrismScalarFailure>;
