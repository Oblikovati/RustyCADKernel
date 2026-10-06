//! Native wide signed inputs use numeric promotion, unlike UTF-8's unsigned-byte reinterpretation.
use super::UtfUnit;

macro_rules! signed_wide_unit {
    ($signed:ty, $unsigned:ty) => {
        impl UtfUnit for $signed {
            fn read_symbol(buffer: &[Self], position: usize) -> (u32, usize) {
                // OCCT 3d097a0 readUTF16 promotes signed halves before its surrogate check.
                // No nonnegative i16 can reach 0xD800, so signed halves never form a pair.
                (
                    buffer.get(position).map_or(0, |&unit| unit as u32),
                    position + 1,
                )
            }
            fn advance_bytes(symbol: u32) -> usize {
                <$unsigned as UtfUnit>::advance_bytes(symbol)
            }
            fn write_symbol(symbol: u32, output: &mut [Self]) {
                let count = Self::advance_bytes(symbol) / size_of::<Self>();
                let mut encoded = [0 as $unsigned; 2];
                <$unsigned as UtfUnit>::write_symbol(symbol, &mut encoded[..count]);
                for (slot, unit) in output.iter_mut().zip(&encoded[..count]) {
                    *slot = *unit as $signed;
                }
            }
            fn to_bits(self) -> u32 {
                u32::from(self as $unsigned)
            }
            fn from_bits(value: u32) -> Self {
                value as $signed
            }
        }
    };
}
signed_wide_unit!(i16, u16);
signed_wide_unit!(i32, u32);
