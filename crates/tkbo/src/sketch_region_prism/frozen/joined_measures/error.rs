// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkmath::predicates::roots::RootIsolationError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedMeasureCheckpoint {
    Begin,
    Finish,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedMeasureFailureClass {
    Incomplete,
    Cancelled,
}
#[derive(Debug)]
pub(crate) struct JoinedPrismMeasureFailure {
    pub operation: AuthoredPrismId,
    pub entities: Vec<FrozenUseKey>,
    pub requested: JoinedPrismMeasureOptions,
    pub class: JoinedMeasureFailureClass,
    pub cause: JoinedPrismMeasureCause,
}
#[derive(Debug)]
pub(crate) enum JoinedPrismMeasureCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Count {
        subject: &'static str,
        actual: usize,
        expected: usize,
    },
    Extrusion {
        role: PrismImageRole,
        actual: [u64; 3],
        expected: [u64; 3],
    },
    Cancelled(JoinedMeasureCheckpoint),
    Arithmetic(Box<BoundPrismMeasureFailure>),
}
pub(super) fn failure(
    join: &JoinedPrismProof,
    requested: JoinedPrismMeasureOptions,
    cause: JoinedPrismMeasureCause,
) -> Box<JoinedPrismMeasureFailure> {
    let class = if is_cancelled(&cause) {
        JoinedMeasureFailureClass::Cancelled
    } else {
        JoinedMeasureFailureClass::Incomplete
    };
    let mut entities: Vec<_> = join
        .faces()
        .iter()
        .map(|face| face.occurrence.clone())
        .collect();
    entities.sort();
    Box::new(JoinedPrismMeasureFailure {
        operation: join.projection().operation,
        entities,
        requested,
        class,
        cause,
    })
}
fn is_cancelled(cause: &JoinedPrismMeasureCause) -> bool {
    match cause {
        JoinedPrismMeasureCause::Cancelled(_) => true,
        JoinedPrismMeasureCause::Arithmetic(cause) => arithmetic_cancelled(cause),
        _ => false,
    }
}
fn arithmetic_cancelled(cause: &BoundPrismMeasureFailure) -> bool {
    match cause {
        BoundPrismMeasureFailure::Subset(PrismScalarFailure::Cap { cause, .. }) => {
            plane_cancelled(&cause.cause)
        }
        BoundPrismMeasureFailure::Subset(PrismScalarFailure::Sweep(cause)) => {
            integral_cancelled(&cause.cause)
        }
        BoundPrismMeasureFailure::Side { cause, .. } => {
            matches!(*cause.cause, PolynomialSideFailure::Cancelled)
        }
        BoundPrismMeasureFailure::Total(cause) => {
            matches!(cause.cause, EnclosureSumCause::Cancelled)
        }
        _ => false,
    }
}
fn plane_cancelled(cause: &PolynomialPlaneFailure) -> bool {
    match cause {
        PolynomialPlaneFailure::Boundary(cause) => integral_cancelled(&cause.cause),
        PolynomialPlaneFailure::Metric(cause) => metric_cancelled(cause),
        _ => false,
    }
}
fn integral_cancelled(cause: &PolynomialIntegralFailure) -> bool {
    matches!(cause, PolynomialIntegralFailure::Metric(cause) if metric_cancelled(cause))
}
fn metric_cancelled(cause: &ResidualError) -> bool {
    matches!(cause, ResidualError::Roots(RootIsolationError::Cancelled))
}
