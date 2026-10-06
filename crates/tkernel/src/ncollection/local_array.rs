//! Inline storage adapter retaining NCollection_LocalArray allocation semantics.
use smallvec::SmallVec;

mod nontrivial_move;
mod pointers;
mod trivial_move;

/// Contiguous storage with `N` inline elements before allocation moves to the heap.
///
/// ```
/// use rustycad_tkernel::ncollection::LocalArray;
/// let array = LocalArray::<i32, 8>::default();
/// assert!(array.as_slice().is_empty());
/// ```
pub struct LocalArray<T, const N: usize = 1024> {
    elements: SmallVec<[T; N]>,
}

impl<T, const N: usize> Default for LocalArray<T, N> {
    fn default() -> Self {
        const { assert!(N > 0, "inline capacity must be greater than zero") };
        Self {
            elements: SmallVec::new(),
        }
    }
}

impl<T, const N: usize> LocalArray<T, N> {
    /// Returns the logical size; `LocalArray::<i32>::default().len() == 0`.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Reports an empty logical range; `LocalArray::<i32>::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Borrows the initialized elements; `LocalArray::<i32>::default().as_slice()`.
    pub fn as_slice(&self) -> &[T] {
        self.elements.as_slice()
    }

    /// Iterates over the initialized elements; `LocalArray::<i32>::default().iter().count() == 0`.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.elements.iter()
    }
}

impl<T: Default, const N: usize> LocalArray<T, N> {
    /// Allocates initialized elements; `LocalArray::<i32, 4>::new(3).len() == 3`.
    pub fn new(size: usize) -> Self {
        let mut array = Self::default();
        array.allocate(size);
        array
    }

    /// Allocates without copying when new storage is needed; `array.allocate(8)`.
    pub fn allocate(&mut self, size: usize) {
        self.reallocate(size, false);
    }

    /// Grows or shrinks with OCCT's inline and retention rules; `array.reallocate(16, true)`.
    /// Replacement growth without copying preserves the old array if `Default` panics.
    /// Rust ownership moves have no callbacks; use `reallocate_with_move` when
    /// the native element type has an observable nontrivial move constructor.
    pub fn reallocate(&mut self, size: usize, copy: bool) {
        if size <= self.len() {
            self.truncate_forward(size);
            return;
        }
        let stays_inline = !self.elements.spilled() && size <= N;
        if !copy && !stays_inline {
            self.replace_without_copy(size);
            return;
        }
        if self.elements.spilled() && size <= N {
            self.elements.shrink_to_fit();
        }
        self.elements.resize_with(size, T::default);
    }

    fn replace_without_copy(&mut self, size: usize) {
        let mut replacement = SmallVec::with_capacity(size);
        replacement.resize_with(size, T::default);
        // OCCT publishes the replacement pointer only after construction completes.
        self.truncate_forward(0);
        self.elements = replacement;
    }
}

impl<T, const N: usize> LocalArray<T, N> {
    /// Borrows mutable contiguous elements; `array.as_mut_slice().reverse()`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.elements.as_mut_slice()
    }

    /// Iterates mutably; `array.iter_mut().for_each(|v| *v = 3)`.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.elements.iter_mut()
    }
}

impl<T, const N: usize> std::ops::Index<usize> for LocalArray<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for LocalArray<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.elements[index]
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a LocalArray<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut LocalArray<T, N> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const N: usize> LocalArray<T, N> {
    /// Borrows the buffer as a zero-based array; `array.to_array1()[0] = 42`.
    pub fn to_array1(&mut self) -> super::Array1<'_, T> {
        super::Array1::from_slice(self.as_mut_slice())
    }

    /// A zero-based array viewing the values, as the const native `ToArray1()`; the view
    /// detaches a private copy on its first write; `array.to_array1_view().len()`.
    pub fn to_array1_view(&self) -> super::Array1<'_, T>
    where
        T: Clone,
    {
        super::Array1::from_shared(self.as_slice(), 0)
    }

    /// Transfers storage and leaves the source empty; `dest.move_from(&mut source)`.
    /// Use `move_from_with` to preserve native element move-constructor callbacks.
    pub fn move_from(&mut self, source: &mut Self) -> &mut Self {
        self.deallocate();
        *self = std::mem::take(source);
        self
    }

    /// Preserves the receiver for native self assignment; `array.move_self()`.
    pub fn move_self(&mut self) -> &mut Self {
        self
    }

    /// Releases all elements and returns to inline storage; `array.deallocate()`.
    pub fn deallocate(&mut self) {
        self.truncate_forward(0);
        self.elements = SmallVec::new();
    }
}

impl<T, const N: usize> LocalArray<T, N> {
    #[allow(unsafe_code)]
    fn truncate_forward(&mut self, size: usize) {
        let removed = self.len() - size;
        // OCCT destroys in place in increasing order; SmallVec::truncate reverses
        // that order, and drain moves values before dropping them. Publish the
        // retained length first so slice drop glue also handles a Rust panic.
        // SAFETY: callers only shorten initialized storage. After shortening,
        // the removed tail stays allocated and is owned solely by this slice drop.
        unsafe {
            self.elements.set_len(size);
            let tail = self.elements.as_mut_ptr().add(size);
            std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(tail, removed));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LocalArray;
    use smallvec::SmallVec;

    struct PanickingDefault {
        value: i32,
    }

    impl Default for PanickingDefault {
        fn default() -> Self {
            panic!("replacement default constructor failure");
        }
    }

    #[test]
    fn replacement_growth_keeps_old_values_when_default_panics() {
        let elements = SmallVec::from_vec(vec![PanickingDefault { value: 42 }]);
        let mut array = LocalArray::<PanickingDefault, 2> { elements };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            array.reallocate(3, false);
        }));

        assert!(result.is_err());
        assert_eq!(array.len(), 1);
        assert_eq!(array[0].value, 42);
    }
}
