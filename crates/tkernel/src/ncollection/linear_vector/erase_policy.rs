//! Native shiftLeft move-assigns into live slots before destroying the tail.
use super::{ArrayError, LinearVector};

impl<T> LinearVector<T> {
    /// Removes one value through native move-assignment callbacks; e.g.
    /// `vector.erase_with_move(0, |target, source| *target = std::mem::take(source))?`.
    /// Callbacks must leave both slots initialized, including when unwinding.
    pub fn erase_with_move<F>(&mut self, index: usize, assign: F) -> Result<(), ArrayError>
    where
        F: FnMut(&mut T, &mut T),
    {
        self.value(index)?;
        self.erase_range_with_move(index, index + 1, assign)
    }

    /// Removes a half-open range after native forward move assignment; e.g.
    /// `vector.erase_range_with_move(1, 3, |target, source| *target = std::mem::take(source))?`.
    /// An assignment panic retains the original length and all completed mutations.
    pub fn erase_range_with_move<F>(
        &mut self,
        from: usize,
        to: usize,
        mut assign: F,
    ) -> Result<(), ArrayError>
    where
        F: FnMut(&mut T, &mut T),
    {
        if from >= to || to > self.len() {
            return Err(self.index_error(to));
        }
        self.shift_left_with_assignment(from, to, &mut assign);
        self.elements.truncate(self.len() - (to - from));
        Ok(())
    }

    fn shift_left_with_assignment<F>(&mut self, from: usize, to: usize, assign: &mut F)
    where
        F: FnMut(&mut T, &mut T),
    {
        for offset in 0..self.len() - to {
            let (left, right) = self.elements.split_at_mut(to + offset);
            assign(&mut left[from + offset], &mut right[0]);
        }
    }
}
