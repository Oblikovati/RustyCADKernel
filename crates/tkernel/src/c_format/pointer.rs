//! The Linux glibc `%p` spelling used by OCCT's C-locale formatting wrappers.
use super::{CArg, Spec, integer::precision_digits, pad, sign_prefix};

pub(super) fn format(spec: &Spec, argument: Option<&CArg<'_>>) -> Vec<u8> {
    let address = match argument {
        Some(CArg::Pointer(pointer)) => pointer.addr(),
        _ => 0,
    };
    if address == 0 {
        // glibc treats the null spelling as text: no sign, precision, or zero padding.
        return pad(spec, Vec::new(), b"(nil)".to_vec(), false);
    }
    let mut prefix = sign_prefix(spec, false);
    prefix.extend_from_slice(b"0x");
    let digits = precision_digits(address as u128, b'x', spec.precision);
    pad(spec, prefix, digits, spec.precision.is_none())
}
