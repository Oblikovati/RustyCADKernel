//! Fill construction follows Resize's initial growth capacity, including one slot -> two.
use super::{LinearVector, relocation::next_capacity_for};

impl<T: Clone> LinearVector<T> {
    pub(super) fn construct_fill(size: usize, value: &T) -> Self {
        let capacity = if size == 0 {
            0
        } else {
            next_capacity_for(0, size, Self::max_size())
        };
        Self::construct_owned(capacity, |destination| {
            destination.construct_unpublished_tail(size, || value.clone())
        })
    }
}

#[cfg(test)]
mod tests;
