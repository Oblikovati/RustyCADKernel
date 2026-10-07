//! Native grow() callback phases with explicit move_if_noexcept policy selection.
use super::LinearVector;

pub(super) fn next_capacity_for(current: usize, minimum: usize, maximum: usize) -> usize {
    let doubled = if current == 0 {
        2
    } else {
        current.saturating_mul(2)
    };
    doubled.min(maximum).max(minimum)
}

impl<T> LinearVector<T> {
    /// Reserves while preserving native nontrivial relocation callbacks; e.g.
    /// `vector.reserve_with_relocation(8, |item| std::mem::take(item))`.
    /// Select a move callback for native nonthrowing moves or a copy callback
    /// when `move_if_noexcept` selects copying. Every old value stays alive
    /// until all new values are constructed, then old values drop in order.
    /// A relocation panic keeps the receiver's length and capacity; the replacement
    /// allocation is freed (the native leaked replacement is D1 class B2).
    pub fn reserve_with_relocation<F>(&mut self, minimum: usize, relocate: F)
    where
        F: FnMut(&mut T) -> T,
    {
        self.reserve_relocation_owned(minimum, relocate);
    }

    pub(super) fn reserve_relocation_owned(
        &mut self,
        minimum: usize,
        mut relocate: impl FnMut(&mut T) -> T,
    ) {
        if minimum <= self.capacity() {
            return;
        }
        let capacity = next_capacity_for(self.capacity(), minimum, Self::max_size());
        let replacement = Self::construct_owned(capacity, |destination| {
            self.relocate_unpublished_into(destination, &mut relocate)
        });
        self.elements = replacement.elements;
    }

    pub(super) fn relocate_unpublished_into(
        &mut self,
        destination: &mut Self,
        mut relocate: impl FnMut(&mut T) -> T,
    ) {
        let mut original = self.elements.iter_mut();
        destination.construct_unpublished_tail(original.len(), || {
            relocate(
                original
                    .next()
                    .expect("relocation slot: expected live source"),
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::next_capacity_for;

    #[test]
    fn native_next_capacity_clamps_doubling_then_honors_minimum() {
        let expected =
            include_str!("../../../../../tools/oracle/tkernel_linear_relocation.expected");
        for row in expected
            .lines()
            .filter(|line| line.starts_with("capacity:"))
        {
            let values: Vec<usize> = row.split(':').skip(1).map(|s| s.parse().unwrap()).collect();
            assert_eq!(
                next_capacity_for(values[0], values[1], values[2]),
                values[3]
            );
        }
    }
}

#[cfg(test)]
#[path = "growth_relocation_tests.rs"]
mod relocation_tests;
