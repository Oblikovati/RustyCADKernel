// SPDX-License-Identifier: GPL-2.0-only
//! Identity equality preserves every occurrence count, independently of traversal order.
pub(super) fn same_multiset<T: PartialEq>(first: &[T], second: &[T]) -> bool {
    // A duplicate must never replace a different retained descendant, query context or parent.
    first.len() == second.len()
        && first.iter().chain(second).all(|key| {
            first.iter().filter(|row| *row == key).count()
                == second.iter().filter(|row| *row == key).count()
        })
}

#[cfg(test)]
mod tests {
    use super::same_multiset;

    #[test]
    fn multiplicity_is_symmetric_and_independent_of_order() {
        assert!(same_multiset(&[1, 2, 1], &[2, 1, 1]));
        assert!(same_multiset::<u8>(&[], &[]));
        assert!(!same_multiset(&[1, 1], &[1, 2]));
        assert!(!same_multiset(&[1, 2], &[1, 1]));
        assert!(!same_multiset(&[1], &[1, 1]));
    }
}
