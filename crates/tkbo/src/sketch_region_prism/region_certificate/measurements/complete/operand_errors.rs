// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;

#[derive(Debug)]
pub(crate) enum BoundPrismMeasureFailure {
    Subset(PrismScalarFailure),
    SideOperand(ScalarOperandCause),
    Side {
        source: PartitionOccurrenceIdentity,
        output: PartitionOccurrenceIdentity,
        cause: Box<PolynomialSideError>,
    },
    Total(EnclosureSumError),
}
impl From<BoundPrismMeasureFailure> for CompletePrismMeasureFailure {
    fn from(cause: BoundPrismMeasureFailure) -> Self {
        match cause {
            BoundPrismMeasureFailure::Subset(cause) => Self::Subset(Box::new(cause.into())),
            BoundPrismMeasureFailure::SideOperand(cause) => Self::Binding(Box::new(
                PrismBindingFailure::from(RegionCertificateCause::from(cause)),
            )),
            BoundPrismMeasureFailure::Side {
                source,
                output,
                cause,
            } => Self::Side {
                source,
                output,
                cause,
            },
            BoundPrismMeasureFailure::Total(cause) => Self::Total(cause),
        }
    }
}
