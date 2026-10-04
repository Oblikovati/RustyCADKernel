//! Integer conversions `d i o u x X` of the C `sprintf` subset.
use super::{CArg, Spec, pad, sign_prefix};

/// The value C reads for an `int` vararg under a length modifier, as a 64-bit pattern.
///
/// `h`/`hh` truncate to `short`/`char`. `l`/`ll` read a 64-bit slot the caller filled with a
/// 32-bit `mov`, which zero-extends on x86-64 SysV: `%ld` of `-1` prints `4294967295`.
fn widened(value: i32, spec: &Spec, signed: bool) -> i128 {
    match (spec.length, signed) {
        (-2, true) => i128::from(value as i8),
        (-2, false) => i128::from(value as u8),
        (-1, true) => i128::from(value as i16),
        (-1, false) => i128::from(value as u16),
        (0, true) => i128::from(value),
        (0, false) => i128::from(value as u32),
        _ => i128::from(value as u32),
    }
}

fn magnitude_digits(magnitude: u128, conversion: u8) -> Vec<u8> {
    let text = match conversion {
        b'o' => format!("{magnitude:o}"),
        b'x' => format!("{magnitude:x}"),
        b'X' => format!("{magnitude:X}"),
        _ => format!("{magnitude}"),
    };
    text.into_bytes()
}

/// Formats one integer conversion; a missing or non-integer argument formats as 0.
pub(super) fn format(spec: &Spec, argument: Option<&CArg<'_>>) -> Vec<u8> {
    let signed = matches!(spec.conversion, b'd' | b'i');
    let value = integer_value(argument, spec, signed);
    let mut digits = precision_digits(value.unsigned_abs(), spec.conversion, spec.precision);
    let prefix = integer_prefix(spec, value, &mut digits, signed);
    pad(spec, prefix, digits, spec.precision.is_none())
}

pub(super) fn precision_digits(value: u128, conversion: u8, precision: Option<usize>) -> Vec<u8> {
    let mut digits = magnitude_digits(value, conversion);
    if let Some(precision) = precision {
        if precision == 0 && value == 0 {
            digits.clear();
        }
        let missing = precision.saturating_sub(digits.len());
        digits.splice(0..0, std::iter::repeat_n(b'0', missing));
    }
    digits
}

fn integer_value(argument: Option<&CArg<'_>>, spec: &Spec, signed: bool) -> i128 {
    match argument {
        Some(CArg::Int(value)) => widened(*value, spec, signed),
        Some(CArg::UInt(value)) => widened(*value as i32, spec, signed),
        Some(CArg::Long(value)) => i128::from(*value),
        Some(CArg::ULong(value)) => i128::from(*value),
        Some(CArg::LongLong(value)) => i128::from(*value),
        Some(CArg::ULongLong(value)) => i128::from(*value),
        Some(CArg::SignedSize(value)) | Some(CArg::PtrDiff(value)) => *value as i128,
        Some(CArg::Size(value)) | Some(CArg::UnsignedPtrDiff(value)) => *value as i128,
        _ => 0,
    }
}

fn integer_prefix(spec: &Spec, value: i128, digits: &mut Vec<u8>, signed: bool) -> Vec<u8> {
    if signed {
        return sign_prefix(spec, value < 0);
    }
    match spec.conversion {
        // `#o` forces a leading zero digit (also for the value 0 at precision 0).
        b'o' if spec.alternate && digits.first() != Some(&b'0') => {
            digits.insert(0, b'0');
            Vec::new()
        }
        b'x' if spec.alternate && value != 0 => b"0x".to_vec(),
        b'X' if spec.alternate && value != 0 => b"0X".to_vec(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::c_format::{CArg, c_sprintf};

    #[test]
    fn private_int_long_mismatch_keeps_legacy_x86_64_widening() {
        assert_eq!(c_sprintf(b"%ld", &[CArg::Int(-1)]), b"4294967295");
        assert_eq!(c_sprintf(b"%lld", &[CArg::Int(-1)]), b"4294967295");
        assert_eq!(c_sprintf(b"%d", &[CArg::UInt(u32::MAX)]), b"-1");
        assert_eq!(c_sprintf(b"%u", &[CArg::UInt(u32::MAX)]), b"4294967295");
    }
}
