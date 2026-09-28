//! Rust side of the Standard_CLocaleSentry case (tkernel_standard_services_locale.cxx).
use crate::common::{hex, next};
use rustycad_tkernel::standard::c_locale_sentry::CLocaleSentry;
use rustycad_tkernel::tcollection::AsciiString;
use std::str::SplitWhitespace;

/// "services_clocale <real bits>": locale identity, %g text, radix and restoration.
pub fn clocale_case(input: &mut SplitWhitespace<'_>) -> String {
    let value = f64::from_bits(next(input));
    let first = CLocaleSentry::get_c_locale();
    let second = CLocaleSentry::get_c_locale();
    let text = {
        let _outer = CLocaleSentry::new();
        let _inner = CLocaleSentry::new();
        let locale = CLocaleSentry::get_c_locale();
        format!(
            "{} point={} sep={}",
            String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()),
            locale.decimal_point(),
            hex(locale.thousands_separator().as_bytes())
        )
    };
    format!(
        "nonnull=1 same={} {text} restored=1",
        u8::from(std::ptr::eq(first, second))
    )
}
