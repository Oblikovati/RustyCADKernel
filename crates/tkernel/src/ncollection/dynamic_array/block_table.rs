//! Native block descriptor growth and release share the LinearVector policy.
use super::DynamicArray;

impl<T> Drop for DynamicArray<T> {
    fn drop(&mut self) {
        // OCCT's destructor calls Clear(true); freeing earlier blocks must not precede later Drops.
        self.clear(true);
    }
}

#[cfg(test)]
mod tests;
