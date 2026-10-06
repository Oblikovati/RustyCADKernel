//! FromUnicode and its two fromUnicodeImpl copy routines.
use super::UtfString;
use crate::ncollection::{UtfIterator, UtfUnit};
use std::any::TypeId;

impl<T: UtfUnit> UtfString<T> {
    /// Replaces the contents with Unicode text of unit `F`, read until a zero symbol or, when
    /// `length > 0`, until `length` symbols (native `FromUnicode`).
    ///
    /// The slice end acts as the terminator. Text of the same unit type is copied verbatim,
    /// invalid sequences included; other unit types are re-encoded symbol by symbol and
    /// illegal symbols are dropped while still counted in `length()`.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::UtfString;
    /// let mut text = UtfString::<u8>::new();
    /// text.from_unicode(&[0x48u16, 0xD800, 0x49], -1);
    /// assert_eq!((text.to_c_string(), text.length()), (&b"HI\0"[..], 3));
    /// ```
    pub fn from_unicode<F: UtfUnit>(&mut self, source: &[F], length: i32) {
        if UtfIterator::new(source).current() == 0 {
            self.clear(); // native special case: an empty source
            return;
        }
        let limit = if length > 0 { length } else { i32::MAX };
        *self = if TypeId::of::<F>() == TypeId::of::<T>() {
            Self::copy_same_units(source, limit)
        } else {
            Self::convert_units(source, limit)
        };
    }

    /// Same-unit path: copies the raw units of the first `limit` symbols.
    fn copy_same_units<F: UtfUnit>(source: &[F], limit: i32) -> Self {
        let end = scan_symbols(source, limit, |_| {});
        let count = end.buffer_here();
        let mut text = Self::with_size((count * std::mem::size_of::<T>()) as i32, end.index());
        for (slot, offset) in text.units[..count].iter_mut().zip(0..) {
            // An overrun past the slice end copies the zeros the model reads there.
            *slot = T::from_bits(source.get(offset).map_or(0, |&unit| unit.to_bits()));
        }
        text
    }

    /// Converting path: sizes the output first, then re-encodes the same symbols.
    fn convert_units<F: UtfUnit>(source: &[F], limit: i32) -> Self {
        let mut size = 0;
        let end = scan_symbols(source, limit, |iter| size += iter.advance_bytes_utf::<T>());
        let mut text = Self::with_size(size, end.index());
        let mut written = 0;
        scan_symbols(source, end.index(), |iter| {
            written += iter
                .get_utf(&mut text.units[written..])
                .expect("the first pass sized the buffer for every symbol");
        });
        text
    }
}

/// Walks symbols while they are non-zero and fewer than `limit`, calling `visit` on each;
/// returns the iterator stopped at the first symbol not visited.
fn scan_symbols<F: UtfUnit>(
    source: &[F],
    limit: i32,
    mut visit: impl FnMut(&UtfIterator<'_, F>),
) -> UtfIterator<'_, F> {
    let mut iter = UtfIterator::new(source);
    while iter.current() != 0 && iter.index() < limit {
        visit(&iter);
        iter.advance();
    }
    iter
}
