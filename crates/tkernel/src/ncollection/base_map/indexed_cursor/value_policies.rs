//! Native accessor results need not be references: callbacks preserve values and proxy types.
use super::BaseMapIndexedIterator;
use crate::standard::Failure;

impl<C, const M: bool> BaseMapIndexedIterator<C, M> {
    /// Reads Size without requiring reference-valued accessors; `cursor.more_with(&map, |m| m.len())?`.
    pub fn more_with(
        &self,
        container: &C,
        size: impl FnOnce(&C) -> usize,
    ) -> Result<bool, Failure> {
        if self.owner.is_null() {
            return Ok(false);
        }
        self.check_owner(container)?;
        Ok(self.index <= size(container))
    }

    /// Preserves a const accessor's actual return type; `cursor.value_with(&map, |m, i| m.read(i))?`.
    pub fn value_with<'a, R>(
        &self,
        container: &'a C,
        access: impl FnOnce(&'a C, usize) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        self.check_owner(container)?;
        access(container, self.index)
    }
    /// Calls the const key policy without forcing a borrowed key; `cursor.key_with(&map, |m, i| m.key(i))?`.
    pub fn key_with<'a, R>(
        &self,
        container: &'a C,
        find_key: impl FnOnce(&'a C, usize) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        self.value_with(container, find_key)
    }
    /// Returns an offset accessor result without changing this cursor; `cursor.value_at_with(&map, -1, |m, i| m.read(i))?`.
    pub fn value_at_with<'a, R>(
        &self,
        container: &'a C,
        offset: isize,
        access: impl FnOnce(&'a C, usize) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        self.shifted(offset).value_with(container, access)
    }
}

impl<C> BaseMapIndexedIterator<C, true> {
    /// Preserves a mutable container's FindKey overload; `cursor.change_key_with(&mut map, |m, i| m.key_mut(i))?`.
    pub fn change_key_with<'a, R>(
        &self,
        container: &'a mut C,
        find_key: impl FnOnce(&'a mut C, usize) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        self.change_value_with(container, find_key)
    }

    /// Preserves a writable accessor's proxy or reference result; `cursor.change_value_with(&mut map, |m, i| m.write(i))?`.
    pub fn change_value_with<'a, R>(
        &self,
        container: &'a mut C,
        access: impl FnOnce(&'a mut C, usize) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        self.check_owner(container)?;
        access(container, self.index)
    }
    /// Applies signed subscript then mutable accessor dispatch; `cursor.change_value_at_with(&mut map, 1, |m, i| m.write(i))?`.
    pub fn change_value_at_with<'a, R>(
        &self,
        container: &'a mut C,
        offset: isize,
        access: impl FnOnce(&'a mut C, usize) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        self.shifted(offset).change_value_with(container, access)
    }
}
