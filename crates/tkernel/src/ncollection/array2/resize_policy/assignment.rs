use super::Array2;

pub(super) trait ResizeAssignment<T> {
    fn assign(&mut self, target: &mut T, source: &mut Array2<'_, T>, offset: usize);
}

pub(super) struct MutableResizeAssignment<F>(pub(super) F);

impl<T, F: FnMut(&mut T, &mut T)> ResizeAssignment<T> for MutableResizeAssignment<F> {
    fn assign(&mut self, target: &mut T, source: &mut Array2<'_, T>, offset: usize) {
        (self.0)(target, &mut source.as_mut_slice()[offset]);
    }
}

pub(super) struct CloneResizeAssignment;

impl<T: Clone> ResizeAssignment<T> for CloneResizeAssignment {
    fn assign(&mut self, target: &mut T, source: &mut Array2<'_, T>, offset: usize) {
        // The copy fallback does not mutate native donors; shared Rust views need not detach.
        target.clone_from(&source.as_slice()[offset]);
    }
}
