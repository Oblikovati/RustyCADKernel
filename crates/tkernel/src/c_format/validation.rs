//! Validation for the safe public formatting surface.
use super::{FormatArg, FormatError, Spec, c_sprintf, parse_spec};

/// Checks supported conversions before using the shared renderer.
pub(crate) fn checked_sprintf(
    format: &[u8],
    args: &[FormatArg<'_>],
) -> Result<Vec<u8>, FormatError> {
    validate_format(format, args)?;
    Ok(c_sprintf(format, args))
}

fn validate_format(format: &[u8], args: &[FormatArg<'_>]) -> Result<(), FormatError> {
    let mut index: usize = 0;
    let mut argument_index: usize = 0;
    while index < format.len() {
        if format[index] != b'%' {
            index += 1;
            continue;
        }
        let offset = index;
        let (spec, end) = parse_spec(format, index + 1);
        argument_index = validate_spec(spec, args, argument_index, offset)?;
        index = end;
    }
    Ok(())
}

fn validate_spec(
    spec: Spec,
    args: &[FormatArg<'_>],
    argument_index: usize,
    offset: usize,
) -> Result<usize, FormatError> {
    let argument_index = validate_dynamic_arguments(spec, args, argument_index, offset)?;
    let expected: &'static str = expected_argument(spec, offset)?;
    if spec.conversion == b'%' {
        if has_percent_modifiers(spec) {
            return Err(format_error(spec, offset, "an unmodified %% directive"));
        }
        return Ok(argument_index);
    }
    validate_argument(spec, args.get(argument_index), expected, offset)?;
    Ok(argument_index + 1)
}

fn validate_dynamic_arguments(
    spec: Spec,
    args: &[FormatArg<'_>],
    mut argument_index: usize,
    offset: usize,
) -> Result<usize, FormatError> {
    let dimensions = [
        (spec.dynamic_width, "an Int for dynamic width"),
        (spec.dynamic_precision, "an Int for dynamic precision"),
    ];
    for (enabled, expected) in dimensions {
        if enabled {
            validate_star_argument(spec, args.get(argument_index), expected, offset)?;
            argument_index += 1;
        }
    }
    Ok(argument_index)
}

fn validate_star_argument(
    spec: Spec,
    argument: Option<&FormatArg<'_>>,
    expected: &'static str,
    offset: usize,
) -> Result<(), FormatError> {
    if matches!(argument, Some(FormatArg::Int(_))) {
        return Ok(());
    }
    Err(format_error(spec, offset, expected))
}

fn validate_argument(
    spec: Spec,
    argument: Option<&FormatArg<'_>>,
    expected: &'static str,
    offset: usize,
) -> Result<(), FormatError> {
    let Some(argument) = argument else {
        return Err(format_error(spec, offset, expected));
    };
    if !matches_argument(argument, spec) {
        return Err(format_error(spec, offset, expected));
    }
    Ok(())
}

fn has_percent_modifiers(spec: Spec) -> bool {
    spec.left
        || spec.plus
        || spec.space
        || spec.alternate
        || spec.zero
        || spec.width != 0
        || spec.dynamic_width
        || spec.precision.is_some()
        || spec.dynamic_precision
        || spec.length != 0
}

fn expected_argument(spec: Spec, offset: usize) -> Result<&'static str, FormatError> {
    supported_argument_kind(spec).ok_or_else(|| {
        format_error(
            spec,
            offset,
            "a supported %c/%d/%i/%o/%u/%x/%X/%ld/%li/%lu/%lx/%lX/%lld/%llu/%f/%e/%g/%s directive",
        )
    })
}

fn supported_argument_kind(spec: Spec) -> Option<&'static str> {
    match spec.conversion {
        b'%' => Some("no argument"),
        b'd' | b'i' | b'o' | b'u' | b'x' | b'X' => integer_argument_kind(spec),
        b'f' | b'F' | b'e' | b'E' | b'g' | b'G' if spec.length == 0 || spec.length == 1 => {
            Some("a Real argument")
        }
        b's' if spec.length == 0 => Some("a Text argument"),
        b'c' if character_format_is_supported(spec) => Some("an Int argument"),
        _ => None,
    }
}

fn integer_argument_kind(spec: Spec) -> Option<&'static str> {
    match (spec.conversion, spec.length) {
        (b'd' | b'i', -2..=0) => Some("an Int argument (hh/h/no modifier)"),
        (b'd' | b'i', 1) => Some("a Long argument for %l"),
        (b'd' | b'i', 2) => Some("a LongLong argument for %ll"),
        (b'o' | b'u' | b'x' | b'X', -2 | -1) => Some("an Int argument (hh/h)"),
        (b'o' | b'u' | b'x' | b'X', 0) => Some("a UInt argument"),
        (b'o' | b'u' | b'x' | b'X', 1) => Some("a ULong argument for %l"),
        (b'o' | b'u' | b'x' | b'X', 2) => Some("a ULongLong argument for %ll"),
        _ => None,
    }
}

fn character_format_is_supported(spec: Spec) -> bool {
    spec.length == 0
        && spec.precision.is_none()
        && !spec.dynamic_precision
        && !spec.plus
        && !spec.space
        && !spec.alternate
        && !spec.zero
}

fn matches_argument(argument: &FormatArg<'_>, spec: Spec) -> bool {
    match spec.conversion {
        b'd' | b'i' | b'o' | b'u' | b'x' | b'X' => integer_argument_matches(argument, spec),
        b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => matches!(argument, FormatArg::Real(_)),
        b's' => matches!(argument, FormatArg::Text(_)),
        b'c' => matches!(argument, FormatArg::Int(_)),
        _ => false,
    }
}

fn integer_argument_matches(argument: &FormatArg<'_>, spec: Spec) -> bool {
    matches!(
        (spec.conversion, spec.length, argument),
        (b'd' | b'i', -2..=0, FormatArg::Int(_))
            | (b'd' | b'i', 1, FormatArg::Long(_))
            | (b'd' | b'i', 2, FormatArg::LongLong(_))
            | (b'o' | b'u' | b'x' | b'X', -2 | -1, FormatArg::Int(_))
            | (b'o' | b'u' | b'x' | b'X', 0, FormatArg::UInt(_))
            | (b'o' | b'u' | b'x' | b'X', 1, FormatArg::ULong(_))
            | (b'o' | b'u' | b'x' | b'X', 2, FormatArg::ULongLong(_))
    )
}

fn format_error(spec: Spec, offset: usize, expected: &'static str) -> FormatError {
    FormatError {
        offset,
        conversion: spec.conversion,
        expected,
    }
}
