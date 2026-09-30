// Rust translation of the printf conversions of the IntPolyh Dump members at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The glibc `printf` conversions the IntPolyh `Dump` members use: `%+8.3e`, `%f`, `%8f`
//! and `%5f` of doubles (the integer conversions are the Rust width formats).

/// `%+W.Pe`: a sign, one digit, `precision` decimals and an exponent of at least two
/// digits, right-aligned in `width`. `c_exp(1234.5, 8, 3) == "+1.234e+03"`.
pub(crate) fn c_exp(value: f64, width: usize, precision: usize) -> String {
    let text: String = if value.is_finite() {
        let raw: String = format!("{value:.precision$e}");
        let (mantissa, exponent) = raw.split_once('e').unwrap_or((&raw, "0"));
        let exponent: i32 = exponent.parse().unwrap_or(0);
        let sign: &str = if value.is_sign_negative() { "" } else { "+" };
        let exp_sign: char = if exponent < 0 { '-' } else { '+' };
        format!("{sign}{mantissa}e{exp_sign}{:02}", exponent.abs())
    } else {
        signed_special(value, true)
    };
    format!("{text:>width$}")
}

/// `%W.Pf`: fixed notation with `precision` decimals, right-aligned in `width`.
/// `c_fixed(0.5, 8, 6) == "0.500000"`.
pub(crate) fn c_fixed(value: f64, width: usize, precision: usize) -> String {
    let text: String = if value.is_finite() {
        format!("{value:.precision$}")
    } else {
        signed_special(value, false)
    };
    format!("{text:>width$}")
}

/// glibc's `inf` and `nan`, with the sign of the value (`-nan` for a negative NaN) and a
/// `+` for a positive one under the `+` flag.
fn signed_special(value: f64, plus: bool) -> String {
    let word: &str = if value.is_nan() { "nan" } else { "inf" };
    if value.is_sign_negative() {
        return format!("-{word}");
    }
    if plus {
        return format!("+{word}");
    }
    word.to_string()
}
