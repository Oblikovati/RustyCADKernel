//! Native cursor arithmetic keeps captured owner/end metadata in every result.
use super::DynamicArrayIterator;
use std::ops::{Add, AddAssign, Sub, SubAssign};

impl<T, const M: bool> Add<isize> for DynamicArrayIterator<T, M> {
    type Output = Self;
    fn add(self, offset: isize) -> Self {
        self.shifted(offset)
    }
}
impl<T, const M: bool> Add<DynamicArrayIterator<T, M>> for isize {
    type Output = DynamicArrayIterator<T, M>;
    fn add(self, cursor: DynamicArrayIterator<T, M>) -> Self::Output {
        cursor.shifted(self)
    }
}
impl<T, const M: bool> Sub<isize> for DynamicArrayIterator<T, M> {
    type Output = Self;
    fn sub(self, offset: isize) -> Self {
        self.shifted_back(offset)
    }
}
impl<T, const M: bool> AddAssign<isize> for DynamicArrayIterator<T, M> {
    fn add_assign(&mut self, offset: isize) {
        self.offset_returning(offset);
    }
}
impl<T, const M: bool> SubAssign<isize> for DynamicArrayIterator<T, M> {
    fn sub_assign(&mut self, offset: isize) {
        self.offset_sub_returning(offset);
    }
}
impl<T, const M: bool, const O: bool> Sub<DynamicArrayIterator<T, O>>
    for DynamicArrayIterator<T, M>
{
    type Output = isize;
    fn sub(self, other: DynamicArrayIterator<T, O>) -> isize {
        self.distance(&other)
    }
}
impl<T> From<DynamicArrayIterator<T, true>> for DynamicArrayIterator<T, false> {
    fn from(other: DynamicArrayIterator<T, true>) -> Self {
        other.as_const_cursor()
    }
}
