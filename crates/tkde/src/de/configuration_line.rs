// Rust port of the file-static helpers of DataExchange/TKDE/DE/DE_ConfigurationContext.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The resource-line grammar of `DE_ConfigurationContext`: `key : value`, `!` comments,
//! the `\` escapes before a value, and the file reader that splits a file into lines.
use rustycad_tkernel::{osd::File, standard::Failure, tcollection::AsciiString};

/// The native `aWhiteSpace` set of `WhatKindOfLine`.
const WHITE_SPACE: &[u8] = b" \t\r\n";

/// The native `DE_ConfigurationContext_KindOfLine`, with the two tokens of a resource
/// line carried by the resource kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConfigurationLineKind {
    /// `DE_ConfigurationContext_KindOfLine_Empty`.
    Empty,
    /// `DE_ConfigurationContext_KindOfLine_Comment`.
    Comment,
    /// `DE_ConfigurationContext_KindOfLine_Resource`: the key and its value.
    Resource(AsciiString, AsciiString),
    /// `DE_ConfigurationContext_KindOfLine_Error`: no `:` after the key.
    Error,
}

/// The native `WhatKindOfLine`. A line that is blank after trimming raises
/// `Standard_OutOfRange` (`Value(0)`), as natively: the native `LoadStr` fails on a line of
/// spaces between two resources (tools/oracle/tkde_configuration_probe.cxx, `ctx.blank`).
pub(crate) fn what_kind_of_line(line: &AsciiString) -> Result<ConfigurationLineKind, Failure> {
    let mut text: AsciiString = line.clone();
    text.left_adjust();
    text.right_adjust();
    // Native condition: after RightAdjust the line never ends with a blank, so a space is
    // appended unless the line ends with ':'.
    let ends_blank: bool = text.ends_with(b" ") && text.ends_with(b"\t") && text.ends_with(b"\n");
    if !text.ends_with(b":") && !ends_blank {
        text.insert_after(text.length(), b" ")?;
    }
    if text.value(1)? == b'!' {
        return Ok(ConfigurationLineKind::Comment);
    }
    let first: i32 = text.first_location_not_in_set(WHITE_SPACE, 1, text.length())?;
    if text.value(first)? == b'\n' {
        return Ok(ConfigurationLineKind::Empty);
    }
    let colon: i32 = text.location_char(1, b':', first, text.length())?;
    if colon == 0 || first == colon {
        return Ok(ConfigurationLineKind::Error);
    }
    let key: AsciiString = text.sub_string(first, key_end(&text, colon)?)?;
    let value: AsciiString = resource_value(text, colon)?;
    Ok(ConfigurationLineKind::Resource(key, value))
}

/// The last position of the key: the byte before the colon, blanks skipped backwards.
fn key_end(text: &AsciiString, colon: i32) -> Result<i32, Failure> {
    let mut position: i32 = colon - 1;
    while matches!(text.value(position)?, b'\t' | b' ') {
        position -= 1;
    }
    Ok(position)
}

/// The value after the colon: leading blanks skipped, one escaped `\`, ` ` or `\t` kept
/// after a backslash, and the last byte (the appended space or the final `:`) removed.
fn resource_value(mut text: AsciiString, colon: i32) -> Result<AsciiString, Failure> {
    let start: i32 = if colon != text.length() {
        colon + 1
    } else {
        colon
    };
    let mut position: i32 = text.first_location_not_in_set(WHITE_SPACE, start, text.length())?;
    if position != 0 && text.value(position)? == b'\\' {
        // The native switch reads past the backslash; the appended space keeps it in range.
        if matches!(text.value(position + 1)?, b'\\' | b' ' | b'\t') {
            position += 1;
        }
    }
    if position == text.length() || position == 0 {
        return Ok(AsciiString::default());
    }
    text.remove(1, position - 1)?;
    text.remove(text.length(), 1)?;
    Ok(text)
}

/// The native `MakeName`: `scope.param`, or `param` alone for an empty scope.
pub(crate) fn make_name(scope: &[u8], param: &[u8]) -> AsciiString {
    let mut name: AsciiString = AsciiString::from_raw(scope);
    if !name.is_empty() {
        name += b'.';
    }
    name += AsciiString::from_raw(param);
    name
}

/// The native `GetLine`: reads `file` in chunks of at most 9 bytes up to the next new line
/// (kept); the last line of a file without a final new line gets one. `false` at the end.
pub(crate) fn read_resource_line(file: &mut File, line: &mut AsciiString) -> Result<bool, Failure> {
    let mut buffer: AsciiString = AsciiString::default();
    line.clear();
    loop {
        file.read_line(&mut buffer, 10)?;
        *line += &buffer;
        if file.is_at_end()? {
            if line.is_empty() {
                return Ok(false);
            }
            *line += "\n";
        }
        if line.value(line.length())? == b'\n' {
            return Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(text: &str) -> ConfigurationLineKind {
        what_kind_of_line(&AsciiString::new(text)).expect("a parsable line")
    }

    fn resource(key: &str, value: &str) -> ConfigurationLineKind {
        ConfigurationLineKind::Resource(AsciiString::new(key), AsciiString::new(value))
    }

    #[test]
    fn a_resource_line_trims_the_key_and_the_value() {
        assert_eq!(
            kind("  a.b  :\t two words \n"),
            resource("a.b", "two words")
        );
    }

    #[test]
    fn comments_errors_and_empty_values_follow_the_native_grammar() {
        assert_eq!(kind("  ! note"), ConfigurationLineKind::Comment);
        assert_eq!(kind("novalue"), ConfigurationLineKind::Error);
        assert_eq!(kind(": v"), ConfigurationLineKind::Error);
        assert_eq!(kind("k :"), resource("k", ""));
        assert_eq!(kind("k : \t "), resource("k", ""));
    }

    #[test]
    fn a_trailing_colon_is_dropped_from_the_value() {
        assert_eq!(kind("a : b:"), resource("a", "b"));
        assert_eq!(kind("c:d:e"), resource("c", "d:e"));
    }

    #[test]
    fn a_backslash_escapes_one_blank_or_backslash() {
        assert_eq!(kind("a : \\ x"), resource("a", " x"));
        assert_eq!(kind("b : \\\\y"), resource("b", "\\y"));
        assert_eq!(kind("d : \\q"), resource("d", "\\q"));
    }

    #[test]
    fn a_blank_line_raises_out_of_range_as_natively() {
        let failure = what_kind_of_line(&AsciiString::new("   ")).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    }

    #[test]
    fn make_name_joins_scope_and_param_with_a_dot() {
        assert_eq!(make_name(b"s", b"p"), AsciiString::new("s.p"));
        assert_eq!(make_name(b"", b"p"), AsciiString::new("p"));
    }
}
