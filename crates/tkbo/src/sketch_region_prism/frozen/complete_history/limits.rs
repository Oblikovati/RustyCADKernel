// SPDX-License-Identifier: GPL-2.0-only
//! Requested bounds are refused, never silently clamped or used to truncate evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrismHistoryLimits {
    pub maximum_depth: usize,
    pub maximum_uses: usize,
    pub maximum_rows: usize,
}
#[derive(Debug)]
pub(crate) enum HistoryBoundCause {
    Setting {
        quantity: &'static str,
        actual: usize,
        minimum: usize,
        maximum: usize,
    },
    Exhausted {
        quantity: &'static str,
        actual: usize,
        maximum: usize,
    },
    AncestorCycle {
        ancestor_depth: usize,
        repeated_depth: usize,
    },
    Native(rustycad_tkernel::standard::Failure),
}
impl PrismHistoryLimits {
    /// Explicit small-operation limits; example: `PrismHistoryLimits::bounded_v1()`.
    pub(crate) fn bounded_v1() -> Self {
        Self {
            maximum_depth: 32,
            maximum_uses: 4096,
            maximum_rows: 16384,
        }
    }
    pub(in crate::sketch_region_prism::frozen) fn admit(self) -> Result<Self, HistoryBoundCause> {
        for (quantity, actual, maximum) in [
            ("ancestor depth", self.maximum_depth, 64),
            ("directed uses", self.maximum_uses, 16384),
            ("history rows", self.maximum_rows, 65536),
        ] {
            if actual == 0 || actual > maximum {
                return Err(HistoryBoundCause::Setting {
                    quantity,
                    actual,
                    minimum: 1,
                    maximum,
                });
            }
        }
        Ok(self)
    }
    pub(in crate::sketch_region_prism::frozen) fn count(
        self,
        quantity: &'static str,
        actual: usize,
        maximum: usize,
    ) -> Result<(), HistoryBoundCause> {
        if actual > maximum {
            return Err(HistoryBoundCause::Exhausted {
                quantity,
                actual,
                maximum,
            });
        }
        Ok(())
    }
}
