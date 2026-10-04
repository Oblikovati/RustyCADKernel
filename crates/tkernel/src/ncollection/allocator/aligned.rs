//! Protected alignment helpers from `NCollection_AccAllocator` for derived allocator logic.
use super::ACC_ALIGN;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// `NCollection_AccAllocator::AlignedSize`, rounding up to its four-byte allocation quantum.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccAlignedSize(usize);

impl AccAlignedSize {
    /// Rounds a byte count up exactly like the native unsigned constructor.
    /// Example: `AccAlignedSize::new(5).get() == 8`.
    pub const fn new(size: usize) -> Self {
        Self(size.wrapping_add(ACC_ALIGN - 1) & !(ACC_ALIGN - 1))
    }

    /// Returns the rounded byte count (`operator size_t`); `AccAlignedSize::new(7).get() == 8`.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// `NCollection_AccAllocator::AlignedPtr`, a raw address rounded down to four bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccAlignedPtr(*mut u8);

impl AccAlignedPtr {
    /// Creates the native aligned pointer view; `AccAlignedPtr::from_ptr(std::ptr::null_mut()).as_ptr().is_null()`.
    pub fn from_ptr(pointer: *mut u8) -> Self {
        Self(pointer.map_addr(|address| address & !(ACC_ALIGN - 1)))
    }

    /// Returns the raw aligned address; `AccAlignedPtr::default().as_ptr().is_null()`.
    pub const fn as_ptr(self) -> *mut u8 {
        self.0
    }

    /// Adds a rounded amount using wrapping address arithmetic; `AccAlignedPtr::default().wrapping_add(AccAlignedSize::new(1))`.
    pub const fn wrapping_add(self, amount: AccAlignedSize) -> Self {
        Self(self.0.wrapping_add(amount.get()))
    }

    /// Subtracts a rounded amount; `AccAlignedPtr::default().wrapping_add(AccAlignedSize::new(1)).wrapping_sub(AccAlignedSize::new(1))`.
    pub const fn wrapping_sub(self, amount: AccAlignedSize) -> Self {
        Self(self.0.wrapping_sub(amount.get()))
    }

    /// Mutates and returns the new pointer (`AlignedPtr::operator+=`); `let mut p = AccAlignedPtr::default(); p.wrapping_add_assign(AccAlignedSize::new(1))`.
    pub fn wrapping_add_assign(&mut self, amount: AccAlignedSize) -> Self {
        *self = self.wrapping_add(amount);
        *self
    }

    /// Mutates and returns the new pointer (`AlignedPtr::operator-=`); `let mut p = AccAlignedPtr::default(); p.wrapping_sub_assign(AccAlignedSize::new(1))`.
    pub fn wrapping_sub_assign(&mut self, amount: AccAlignedSize) -> Self {
        *self = self.wrapping_sub(amount);
        *self
    }
}

impl Add<AccAlignedSize> for AccAlignedPtr {
    type Output = Self;

    fn add(self, rhs: AccAlignedSize) -> Self::Output {
        self.wrapping_add(rhs)
    }
}

impl Sub<AccAlignedSize> for AccAlignedPtr {
    type Output = Self;

    fn sub(self, rhs: AccAlignedSize) -> Self::Output {
        self.wrapping_sub(rhs)
    }
}

impl AddAssign<AccAlignedSize> for AccAlignedPtr {
    fn add_assign(&mut self, rhs: AccAlignedSize) {
        self.wrapping_add_assign(rhs);
    }
}

impl SubAssign<AccAlignedSize> for AccAlignedPtr {
    fn sub_assign(&mut self, rhs: AccAlignedSize) {
        self.wrapping_sub_assign(rhs);
    }
}
