// SPDX-License-Identifier: GPL-2.0-only
use super::{errors::*, joined_orientation::*};
use crate::sketch_region_prism::region_certificate::orientation::PrismOrientationEvaluationCause;

pub(super) fn orientation_stop(failure: JoinedPrismOrientationFailure) -> OrderedStop {
    let class = orientation_class(&failure.cause);
    OrderedStop {
        class,
        cause: OrderedPrismCause::Orientation(Box::new(failure)),
    }
}
pub(super) fn orientation_class(cause: &JoinedPrismOrientationCause) -> OrderedPrismFailureClass {
    use JoinedPrismOrientationCause as C;
    use OrderedPrismFailureClass as F;
    match cause {
        C::Cancelled => F::Cancelled,
        C::Named { cause, .. } => orientation_class(cause),
        C::Direction { .. } => F::Failed,
        C::Orientation(error) => evaluation_class(&error.cause),
        C::Version { .. } | C::Count { .. } | C::Interval { .. } | C::ParentMultiplicity { .. } => {
            F::Incomplete
        }
    }
}
fn evaluation_class(cause: &PrismOrientationEvaluationCause) -> OrderedPrismFailureClass {
    use OrderedPrismFailureClass as F;
    use PrismOrientationEvaluationCause as C;
    match cause {
        C::Unsupported { .. } => F::Incomplete,
        C::Winding { .. } | C::ZeroJacobian | C::Face { .. } | C::Trim { .. } => F::Failed,
    }
}
