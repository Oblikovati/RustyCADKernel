//! The stream and dictionary formatters of `Graphic3d_FrameStats.cxx` (`formatCounter`,
//! `formatBytes`, `formatTime`, `addInfo`, `addTimeInfo`), in the C locale layout of the
//! native `std::setw`, `std::fixed` and `Sprintf`.
use crate::cast::uint_from_f64;
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::storage::format_general;
use rustycad_tkernel::tcollection::AsciiString;

/// The statistics dictionary of `FormatStats`.
pub type Graphic3dFrameStatsDict = IndexedDataMap<AsciiString, AsciiString>;

/// `THE_SECOND_IN_HOUR`: the native multiplies by the reciprocal, not divides.
const SECOND_IN_HOUR: f64 = 1.0 / 3600.0;
/// `THE_SECOND_IN_MINUTE`.
const SECOND_IN_MINUTE: f64 = 1.0 / 60.0;

/// `std::setw(width)` with the default right alignment and a space fill; `pad("7", 3) == "  7"`.
pub(crate) fn pad(text: &str, width: usize) -> String {
    format!("{text:>width$}")
}

/// `std::fixed << std::setprecision(1) << value` (glibc `%.1f`); `fixed1(2.25) == "2.2"`.
pub(crate) fn fixed1(value: f64) -> String {
    if value.is_nan() {
        return if value.is_sign_negative() {
            "-nan"
        } else {
            "nan"
        }
        .to_string();
    }
    format!("{value:.1}")
}

/// Appends `prefix`, `body` padded to `width`, and `postfix`.
fn push_field(
    out: &mut String,
    prefix: Option<&str>,
    body: &str,
    width: usize,
    postfix: Option<&str>,
) {
    out.push_str(prefix.unwrap_or(""));
    out.push_str(&pad(body, width));
    out.push_str(postfix.unwrap_or(""));
}

/// The value scaled to the first unit it reaches, as (`fixed1` number, unit suffix).
fn scaled(value: usize, units: &[(f64, &'static str)]) -> Option<(String, &'static str)> {
    let real = value as f64;
    let (scale, suffix) = units.iter().find(|(scale, _)| real >= *scale)?;
    Some((fixed1(real / scale), suffix))
}

/// `formatCounter`: a count with a G/M/k suffix; a plain count without postfix gets a
/// trailing space. Only the number is padded, as `std::setw` pads the next output only.
/// Example: `format_counter(&mut out, 5, Some("Layers: "), 1500, None)` appends "Layers:   1.5k".
pub(crate) fn format_counter(
    out: &mut String,
    width: usize,
    prefix: Option<&str>,
    value: usize,
    postfix: Option<&str>,
) {
    let units = [(1_000_000_000.0, "G"), (1_000_000.0, "M"), (1_000.0, "k")];
    let (number, suffix) = match scaled(value, &units) {
        Some(scaled) => scaled,
        None if postfix.is_none() => (value.to_string(), " "),
        None => (value.to_string(), ""),
    };
    push_field(out, prefix, &number, width, Some(suffix));
    out.push_str(postfix.unwrap_or(""));
}

/// `formatCounterPair`: a counter, the immediate counter in brackets when shown, and a
/// line end; `format_counter_pair(&mut out, 5, "    Layers: ", 3, 1, true)`.
pub(crate) fn format_counter_pair(
    out: &mut String,
    width: usize,
    prefix: &str,
    value: usize,
    immediate: usize,
    show_immediate: bool,
) {
    format_counter(out, width, Some(prefix), value, None);
    if show_immediate {
        format_counter(out, 1, Some("("), immediate, Some(")"));
    }
    out.push('\n');
}

/// `formatBytes`: a byte count in B, KiB, MiB or GiB; `format_bytes(&mut out, 5, "  Geometry: ", 2048, "\n")`
/// appends "  Geometry:   2.0 KiB\n".
pub(crate) fn format_bytes(
    out: &mut String,
    width: usize,
    prefix: &str,
    value: usize,
    postfix: &str,
) {
    let units = [
        (1_073_741_824.0, " GiB"),
        (1_048_576.0, " MiB"),
        (1_024.0, " KiB"),
    ];
    let (number, unit) = scaled(value, &units).unwrap_or_else(|| (value.to_string(), " B"));
    push_field(out, Some(prefix), &number, width, Some(unit));
    out.push_str(postfix);
}

/// The native split of seconds into (hours, minutes, seconds, milliseconds), with the
/// x86-64 `unsigned int` conversions; `split_time(3725.5) == (1, 2, 5, 500.0)`.
pub(crate) fn split_time(seconds: f64) -> (u32, u32, u32, f64) {
    let mut rest = seconds;
    let hours = uint_from_f64(rest * SECOND_IN_HOUR);
    rest -= f64::from(hours) * 3600.0;
    let minutes = uint_from_f64(rest * SECOND_IN_MINUTE);
    rest -= f64::from(minutes) * 60.0;
    let whole = uint_from_f64(rest);
    rest -= f64::from(whole);
    (hours, minutes, whole, 1000.0 * rest)
}

/// The `Sprintf` clock text of a time of at least one second, or `None` below it;
/// `seconds_suffix` is the text after a seconds-only value.
fn clock_text(seconds: f64, seconds_suffix: &str) -> (Option<String>, f64) {
    let (hours, minutes, whole, millis) = split_time(seconds);
    let text = if hours > 0 {
        Some(format!("{hours:02}:{minutes:02}:{whole:02}"))
    } else if minutes > 0 {
        Some(format!("{minutes:02}:{whole:02}"))
    } else if whole > 0 {
        Some(format!("{whole:2}{seconds_suffix}"))
    } else {
        None
    };
    (text, millis)
}

/// `formatTime`: hh:mm:ss, mm:ss, "s" or "ms" text; `format_time(&mut out, 5, " Elapsed Frame: ", 0.016, "\n")`.
pub(crate) fn format_time(
    out: &mut String,
    width: usize,
    prefix: &str,
    seconds: f64,
    postfix: &str,
) {
    let (clock, millis) = clock_text(seconds, " s");
    match clock {
        Some(text) => push_field(out, Some(prefix), &text, width, Some(postfix)),
        None => {
            push_field(out, Some(prefix), &fixed1(millis), width, None);
            out.push_str(" ms");
            out.push_str(postfix);
        }
    }
}

/// `addInfo(const char*)`: adds or replaces a key in place; `add_info_text(&mut dict, "FPS", "60")`.
pub(crate) fn add_info_text(dict: &mut Graphic3dFrameStatsDict, key: &str, value: &str) {
    let value = AsciiString::from(value);
    let index = dict.add(AsciiString::from(key), value.clone());
    if let Ok(slot) = dict.change_from_index(index) {
        *slot = value;
    }
}

/// `addInfo(double)`: the value as `%.1g`; `add_info_real(&mut dict, "FPS", 59.9)` stores "6e+01".
pub(crate) fn add_info_real(dict: &mut Graphic3dFrameStatsDict, key: &str, value: f64) {
    add_info_text(dict, key, &format_general(value, 1));
}

/// `addInfo(size_t)`: the value as `%zu`; `add_info_size(&mut dict, "Layers", 5)`.
pub(crate) fn add_info_size(dict: &mut Graphic3dFrameStatsDict, key: &str, value: usize) {
    add_info_text(dict, key, &value.to_string());
}

/// `addTimeInfo`: the clock text, or the milliseconds as `%.1g` below one second;
/// `add_time_info(&mut dict, "CPU Frame (max)", 0.02)`.
pub(crate) fn add_time_info(dict: &mut Graphic3dFrameStatsDict, key: &str, seconds: f64) {
    match clock_text(seconds, "") {
        (Some(text), _) => add_info_text(dict, key, &text),
        (None, millis) => add_info_real(dict, key, millis),
    }
}
