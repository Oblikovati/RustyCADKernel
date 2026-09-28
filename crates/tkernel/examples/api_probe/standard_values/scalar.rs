//! Standard_Real, Standard_ShortReal, Standard_Integer, Standard_Character and
//! Standard_ExtCharacter cases (twin of tkernel_standard_values_scalar_probe.cxx).
use super::super::common::{hex, next};
use rustycad_tkernel::standard::{character, ext_character, integer, real, short_real};
use std::str::SplitWhitespace;

/// A double read from its 16-hex-digit IEEE-754 pattern; `next_real(&mut input)`.
pub fn next_real(input: &mut SplitWhitespace<'_>) -> f64 {
    let token = input.next().expect("double bits");
    f64::from_bits(u64::from_str_radix(token, 16).expect("hex double bits"))
}

/// A float read from its 8-hex-digit IEEE-754 pattern.
fn next_short_real(input: &mut SplitWhitespace<'_>) -> f32 {
    let token = input.next().expect("float bits");
    f32::from_bits(u32::from_str_radix(token, 16).expect("hex float bits"))
}

/// An unsigned unit (byte or UTF-16 unit) read from hex.
fn next_hex_unit(input: &mut SplitWhitespace<'_>) -> u32 {
    u32::from_str_radix(input.next().expect("hex unit"), 16).expect("hex unit value")
}

/// 16 lower-case hex digits, as the C++ `hex64`.
pub fn hex64(value: u64) -> String {
    format!("{value:016x}")
}

/// 8 lower-case hex digits, as the C++ `hex32`.
pub fn hex32(value: u32) -> String {
    format!("{value:08x}")
}

/// Binds a unary Standard_Real name to its port.
fn real_unary(name: &str) -> fn(f64) -> f64 {
    match name {
        "acos" => real::acos,
        "acos_approx" => real::acos_approx,
        "asin" => real::asin,
        "atan" => real::atan,
        "atanh" => real::atanh,
        "acosh" => real::acosh,
        "asinh" => real::asinh,
        "cosh" => real::cosh,
        "sinh" => real::sinh,
        "tanh" => real::tanh,
        "cos" => real::cos,
        "sin" => real::sin,
        "tan" => real::tan,
        "exp" => real::exp,
        "log" => real::log,
        "log10" => real::log10,
        "sqrt" => real::sqrt,
        "abs" => real::abs,
        "ceiling" => real::ceiling,
        "floor" => real::floor,
        "integer_part" => real::integer_part,
        "round" => real::round,
        "epsilon" => real::epsilon,
        "real_part" => real::real_part,
        "square" => real::square,
        _ => panic!("unary Standard_Real function {name:?} is not in the probe table"),
    }
}

/// Binds a binary Standard_Real name to its port (IsEqual answers 1.0 or 0.0).
fn real_binary(name: &str) -> fn(f64, f64) -> f64 {
    match name {
        "atan2" => real::atan2,
        "next_after" => real::next_after,
        "sign" => real::sign,
        "max" => real::max,
        "min" => real::min,
        "pow" => real::pow,
        "is_equal" => |a, b| f64::from(u8::from(real::is_equal(a, b))),
        _ => panic!("binary Standard_Real function {name:?} is not in the probe table"),
    }
}

/// "real1 <name> <bits>".
pub fn real1_case(input: &mut SplitWhitespace<'_>) -> String {
    let function = real_unary(input.next().expect("function name"));
    hex64(function(next_real(input)).to_bits())
}

/// "real2 <name> <bits> <bits>".
pub fn real2_case(input: &mut SplitWhitespace<'_>) -> String {
    let function = real_binary(input.next().expect("function name"));
    let first = next_real(input);
    hex64(function(first, next_real(input)).to_bits())
}

/// "real_to_int <bits>".
pub fn real_to_int_case(input: &mut SplitWhitespace<'_>) -> String {
    real::real_to_int(next_real(input)).to_string()
}

/// "real_to_short_real <bits>".
pub fn real_to_short_real_case(input: &mut SplitWhitespace<'_>) -> String {
    hex32(real::real_to_short_real(next_real(input)).to_bits())
}

/// "int_to_real <int>".
pub fn int_to_real_case(input: &mut SplitWhitespace<'_>) -> String {
    hex64(real::int_to_real(next::<i32>(input)).to_bits())
}

/// "short_real <abs|max|min> <bits32> <bits32>".
pub fn short_real_case(input: &mut SplitWhitespace<'_>) -> String {
    let name = input.next().expect("function name");
    let first = next_short_real(input);
    let second = next_short_real(input);
    let result = match name {
        "abs" => short_real::abs(first),
        "max" => short_real::max(first, second),
        _ => short_real::min(first, second),
    };
    hex32(result.to_bits())
}

/// "integer <name> <a> <b>".
pub fn integer_case(input: &mut SplitWhitespace<'_>) -> String {
    let name = input.next().expect("function name");
    let first = next::<i32>(input);
    let second = next::<i32>(input);
    match name {
        "is_even" => u8::from(integer::is_even(first)).to_string(),
        "is_odd" => u8::from(integer::is_odd(first)).to_string(),
        "abs" => integer::abs(first).to_string(),
        "max" => integer::max(first, second).to_string(),
        "min" => integer::min(first, second).to_string(),
        "square" => integer::square(first).to_string(),
        _ => integer::modulus(first, second).to_string(),
    }
}

/// "character <hex byte>": the classifier bits, then LowerCase and UpperCase.
pub fn character_case(input: &mut SplitWhitespace<'_>) -> String {
    let byte = next_hex_unit(input) as u8;
    let classifiers: [fn(u8) -> bool; 11] = [
        character::is_alphabetic,
        character::is_digit,
        character::is_xdigit,
        character::is_alphanumeric,
        character::is_control,
        character::is_graphic,
        character::is_lower_case,
        character::is_printable,
        character::is_punctuation,
        character::is_space,
        character::is_upper_case,
    ];
    let flags: String = classifiers
        .iter()
        .map(|classify| if classify(byte) { '1' } else { '0' })
        .collect();
    let lower = hex(&[character::lower_case(byte)]);
    format!("{flags} {lower} {}", hex(&[character::upper_case(byte)]))
}

/// "char_equal <hex> <hex>".
pub fn char_equal_case(input: &mut SplitWhitespace<'_>) -> String {
    let first = next_hex_unit(input) as u8;
    let second = next_hex_unit(input) as u8;
    u8::from(character::is_equal(first, second)).to_string()
}

/// "to_ext_character <hex byte>".
pub fn to_ext_character_case(input: &mut SplitWhitespace<'_>) -> String {
    ext_character::to_ext_character(next_hex_unit(input) as u8).to_string()
}

/// "ext_character <unit> <unit>": ToCharacter, IsAnAscii and IsEqual.
pub fn ext_character_case(input: &mut SplitWhitespace<'_>) -> String {
    let first = next_hex_unit(input) as u16;
    let second = next_hex_unit(input) as u16;
    format!(
        "{} {} {}",
        hex(&[ext_character::to_character(first)]),
        u8::from(ext_character::is_an_ascii(first)),
        u8::from(ext_character::is_equal(first, second))
    )
}

/// "limits": every Standard_Real/ShortReal/Integer limit function.
pub fn limits_case(_: &mut SplitWhitespace<'_>) -> String {
    let reals = format!(
        "{} {} {} {} {} {} {} {} {} {}",
        hex64(real::real_small().to_bits()),
        real::real_digits(),
        hex64(real::real_epsilon().to_bits()),
        hex64(real::real_first().to_bits()),
        real::real_first10_exp(),
        hex64(real::real_last().to_bits()),
        real::real_last10_exp(),
        real::real_mantissa(),
        real::real_radix(),
        real::real_size()
    );
    let short_reals = format!(
        "{} {} {} {} {} {} {} {} {} {}",
        hex32(short_real::short_real_small().to_bits()),
        short_real::short_real_digits(),
        hex32(short_real::short_real_epsilon().to_bits()),
        hex32(short_real::short_real_first().to_bits()),
        short_real::short_real_first10_exp(),
        hex32(short_real::short_real_last().to_bits()),
        short_real::short_real_last10_exp(),
        short_real::short_real_mantissa(),
        short_real::short_real_radix(),
        short_real::short_real_size()
    );
    let integers = format!(
        "{} {} {}",
        integer::integer_first(),
        integer::integer_last(),
        integer::integer_size()
    );
    format!("{reals} {short_reals} {integers}")
}
