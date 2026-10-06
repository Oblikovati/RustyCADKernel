//! Detached cursors preserve OCCT's captured iteration end while appends continue.
use super::DynamicArray;
use crate::standard::Failure;

/// Native legacy cursor with an explicit array argument for safe access.
/// `let cursor = array.legacy_cursor::<false>(); cursor.value(&array)?;`
/// Moving the array invalidates its identity, as moving a native container does.
pub struct DynamicArrayIterator<T, const MUTABLE: bool = false> {
    owner: *const DynamicArray<T>,
    index: usize,
    end: usize,
}

impl<T, const M: bool> Default for DynamicArrayIterator<T, M> {
    fn default() -> Self {
        Self {
            owner: std::ptr::null(),
            index: 0,
            end: 0,
        }
    }
}
impl<T, const M: bool> Copy for DynamicArrayIterator<T, M> {}
impl<T, const M: bool> Clone for DynamicArrayIterator<T, M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> DynamicArray<T> {
    /// Captures the current iteration end; `array.legacy_cursor::<false>()`.
    pub fn legacy_cursor<const M: bool>(&self) -> DynamicArrayIterator<T, M> {
        DynamicArrayIterator::new(self, 0)
    }
}
impl<T, const M: bool> DynamicArrayIterator<T, M> {
    /// Captures the array and initial position; `DynamicArrayIterator::<i32>::new(&array, 2)`.
    pub fn new(array: &DynamicArray<T>, index: usize) -> Self {
        Self {
            owner: array,
            index,
            end: array.len(),
        }
    }
    /// Reinitializes and recaptures the end, as Init/Initialize; `cursor.initialize(&array)`.
    pub fn initialize(&mut self, array: &DynamicArray<T>) {
        *self = Self::new(array, 0);
    }
    /// Checks the captured range; `cursor.more()`.
    pub fn more(&self) -> bool {
        !self.owner.is_null() && self.index < self.end
    }
    /// Advances one position, including past the end, as Next; `cursor.next()`.
    pub fn next(&mut self) {
        self.index = self.index.wrapping_add(1);
    }
    /// Advances and returns this cursor, as prefix ++; `cursor.advance_returning()`.
    pub fn advance_returning(&mut self) -> &mut Self {
        self.next();
        self
    }
    /// Returns the old position and advances, as postfix ++; `cursor.advance_post()`.
    pub fn advance_post(&mut self) -> Self {
        let old = *self;
        self.next();
        old
    }
    /// Retreats from an exhausted cursor to the captured last item; `cursor.retreat_returning()`.
    pub fn retreat_returning(&mut self) -> &mut Self {
        self.index = if self.more() {
            self.index.wrapping_sub(1)
        } else {
            self.end.saturating_sub(1)
        };
        self
    }
    /// Returns the old position and retreats, as postfix --; `cursor.retreat_post()`.
    pub fn retreat_post(&mut self) -> Self {
        let old = *self;
        self.retreat_returning();
        old
    }
    /// Applies a signed position offset, as +=; `cursor.offset_returning(-1)`.
    pub fn offset_returning(&mut self, offset: isize) -> &mut Self {
        self.index = self.index.wrapping_add_signed(offset);
        self
    }
    /// Subtracts an offset, as -=; `cursor.offset_sub_returning(1)`.
    pub fn offset_sub_returning(&mut self, offset: isize) -> &mut Self {
        self.offset_returning(offset.wrapping_neg())
    }
    /// Returns a displaced copy, as iterator + offset; `cursor.shifted(2)`.
    pub fn shifted(&self, offset: isize) -> Self {
        let mut result = *self;
        result.offset_returning(offset);
        result
    }
    /// Returns a copy displaced backwards, as iterator - offset; `cursor.shifted_back(2)`.
    pub fn shifted_back(&self, offset: isize) -> Self {
        self.shifted(offset.wrapping_neg())
    }
    /// Returns the signed positional difference, as iterator subtraction; `end.distance(&begin)`.
    pub fn distance<const O: bool>(&self, other: &DynamicArrayIterator<T, O>) -> isize {
        (self.index as isize).wrapping_sub(other.index as isize)
    }
    /// Compares positions as native relational operators do; `begin.compare_position(&end)`.
    pub fn compare_position<const O: bool>(
        &self,
        other: &DynamicArrayIterator<T, O>,
    ) -> std::cmp::Ordering {
        self.distance(other).cmp(&0)
    }
    /// Reads the current item through its owning array; `cursor.value(&array)?`.
    pub fn value<'a>(&self, array: &'a DynamicArray<T>) -> Result<&'a T, Failure> {
        self.validate_owner(array)?;
        if !self.more() {
            return Err(self.exhausted_failure());
        }
        array
            .value(self.index)
            .map_err(|error| error.into_failure("NCollection_DynamicArray::Iterator"))
    }
    /// Reads an offset item, as `iterator[offset]`; `cursor.value_at(&array, 2)?`.
    pub fn value_at<'a>(
        &self,
        array: &'a DynamicArray<T>,
        offset: isize,
    ) -> Result<&'a T, Failure> {
        self.shifted(offset).value(array)
    }
    /// Returns the current pointer or null when exhausted, as operator->; `cursor.value_ptr(&array)?`.
    pub fn value_ptr(&self, array: &DynamicArray<T>) -> Result<*const T, Failure> {
        self.validate_owner(array)?;
        if !self.more() {
            return Ok(std::ptr::null());
        }
        Ok(self.value(array)? as *const T)
    }
    /// Copies owner, position and captured end, as assignment; `cursor.assign_from(&other)`.
    pub fn assign_from(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }
    /// Preserves the position for self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Copies this position as a const cursor; `mutable_cursor.as_const_cursor()`.
    pub fn as_const_cursor(&self) -> DynamicArrayIterator<T, false> {
        DynamicArrayIterator {
            owner: self.owner,
            index: self.index,
            end: self.end,
        }
    }
    fn validate_owner(&self, array: &DynamicArray<T>) -> Result<(), Failure> {
        if std::ptr::eq(self.owner, array) {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_DomainError",
            &format!("array {:p}: expected cursor owner {:p}", array, self.owner),
            "",
        ))
    }
    fn exhausted_failure(&self) -> Failure {
        Failure::with_type(
            "Standard_NoSuchObject",
            &format!(
                "cursor index {}: expected initialized range [0, {})",
                self.index, self.end
            ),
            "",
        )
    }
}
impl<T> DynamicArrayIterator<T, true> {
    /// Mutates the current item with an exclusive array borrow; `*cursor.change_value(&mut array)? = 3`.
    pub fn change_value<'a>(&self, array: &'a mut DynamicArray<T>) -> Result<&'a mut T, Failure> {
        self.validate_owner(array)?;
        if !self.more() {
            return Err(self.exhausted_failure());
        }
        array
            .change_value(self.index)
            .map_err(|error| error.into_failure("NCollection_DynamicArray::Iterator"))
    }
}
impl<T> DynamicArrayIterator<T, false> {
    /// Assigns a mutable cursor into a const cursor; `constant.assign_from_mutable(&mutable)`.
    pub fn assign_from_mutable(&mut self, other: &DynamicArrayIterator<T, true>) -> &mut Self {
        *self = other.as_const_cursor();
        self
    }
}
impl<T, const M: bool, const O: bool> PartialEq<DynamicArrayIterator<T, O>>
    for DynamicArrayIterator<T, M>
{
    fn eq(&self, other: &DynamicArrayIterator<T, O>) -> bool {
        self.owner == other.owner && self.index == other.index
    }
}
impl<T, const M: bool> Eq for DynamicArrayIterator<T, M> {}
impl<T, const M: bool, const O: bool> PartialOrd<DynamicArrayIterator<T, O>>
    for DynamicArrayIterator<T, M>
{
    fn partial_cmp(&self, other: &DynamicArrayIterator<T, O>) -> Option<std::cmp::Ordering> {
        // Rust's PartialOrd must agree with owner-sensitive PartialEq.
        if self.owner != other.owner {
            return None;
        }
        Some(self.compare_position(other))
    }
}
