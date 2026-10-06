//! Native copy-constructor failure has no container destructor to reclaim its raw storage.
use super::LinearVector;

impl<T: Clone> LinearVector<T> {
    pub(super) fn construct_copy(
        &self,
        #[cfg(test)] observe_failure: impl FnOnce(&mut Self),
    ) -> Self {
        Self::construct_owned(
            self.len(),
            |destination| destination.clone_unpublished_from(self, 0),
            #[cfg(test)]
            observe_failure,
        )
    }

    pub(super) fn clone_unpublished_from(&mut self, source: &Self, beginning: usize) {
        let mut index = beginning;
        self.construct_unpublished_tail(source.len(), || {
            let copied = source.elements[index].clone();
            index += 1;
            copied
        });
    }
}

#[cfg(test)]
mod tests;
