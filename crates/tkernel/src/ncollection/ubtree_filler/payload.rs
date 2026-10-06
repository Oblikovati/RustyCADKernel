//! Construction order follows the native declaration, independently from Rust drop order.
use super::ObjBnd;
impl<T, B> ObjBnd<T, B> {
    /// Pair an object with its bound; `ObjBnd::new(1, bounds)`.
    pub fn new(object: T, bounds: B) -> Self {
        Self { object, bounds }
    }
}
impl<T: Clone, B: Clone> ObjBnd<T, B> {
    /// Copies the object before its bounds, as native ObjBnd does; `ObjBnd::new_ref(&1, &bounds)`.
    pub fn new_ref(object: &T, bounds: &B) -> Self {
        Self {
            object: object.clone(),
            bounds: bounds.clone(),
        }
    }
}
impl<T: Clone, B: Clone> Clone for ObjBnd<T, B> {
    fn clone(&self) -> Self {
        Self::new_ref(&self.object, &self.bounds)
    }
    fn clone_from(&mut self, source: &Self) {
        self.object.clone_from(&source.object);
        self.bounds.clone_from(&source.bounds);
    }
}

impl<T: Default, B: Default> Default for ObjBnd<T, B> {
    fn default() -> Self {
        // Explicit initializer order preserves native object-before-bounds construction.
        Self {
            object: T::default(),
            bounds: B::default(),
        }
    }
}
