//! Argument parsing helpers of the `Draw` class: `ParseOnOff`, `ParseOnOffIterator`,
//! `ParseOnOffNoIterator` and `ParseColor`.
use super::session::DrawSession;
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::quantity::{Color, ColorRgba};

/// `Draw::ParseOnOff`: `on`/`1` or `off`/`0` (any case); `None` otherwise.
///
/// ```
/// use rustycad_tkdraw::draw::parse_on_off;
/// assert_eq!(parse_on_off("ON"), Some(true));
/// assert_eq!(parse_on_off("0"), Some(false));
/// assert_eq!(parse_on_off("yes"), None);
/// ```
pub fn parse_on_off(argument: &str) -> Option<bool> {
    match argument.to_ascii_lowercase().as_str() {
        "on" | "1" => Some(true),
        "off" | "0" => Some(false),
        _ => None,
    }
}

/// `Draw::ParseOnOffIterator`: the flag after `args[*index]` when it is an on/off word
/// (the index then moves onto it), true otherwise.
///
/// ```
/// use rustycad_tkdraw::draw::parse_on_off_iterator;
/// let args = ["cmd", "-flag", "off", "x"];
/// let mut index = 1;
/// assert!(!parse_on_off_iterator(&args, &mut index));
/// assert_eq!(index, 2);
/// ```
pub fn parse_on_off_iterator(args: &[&str], index: &mut usize) -> bool {
    let value = args.get(*index + 1).and_then(|next| parse_on_off(next));
    if let Some(value) = value {
        *index += 1;
        return value;
    }
    true
}

/// `Draw::ParseOnOffNoIterator`: as [`parse_on_off_iterator`], inverted when the option
/// starts with `no` or `-no` (any case).
///
/// ```
/// use rustycad_tkdraw::draw::parse_on_off_no_iterator;
/// let mut index = 1;
/// assert!(!parse_on_off_no_iterator(&["cmd", "-noFeature"], &mut index));
/// ```
pub fn parse_on_off_no_iterator(args: &[&str], index: &mut usize) -> bool {
    let option = args
        .get(*index)
        .map(|a| a.to_ascii_lowercase())
        .unwrap_or_default();
    let reverse = option.starts_with("no") || option.starts_with("-no");
    let value = parse_on_off_iterator(args, index);
    if reverse { !value } else { value }
}

/// `THE_MAX_INTEGER_COLOR_COMPONENT`.
const MAX_INTEGER_COMPONENT: i32 = 255;
/// `THE_MAX_REAL_COLOR_COMPONENT`.
const MAX_REAL_COMPONENT: f32 = 1.0;

impl DrawSession {
    /// `Draw::ParseColor` (RGBA): a hex color, a named color with an optional alpha, or
    /// 3-4 components either as integers 0..255 (sRGB, at least one above 1) or reals
    /// 0..1 (linear). Returns how many arguments were used (0 on a syntax error).
    pub fn parse_color_rgba(&self, args: &[&str]) -> (usize, Option<ColorRgba>) {
        self.parse_color(args, true)
    }

    /// `Draw::ParseColor` (RGB): as [`Self::parse_color_rgba`] without alpha.
    pub fn parse_color_rgb(&self, args: &[&str]) -> (usize, Option<Color>) {
        let (count, color) = self.parse_color(args, false);
        (count, color.map(|rgba| *rgba.rgb()))
    }

    /// `Draw::parseColor`.
    fn parse_color(&self, args: &[&str], with_alpha: bool) -> (usize, Option<ColorRgba>) {
        if let Some(color) = args
            .first()
            .and_then(|first| ColorRgba::from_hex(first, !with_alpha))
        {
            return (1, Some(color));
        }
        if let Some(mut color) = args.first().and_then(|first| ColorRgba::from_name(first)) {
            if with_alpha
                && let Some(alpha) = args.get(1).and_then(|text| self.color_component(text))
            {
                color.set_alpha(alpha);
                return (2, Some(color));
            }
            return (1, Some(color));
        }
        if args.len() < 3 {
            return (0, None);
        }
        let count = args.len().min(if with_alpha { 4 } else { 3 });
        if let Some(parsed) = self.integer_color(&args[..count]) {
            return parsed;
        }
        self.real_color(&args[..count]).unwrap_or((0, None))
    }

    /// `parseColorComponent`: an integer 0..255 scaled to 0..1 (except that 1 stays 1),
    /// or a real 0..1.
    fn color_component(&self, text: &str) -> Option<f32> {
        if let Some(integer) = self.integer_component(text) {
            return Some(if integer == 1 {
                MAX_REAL_COMPONENT
            } else {
                integer as f32 * 1.0 / MAX_INTEGER_COMPONENT as f32
            });
        }
        self.real_component(text)
    }

    /// The integer variant of `parseNumericalColorComponent`: 0..255.
    fn integer_component(&self, text: &str) -> Option<i32> {
        self.parse_integer(text)
            .filter(|value| (0..=MAX_INTEGER_COMPONENT).contains(value))
    }

    /// The real variant of `parseNumericalColorComponent`: 0..1 after the cast to float.
    fn real_component(&self, text: &str) -> Option<f32> {
        let value = self.parse_real(text)? as f32;
        (0.0..=MAX_REAL_COMPONENT).contains(&value).then_some(value)
    }

    /// `parseIntegerColor`: integer components (a bad fourth one ends the color at three);
    /// at least one above 1; converted from sRGB to linear.
    fn integer_color(&self, args: &[&str]) -> Option<(usize, Option<ColorRgba>)> {
        let (count, values) = numerical_color(args, -1, |text| self.integer_component(text))?;
        if values.iter().copied().max().unwrap_or(-1) <= 1 {
            return None;
        }
        let alpha = if values[3] == -1 {
            MAX_INTEGER_COMPONENT
        } else {
            values[3]
        };
        let scaled = [values[0], values[1], values[2], alpha]
            .map(|v| v as f32 / MAX_INTEGER_COMPONENT as f32);
        let linear = ColorRgba::srgb_to_linear_vec4(scaled);
        let color =
            ColorRgba::from_vec4(Vec4::new(linear[0], linear[1], linear[2], linear[3])).ok()?;
        Some((count, Some(color)))
    }

    /// `parseRealColor`: real components in 0..1, alpha 1 by default.
    fn real_color(&self, args: &[&str]) -> Option<(usize, Option<ColorRgba>)> {
        let (count, values) =
            numerical_color(args, MAX_REAL_COMPONENT, |text| self.real_component(text))?;
        let color =
            ColorRgba::from_vec4(Vec4::new(values[0], values[1], values[2], values[3])).ok()?;
        Some((count, Some(color)))
    }
}

/// `parseNumericalColor`: parses each component into a 4-vector filled with `default`;
/// a bad fourth component shortens the color to three, any other bad one fails.
fn numerical_color<T: Copy>(
    args: &[&str],
    default: T,
    parse: impl Fn(&str) -> Option<T>,
) -> Option<(usize, [T; 4])> {
    let mut values = [default; 4];
    let mut count = args.len();
    for (index, text) in args.iter().enumerate() {
        match parse(text) {
            Some(value) => values[index] = value,
            None if index == 3 => count = 3,
            None => return None,
        }
    }
    Some((count, values))
}
