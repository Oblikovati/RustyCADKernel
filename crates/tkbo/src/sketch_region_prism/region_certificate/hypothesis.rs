// SPDX-License-Identifier: GPL-2.0-only
//! Owned exact operand admission; legacy adapters must not enlarge this pure failure vocabulary.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RegionHypothesisCause {
    Missing {
        hypothesis: &'static str,
    },
    Condition {
        name: &'static str,
        actual: usize,
        expected: usize,
    },
}
pub(crate) type HypothesisCheck<T = ()> = Result<T, RegionHypothesisCause>;

/// Require an exact count; `require("unique named side", observed, 1)` retains both values.
pub(crate) fn require(name: &'static str, actual: usize, expected: usize) -> HypothesisCheck {
    if actual != expected {
        return Err(RegionHypothesisCause::Condition {
            name,
            actual,
            expected,
        });
    }
    Ok(())
}

/// Record a discrete operand predicate; `holds("matching source", matches)` never applies a tolerance.
pub(crate) fn holds(name: &'static str, condition: bool) -> HypothesisCheck {
    let actual = usize::from(condition);
    require(name, actual, 1)
}

/// Refuse absent evidence; `need(observed_height, "side height")` never invents a fallback.
pub(crate) fn need<T>(value: Option<T>, hypothesis: &'static str) -> HypothesisCheck<T> {
    let missing = RegionHypothesisCause::Missing { hypothesis };
    value.ok_or(missing)
}
