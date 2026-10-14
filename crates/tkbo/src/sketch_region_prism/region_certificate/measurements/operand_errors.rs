// SPDX-License-Identifier: GPL-2.0-only
//! The captured scalar kernel cannot return live geometry or mutable diagnostics.
use super::*;

#[derive(Debug)]
pub(crate) enum ScalarOperandCause {
    Missing {
        hypothesis: &'static str,
    },
    Condition {
        name: &'static str,
        actual: usize,
        expected: usize,
    },
}
impl ScalarOperandCause {
    pub(super) fn need<T>(value: Option<T>, hypothesis: &'static str) -> Result<T, Self> {
        value.ok_or(Self::Missing { hypothesis })
    }
    pub(super) fn holds(name: &'static str, condition: bool) -> Result<(), Self> {
        if !condition {
            return Err(Self::Condition {
                name,
                actual: 0,
                expected: 1,
            });
        }
        Ok(())
    }
}
impl From<ScalarOperandCause> for RegionCertificateCause {
    fn from(cause: ScalarOperandCause) -> Self {
        match cause {
            ScalarOperandCause::Missing { hypothesis } => Self::MissingEvidence { hypothesis },
            ScalarOperandCause::Condition {
                name,
                actual,
                expected,
            } => Self::Hypothesis {
                name,
                actual,
                expected,
            },
        }
    }
}
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
