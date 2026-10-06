//! Contiguous growable storage matching NCollection_LinearVector.
use super::{Array1, ArrayError};

mod append_policy;
mod borrowed;
mod erase_policy;
mod insert_policy;
mod pointers;
mod relocation;
mod resize_policy;
mod set_value_policy;
mod trivial_assignment;

struct LengthRollback<'a, T> {
    elements: &'a mut Vec<T>,
    previous_len: usize,
    committed: bool,
}

impl<'a, T> LengthRollback<'a, T> {
    fn new(elements: &'a mut Vec<T>, previous_len: usize) -> Self {
        Self {
            elements,
            previous_len,
            committed: false,
        }
    }

    fn commit(&mut self) {
        debug_assert!(!self.committed);
        self.committed = true;
    }
}

impl<T> Drop for LengthRollback<'_, T> {
    fn drop(&mut self) {
        if !self.committed {
            self.elements.truncate(self.previous_len);
        }
    }
}

/// A zero-based growable collection; `LinearVector::<i32>::with_capacity(10)`.
#[derive(Debug)]
pub struct LinearVector<T> {
    elements: Vec<T>,
}
impl<T> Default for LinearVector<T> {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}
impl<T> LinearVector<T> {
    /// Reserves without constructing elements; `LinearVector::<i32>::with_capacity(10).len() == 0`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }
    /// Counts constructed elements; `LinearVector::<i32>::default().len() == 0`.
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    /// Checks whether any elements exist; `LinearVector::<i32>::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    /// Returns available element slots; `LinearVector::<i32>::with_capacity(10).capacity() == 10`.
    pub fn capacity(&self) -> usize {
        self.elements.capacity()
    }
    /// Borrows the contiguous sequence; `vector.as_slice()[0]`.
    pub fn as_slice(&self) -> &[T] {
        &self.elements
    }
    /// Mutates the contiguous sequence; `vector.as_mut_slice().reverse()`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.elements
    }
    /// Iterates constructed values; `vector.iter().count()`.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.elements.iter()
    }
    /// Mutably iterates constructed values; `vector.iter_mut().for_each(|v| *v = 1)`.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.elements.iter_mut()
    }
    /// Ensures capacity using OCCT's doubling rule; `vector.reserve(100)`.
    pub fn reserve(&mut self, minimum: usize) {
        if minimum <= self.capacity() {
            return;
        }
        let grown = relocation::next_capacity_for(self.capacity(), minimum, Self::max_size());
        self.elements.reserve_exact(grown - self.len());
    }
    /// Appends an owned value; `vector.append(42)`.
    pub fn append(&mut self, value: T) -> &mut T {
        self.reserve_next_item();
        self.push_reserved_item(value)
    }
    /// Constructs the next value from arguments captured by a closure; `vector.emplace_append(|| 42)`.
    /// Use `emplace_append_with_relocation` when native growth callbacks are observable.
    pub fn emplace_append(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.reserve_next_item();
        self.push_reserved_item(construct())
    }
    /// Reads a checked zero-based index; `vector.value(0)?`.
    pub fn value(&self, index: usize) -> Result<&T, ArrayError> {
        self.elements
            .get(index)
            .ok_or_else(|| self.index_error(index))
    }
    /// Mutates a checked zero-based index; `*vector.change_value(0)? = 42`.
    pub fn change_value(&mut self, index: usize) -> Result<&mut T, ArrayError> {
        let error = self.index_error(index);
        self.elements.get_mut(index).ok_or(error)
    }
    /// Reads the first value; `vector.first()?`.
    pub fn first(&self) -> Result<&T, ArrayError> {
        self.value(0)
    }
    /// Reads the last value; `vector.last()?`.
    pub fn last(&self) -> Result<&T, ArrayError> {
        self.value(self.len().saturating_sub(1))
    }
    /// Mutates the first value; `*vector.change_first()? = 42`.
    pub fn change_first(&mut self) -> Result<&mut T, ArrayError> {
        self.change_value(0)
    }
    /// Mutates the last value; `*vector.change_last()? = 42`.
    pub fn change_last(&mut self) -> Result<&mut T, ArrayError> {
        self.change_value(self.len().saturating_sub(1))
    }
    /// Removes the last value in its original slot after reducing length; `vector.erase_last()`.
    pub fn erase_last(&mut self) {
        // OCCT 3d097a03 EraseLast reduces Size before destroying the original final slot.
        self.elements.truncate(self.len().saturating_sub(1));
    }
    /// Removes one checked index; `vector.erase(0)?`.
    /// Rust ownership shifts skip element move-assignment callbacks; use `erase_with_move` for those.
    pub fn erase(&mut self, index: usize) -> Result<(), ArrayError> {
        self.value(index)?;
        self.elements.remove(index);
        Ok(())
    }
    /// Removes a nonempty half-open range; `vector.erase_range(1,4)?`.
    /// Use `erase_range_with_move` when native element move assignment is observable.
    pub fn erase_range(&mut self, from: usize, to: usize) -> Result<(), ArrayError> {
        if from >= to || to > self.len() {
            return Err(self.index_error(to));
        }
        self.elements.drain(from..to);
        Ok(())
    }
    /// Inserts before an index, including the end; `vector.insert_before(0,42)?`.
    /// Rust ownership shifts omit native element move callbacks; explicit constructor policies
    /// are available through the unsafe `insert_before_with_constructors` adapter.
    pub fn insert_before(&mut self, index: usize, value: T) -> Result<(), ArrayError> {
        if index > self.len() {
            return Err(self.index_error(index));
        }
        self.reserve(self.len() + 1);
        self.elements.insert(index, value);
        Ok(())
    }
    /// Inserts after an existing index; `vector.insert_after(0,42)?`.
    pub fn insert_after(&mut self, index: usize, value: T) -> Result<(), ArrayError> {
        self.value(index)?;
        self.insert_before(index + 1, value)
    }
    /// Clears values and optionally storage; `vector.clear(false)` retains capacity.
    pub fn clear(&mut self, release: bool) {
        if release {
            self.elements = Vec::new();
            return;
        }
        self.elements.clear();
    }
    /// Transfers storage and resets the source; `dest.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) -> &mut Self {
        // Native assignment clears the receiver before touching the source;
        // extracting first also destroys the source if a Rust destructor panics.
        self.clear(true);
        *self = Self::from_moved(source);
        self
    }

    /// Constructs by transferring the source allocation without element callbacks;
    /// `let moved = LinearVector::from_moved(&mut source)` leaves source empty.
    pub fn from_moved(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Preserves the receiver for native self assignment; `vector.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Borrows a mutable zero-based array view; `vector.to_array1()[0] = 42`.
    pub fn to_array1(&mut self) -> Array1<'_, T> {
        Array1::from_slice(&mut self.elements)
    }
    /// True once storage is allocated, as native `myData != nullptr`: capacity survives
    /// `clear(false)` and is dropped by `clear(true)`; `LinearVector::<i32>::with_capacity(1).has_data()`.
    pub fn has_data(&self) -> bool {
        self.capacity() > 0
    }
    /// The largest representable size, `std::numeric_limits<size_t>::max()`;
    /// `LinearVector::<i32>::max_size() == usize::MAX`.
    pub const fn max_size() -> usize {
        usize::MAX
    }
    fn index_error(&self, index: usize) -> ArrayError {
        ArrayError::Index {
            index: index as i64,
            lower: 0,
            upper: self.len() as i64 - 1,
        }
    }

    fn reserve_next_item(&mut self) {
        let next_size = self.len() + 1;
        self.reserve(next_size);
    }

    fn push_reserved_item(&mut self, value: T) -> &mut T {
        self.elements.push(value);
        self.elements
            .last_mut()
            .expect("reserved append constructed the last element")
    }
}
impl<T: Default> LinearVector<T> {
    /// Appends a default value; `*vector.appended() = 42`.
    pub fn appended(&mut self) -> &mut T {
        self.reserve_next_item();
        self.push_reserved_item(T::default())
    }
    /// Resizes, default-constructing new values; `vector.resize(10)`.
    /// Use `resize_with_relocation` for observable native relocation callbacks.
    pub fn resize(&mut self, size: usize) {
        self.resize_with_constructor(size, T::default);
    }
    /// Overwrites or extends to an index, returning the stored slot as natively;
    /// `*vector.set_value(5, 42) += 1`.
    pub fn set_value(&mut self, index: usize, value: T) -> &mut T {
        if index >= self.len() {
            self.resize(index + 1);
        }
        self.elements[index] = value;
        &mut self.elements[index]
    }
}
impl<T: Clone> Clone for LinearVector<T> {
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
        }
    }
}
impl<T: Clone> LinearVector<T> {
    /// A zero-based array viewing the values, as the const native `ToArray1()`; the view
    /// detaches a private copy on its first write (see [`Array1::from_shared`]);
    /// `vector.to_array1_view().upper()`.
    pub fn to_array1_view(&self) -> Array1<'_, T> {
        Array1::from_shared(&self.elements, 0)
    }
    /// Constructs repeated values; `LinearVector::with_value(3,42)`.
    pub fn with_value(size: usize, value: T) -> Self {
        Self::with_value_ref(size, &value)
    }
    /// Constructs repeated copies while retaining OCCT's growth capacity rule:
    ///
    /// ```
    /// let fill = 42;
    /// let vector = rustycad_tkernel::ncollection::LinearVector::with_value_ref(3, &fill);
    /// assert_eq!(vector.as_slice(), &[42, 42, 42]);
    /// ```
    pub fn with_value_ref(size: usize, value: &T) -> Self {
        let mut vector = Self::default();
        vector.resize_with_value_ref(size, value);
        vector
    }
    /// Reuses sufficient destination storage during assignment; `dest.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        if source.len() > self.capacity() {
            *self = source.clone();
            return self;
        }
        let previous_len = self.len();
        let common_len = previous_len.min(source.len());
        self.clone_common_items(source, common_len);
        self.finish_assignment_tail(source, common_len, previous_len);
        self
    }

    fn clone_common_items(&mut self, source: &Self, common_len: usize) {
        for index in 0..common_len {
            self.elements[index].clone_from(&source.elements[index]);
        }
    }

    fn finish_assignment_tail(&mut self, source: &Self, common_len: usize, previous_len: usize) {
        if source.len() < previous_len {
            self.elements.truncate(source.len());
            return;
        }
        let mut rollback = LengthRollback::new(&mut self.elements, previous_len);
        for item in &source.elements[common_len..] {
            rollback.elements.push(item.clone());
        }
        rollback.commit();
    }
    /// Resizes by cloning `value` into each new slot:
    ///
    /// ```
    /// let mut vector = rustycad_tkernel::ncollection::LinearVector::<i32>::default();
    /// let fill = -1;
    /// vector.resize_with_value_ref(5, &fill);
    /// assert_eq!(vector[4], -1);
    /// ```
    pub fn resize_with_value_ref(&mut self, size: usize, value: &T) {
        if size < self.len() {
            self.elements.truncate(size);
            return;
        }
        self.reserve(size);
        let previous_len = self.len();
        let mut rollback = LengthRollback::new(&mut self.elements, previous_len);
        while rollback.elements.len() < size {
            rollback.elements.push(value.clone());
        }
        rollback.commit();
    }

    /// Resizes, cloning `value` into new slots; `vector.resize_with_value(5, -1)`.
    pub fn resize_with_value(&mut self, size: usize, value: T) {
        self.resize_with_value_ref(size, &value);
    }
}
impl<T> std::ops::Index<usize> for LinearVector<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.elements[index]
    }
}
impl<T> std::ops::IndexMut<usize> for LinearVector<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.elements[index]
    }
}
impl<'a, T> IntoIterator for &'a LinearVector<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut LinearVector<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
