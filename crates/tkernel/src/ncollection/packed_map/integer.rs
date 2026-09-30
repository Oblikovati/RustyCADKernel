/// Unsigned storage word for packed integer blocks.
pub trait PackedWord: Copy + Default {
    /// Preserve the low word bits; `u32::from_bits(42)`.
    fn from_bits(value: u64) -> Self;
    /// Widen without sign extension; `word.bits()`.
    fn bits(self) -> u64;
}
impl PackedWord for u32 {
    fn from_bits(value: u64) -> Self {
        value as Self
    }
    fn bits(self) -> u64 {
        u64::from(self)
    }
}
impl PackedWord for u64 {
    fn from_bits(value: u64) -> Self {
        value
    }
    fn bits(self) -> u64 {
        self
    }
}
/// Integral type with OCCT's width and bit-preserving signed conversion.
pub trait PackedInteger: Copy + Ord + Default + std::fmt::Debug {
    /// Matching unsigned block/index storage type.
    type Word: PackedWord;
    /// Whether to use 64 rather than 32 bits per block.
    const IS_64_BIT: bool;
    /// Lowest representable value, returned by an empty maximum query.
    const MIN: Self;
    /// Highest representable value, returned by an empty minimum query.
    const MAX: Self;
    /// Convert to the native unsigned representation; `value.bits()`.
    fn bits(self) -> u64;
    /// Restore the signed or unsigned value; `T::from_bits(bits)`.
    fn from_bits(bits: u64) -> Self;
}
macro_rules! integer {
    ($type:ty, $word:ty) => {
        impl PackedInteger for $type {
            type Word = $word;
            const IS_64_BIT: bool = size_of::<Self>() > 4;
            const MIN: Self = Self::MIN;
            const MAX: Self = Self::MAX;
            fn bits(self) -> u64 {
                (self as $word) as u64
            }
            fn from_bits(bits: u64) -> Self {
                bits as Self
            }
        }
    };
}
integer!(i32, u32);
integer!(u32, u32);
integer!(i64, u64);
integer!(u64, u64);
#[cfg(target_pointer_width = "64")]
integer!(usize, u64);
#[cfg(target_pointer_width = "32")]
integer!(usize, u32);
