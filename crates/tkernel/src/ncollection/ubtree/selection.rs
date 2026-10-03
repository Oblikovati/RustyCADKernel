//! Fallible selection preserves native callback exceptions without panicking.
use super::{BoundingVolume, TreeSelector, UBTree};

/// A selector whose acceptance may fail; `tree.try_select(&mut selector)?`.
pub trait FallibleTreeSelector<T, B> {
    /// The callback error propagated unchanged to the caller.
    type Error;
    /// Reject an entire branch; `selector.reject(&bounds)`.
    fn reject(&self, bounds: &B) -> bool;
    /// Accept one leaf or abort traversal; `selector.accept(&object)?`.
    fn accept(&mut self, object: &T) -> Result<bool, Self::Error>;
    /// Stop after a successful callback; `selector.stop()`.
    fn stop(&self) -> bool;
}
struct FallibleSelection<'a, S, E> {
    selector: &'a mut S,
    error: Option<E>,
}
impl<T, B, S: FallibleTreeSelector<T, B>> TreeSelector<T, B>
    for FallibleSelection<'_, S, S::Error>
{
    fn reject(&self, bounds: &B) -> bool {
        self.selector.reject(bounds)
    }
    fn accept(&mut self, object: &T) -> bool {
        match self.selector.accept(object) {
            Ok(accepted) => accepted,
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }
    fn stop(&self) -> bool {
        self.error.is_some() || self.selector.stop()
    }
}
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    /// Traverse in native order and return callback errors; `tree.try_select(&mut selector)?`.
    pub fn try_select<S: FallibleTreeSelector<T, B>>(
        &self,
        selector: &mut S,
    ) -> Result<i32, S::Error> {
        let mut adapter = FallibleSelection {
            selector,
            error: None,
        };
        let count = self.select(&mut adapter);
        match adapter.error {
            Some(error) => Err(error),
            None => Ok(count),
        }
    }
}
