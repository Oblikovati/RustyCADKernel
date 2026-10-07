use super::{VectorBase, VectorItem};
use std::fmt;
use std::ops::{Index, IndexMut, Mul, MulAssign, Neg};
// Why `Index`/`IndexMut` stay although library code never panics (2026-10 audit):
// the fallible arithmetic operators were removed (their only caller used the Result
// methods instead), but indexing has 943 call sites in rustycad-tkmath alone and more in
// every geometry crate. The index is a caller precondition, as for slices and as for the
// native `operator()` (bounds checked only by Standard_OutOfRange_Raise_if in debug
// builds): every library caller iterates within `lower()..=upper()`, so the panic below
// is unreachable there. Code holding an index it has not bounded uses the Result
// accessors `value`/`change_value`, which return Standard_OutOfRange with the index.
/// `operator()(int)` const with the caller-guaranteed index of a bounded loop;
/// `vector[vector.lower()]`. Panics (naming the index and bounds) outside the bounds.
impl<T: VectorItem> Index<i32> for VectorBase<'_, T> {
    type Output = T;
    #[track_caller]
    fn index(&self, index: i32) -> &T {
        let (lower, upper) = (self.lower(), self.upper());
        self.value(index)
            .unwrap_or_else(|_| index_out_of_bounds(index, lower, upper))
    }
}
/// `operator()(int)` with the caller-guaranteed index of a bounded loop;
/// `vector[vector.lower()] = 1.0`. Panics (naming the index and bounds) outside them.
impl<T: VectorItem> IndexMut<i32> for VectorBase<'_, T> {
    #[track_caller]
    fn index_mut(&mut self, index: i32) -> &mut T {
        let (lower, upper) = (self.lower(), self.upper());
        self.change_value(index)
            .unwrap_or_else(|_| index_out_of_bounds(index, lower, upper))
    }
}
/// The precondition failure of vector indexing: the index and the bounds it missed.
#[cold]
#[track_caller]
fn index_out_of_bounds(index: i32, lower: i32, upper: i32) -> ! {
    panic!(
        "math_VectorBase index {index} outside [{lower}, {upper}]; use value()/change_value() for an unchecked index"
    )
}
impl<'a, T: VectorItem> Mul<T> for &VectorBase<'a, T> {
    type Output = VectorBase<'a, T>;
    fn mul(self, right: T) -> Self::Output {
        self.multiplied(right)
    }
}
impl<'a, T: VectorItem> Neg for &VectorBase<'a, T> {
    type Output = VectorBase<'a, T>;
    fn neg(self) -> Self::Output {
        self.opposite()
    }
}
impl<T: VectorItem> MulAssign<T> for VectorBase<'_, T> {
    fn mul_assign(&mut self, right: T) {
        self.multiply(right);
    }
}
/// The friend `operator<<`, which forwards to `Dump`; `format!("{vector}")`.
impl<T: VectorItem> fmt::Display for VectorBase<'_, T> {
    fn fmt(&self, stream: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(stream)
    }
}
// The friend `operator*(TheItemType, const math_VectorBase&)`; coherence rules need one
// impl per concrete item type.
macro_rules! scalar_times_vector {
    ($item:ty) => {
        impl<'a> Mul<&VectorBase<'a, $item>> for $item {
            type Output = VectorBase<'a, $item>;
            fn mul(self, right: &VectorBase<'a, $item>) -> Self::Output {
                right.multiplied(self)
            }
        }
    };
}
scalar_times_vector!(f64);
scalar_times_vector!(i32);
