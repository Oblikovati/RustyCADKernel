//! An unforgeable safe-code witness that native rehash cannot call a user word implementation.
use std::marker::PhantomData;
use std::num::NonZeroUsize;

/// Kernel-created primitive reader; `<u32 as PackedWord>::NATIVE_REHASH_READER.is_some()`.
/// Its invariant word parameter prevents substituting another type's reader.
///
/// Safe callers cannot inject a function that panics during destructive rehash:
/// ```compile_fail,E0451
/// use rustycad_tkernel::ncollection::PackedWordReader;
/// let forged = PackedWordReader::<u32> {
///     reader: |_| panic!("injected reader"),
///     identity: std::marker::PhantomData,
/// };
/// ```
/// Nor can a reader be applied to another word representation:
/// ```compile_fail,E0308
/// use rustycad_tkernel::ncollection::{PackedWord, PackedWordReader};
/// let _: PackedWordReader<u64> = <u32 as PackedWord>::NATIVE_REHASH_READER.unwrap();
/// ```
/// A copied reader's function remains private:
/// ```compile_fail,E0616
/// use rustycad_tkernel::ncollection::PackedWord;
/// let mut reader = <u32 as PackedWord>::NATIVE_REHASH_READER.unwrap();
/// reader.reader = |_| panic!("replaced reader");
/// ```
#[derive(Clone, Copy)]
pub struct PackedWordReader<W> {
    reader: fn(W) -> u64,
    identity: PhantomData<fn(W) -> W>,
}
impl<W> PackedWordReader<W> {
    pub(in super::super) fn bucket(self, mask: W, is_64_bit: bool, modulus: NonZeroUsize) -> usize {
        // Only fixed primitive casts below create readers; the shift is always 5 or 6.
        let shift: u32 = if is_64_bit { 6 } else { 5 };
        ((self.reader)(mask) >> shift) as usize % modulus.get() + 1
    }
}
pub(super) const U32_READER: PackedWordReader<u32> = PackedWordReader {
    reader: |word| u64::from(word),
    identity: PhantomData,
};
pub(super) const U64_READER: PackedWordReader<u64> = PackedWordReader {
    reader: |word| word,
    identity: PhantomData,
};

#[cfg(test)]
mod tests;
