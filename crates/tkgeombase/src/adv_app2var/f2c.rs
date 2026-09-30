//! The f2c array conventions of the AdvApp2Var kernels, without pointer arithmetic.
//!
//! An f2c routine receives a pointer to the first element of a Fortran array and
//! shifts it (`courbe -= courbe_offset;`) so that the Fortran subscripts, written as
//! `courbe[i + j * courbe_dim1]`, index it directly. [`F2cArray`] and
//! [`F2cArrayMut`] hold the slice that starts at the passed pointer and that shift,
//! so the translated kernels keep the native subscript expressions verbatim:
//! C index `k` reads `data[k - offset]`. `tail(k)` is the native `&array[k]` passed
//! to a callee. An index outside the slice panics where the native code reads or
//! writes out of bounds.
use std::ops::{Index, IndexMut};

/// A read-only Fortran array seen through the f2c pointer shift.
///
/// `let a = F2cArray::new(&values, 1); assert_eq!(a[1], values[0]);`.
#[derive(Clone, Copy)]
pub(crate) struct F2cArray<'a, T> {
    data: &'a [T],
    offset: i32,
}

impl<'a, T> F2cArray<'a, T> {
    /// Views `data` as the native pointer shifted by `offset` (`p -= offset`);
    /// `F2cArray::new(&courbe, courbe_dim1 + 1)`.
    pub(crate) fn new(data: &'a [T], offset: i32) -> Self {
        Self { data, offset }
    }

    /// The slice that starts at C index `k` (the native `&array[k]`); `a.tail(3)`.
    pub(crate) fn tail(&self, k: i32) -> &'a [T] {
        &self.data[slot(k, self.offset)..]
    }
}

impl<T> Index<i32> for F2cArray<'_, T> {
    type Output = T;

    fn index(&self, k: i32) -> &T {
        &self.data[slot(k, self.offset)]
    }
}

/// A writable Fortran array seen through the f2c pointer shift.
///
/// `let mut a = F2cArrayMut::new(&mut values, 1); a[1] = 2.0;`.
pub(crate) struct F2cArrayMut<'a, T> {
    data: &'a mut [T],
    offset: i32,
}

impl<'a, T> F2cArrayMut<'a, T> {
    /// Views `data` as the native pointer shifted by `offset` (`p -= offset`);
    /// `F2cArrayMut::new(&mut crvnew, crvnew_dim1 + 1)`.
    pub(crate) fn new(data: &'a mut [T], offset: i32) -> Self {
        Self { data, offset }
    }

    /// The slice that starts at C index `k` (the native `&array[k]`); `a.tail(3)`.
    pub(crate) fn tail(&self, k: i32) -> &[T] {
        &self.data[slot(k, self.offset)..]
    }

    /// The writable slice that starts at C index `k`; `a.tail_mut(3)`.
    pub(crate) fn tail_mut(&mut self, k: i32) -> &mut [T] {
        &mut self.data[slot(k, self.offset)..]
    }
}

impl<T> Index<i32> for F2cArrayMut<'_, T> {
    type Output = T;

    fn index(&self, k: i32) -> &T {
        &self.data[slot(k, self.offset)]
    }
}

impl<T> IndexMut<i32> for F2cArrayMut<'_, T> {
    fn index_mut(&mut self, k: i32) -> &mut T {
        &mut self.data[slot(k, self.offset)]
    }
}

/// The slice position of C index `k`; a negative position is a native out-of-bounds
/// access, reported with the offending subscript.
fn slot(k: i32, offset: i32) -> usize {
    let position: i32 = k - offset;
    usize::try_from(position).unwrap_or_else(|_| {
        panic!("f2c subscript {k} lies before the array start (offset {offset})")
    })
}

/// The f2c `pow_di` of the kernels: `x` multiplied `|n|` times, inverted for a
/// negative `n` (`AdvApp2Var_MathBase::pow__di` and the file-static `pow__ii`
/// share this loop); `pow_by_loop(2.0, 3) == 8.0`.
pub(crate) fn pow_by_loop(x: f64, n: i32) -> f64 {
    let mut result: f64 = 1.0;
    for _ in 0..n.unsigned_abs() {
        result *= x;
    }
    if n < 0 {
        result = 1.0 / result;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifted_views_keep_the_fortran_subscripts() {
        let mut values: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        let view = F2cArray::new(&values, 3);
        assert_eq!(view[3], 1.0);
        assert_eq!(view.tail(5), &[3.0, 4.0]);
        let mut writable = F2cArrayMut::new(&mut values, 1);
        writable[4] += writable[1];
        assert_eq!(writable.tail_mut(4)[0], 5.0);
    }

    #[test]
    #[should_panic(expected = "f2c subscript 0 lies before the array start (offset 1)")]
    fn a_subscript_before_the_start_panics_with_the_subscript() {
        let values: [f64; 1] = [0.0];
        let _ = F2cArray::new(&values, 1)[0];
    }

    #[test]
    fn the_power_loop_inverts_negative_exponents() {
        assert_eq!(pow_by_loop(2.0, 3), 8.0);
        assert_eq!(pow_by_loop(2.0, -2), 0.25);
        assert_eq!(pow_by_loop(7.5, 0), 1.0);
    }
}
