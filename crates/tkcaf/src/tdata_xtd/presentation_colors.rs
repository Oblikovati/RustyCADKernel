// Rust port of TDataXtd_Presentation::getColorNameFromOldEnum and getOldColorNameFromNewEnum
// (ApplicationFramework/TKCAF/TDataXtd/TDataXtd_Presentation.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The conversions between the current `Quantity_NameOfColor` values and the values of the
//! old enumeration (before the merged and moved colors), used by stored documents.
use rustycad_tkernel::quantity::NamedColor;
use rustycad_tkernel::standard::{Failure, exceptions};

/// The old values that map to one gray directly, in both directions.
const OLD_GRAYS: [(i32, NamedColor); 11] = [
    (146, NamedColor::GRAY2),
    (157, NamedColor::GRAY3),
    (168, NamedColor::GRAY4),
    (179, NamedColor::GRAY5),
    (190, NamedColor::GRAY6),
    (201, NamedColor::GRAY7),
    (212, NamedColor::GRAY8),
    (222, NamedColor::GRAY9),
    (235, NamedColor::GRAY97),
    (236, NamedColor::GRAY98),
    (237, NamedColor::GRAY99),
];

/// The old values of the merged colors (`CHARTREUSE1`, `GOLD1`, `GREEN1`, `ORANGE1`,
/// `ORANGERED1`, `RED1`, `TOMATO1`, `YELLOW1`): each one at or below shifts the value down.
const OLD_MERGED: [i32; 8] = [43, 124, 230, 360, 365, 413, 489, 511];

/// The old grays `GRAY2`..`GRAY8`: the old gray block was ordered by tens, so each
/// threshold passed moves the value one place less.
const OLD_GRAY_TENS: [i32; 7] = [146, 157, 168, 179, 190, 201, 212];

/// The first new colors after a merged one (the `++anOld` corrections).
const NEW_AFTER_MERGED: [NamedColor; 8] = [
    NamedColor::CHARTREUSE2,
    NamedColor::GOLD2,
    NamedColor::GREEN2,
    NamedColor::ORANGE2,
    NamedColor::ORANGERED2,
    NamedColor::RED2,
    NamedColor::TOMATO2,
    NamedColor::YELLOW2,
];

/// The new grays by tens, `GRAY20`..`GRAY80`.
const NEW_GRAY_TENS: [NamedColor; 7] = [
    NamedColor::GRAY20,
    NamedColor::GRAY30,
    NamedColor::GRAY40,
    NamedColor::GRAY50,
    NamedColor::GRAY60,
    NamedColor::GRAY70,
    NamedColor::GRAY80,
];

/// The number of `thresholds` passed by `value` (`>` when `strict`, else `>=`).
fn passed(value: i32, thresholds: impl Iterator<Item = i32>, strict: bool) -> i32 {
    let count = thresholds
        .filter(|&t| if strict { value > t } else { value >= t })
        .count();
    i32::try_from(count).unwrap_or(i32::MAX)
}

/// `getColorNameFromOldEnum(theOld)`: the current color of an old value, clamped to
/// `WHITE` (a negative value has no color: `Standard_OutOfRange`, natively an invalid
/// enumerator).
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::presentation_colors::color_from_old;
/// use rustycad_tkernel::quantity::NamedColor;
/// assert_eq!(color_from_old(146).unwrap(), NamedColor::GRAY2);
/// assert_eq!(color_from_old(10_000).unwrap(), NamedColor::WHITE);
/// ```
pub fn color_from_old(old: i32) -> Result<NamedColor, Failure> {
    if let Some((_, color)) = OLD_GRAYS.iter().find(|(value, _)| *value == old) {
        return Ok(*color);
    }
    let mut new = old - passed(old, OLD_MERGED.into_iter(), false);
    if (136..223).contains(&old) {
        new += 8 - passed(old, OLD_GRAY_TENS.into_iter(), true);
    }
    if old > 228 {
        new += 3;
    }
    if old >= 238 {
        new -= 3;
    }
    if new > NamedColor::WHITE.index() {
        return Ok(NamedColor::WHITE);
    }
    NamedColor::from_index(new).ok_or_else(|| {
        exceptions::out_of_range(&format!(
            "TDataXtd_Presentation::getColorNameFromOldEnum: old color {old} gives {new}; expected 0 or more"
        ))
    })
}

/// `getOldColorNameFromNewEnum(theNew)`: the old value of a current color.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::presentation_colors::{color_from_old, old_from_color};
/// use rustycad_tkernel::quantity::NamedColor;
/// assert_eq!(old_from_color(NamedColor::GRAY99), 237);
/// assert_eq!(color_from_old(old_from_color(NamedColor::RED)).unwrap(), NamedColor::RED);
/// ```
pub fn old_from_color(color: NamedColor) -> i32 {
    if let Some((old, _)) = OLD_GRAYS.iter().find(|(_, gray)| *gray == color) {
        return *old;
    }
    let new = color.index();
    let mut old = new + passed(new, NEW_AFTER_MERGED[..2].iter().map(|c| c.index()), false);
    if (NamedColor::GRAY2.index()..=NamedColor::GRAY89.index()).contains(&new) {
        old -= 8 - passed(new, NEW_GRAY_TENS.iter().map(|c| c.index()), false);
    }
    if new >= NamedColor::GREEN.index() {
        old -= 3;
    }
    if new >= NamedColor::HONEYDEW.index() {
        old += 3;
    }
    old + passed(new, NEW_AFTER_MERGED[2..].iter().map(|c| c.index()), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_color_survives_the_old_values() {
        for color in NamedColor::all() {
            assert_eq!(
                color_from_old(old_from_color(color)).unwrap(),
                color,
                "{}",
                color.name()
            );
        }
    }

    #[test]
    fn the_merged_old_values_map_to_their_survivor() {
        assert_eq!(color_from_old(42).unwrap(), color_from_old(43).unwrap());
        assert!(color_from_old(-1).is_err());
    }
}
