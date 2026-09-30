//! CorrectSplitValues preserves S4208 parameter synchronization, including crossed tail bounds.
use super::*;
use rustycad_tkernel::ncollection::HSequence;
pub(super) type Knots = Handle<HSequence<f64>>;
pub(super) fn value(knots: &Knots, index: usize) -> Result<f64, Failure> {
    state::require(knots, "split values")?;
    knots.sequence(|values| values.value(index).copied())
}
fn set_value(knots: &Knots, index: usize, value: f64) -> Result<(), Failure> {
    state::require(knots, "split values")?;
    knots.change_sequence(|values| values.set_value(index, value))
}
pub(super) fn length(knots: &Knots) -> Result<usize, Failure> {
    state::require(knots, "split values")?;
    Ok(knots.sequence(|values| values.len()))
}
pub(super) fn correct(
    original3: &Knots,
    original2: &Knots,
    new2: &Knots,
    new3: &Knots,
) -> Result<(), Failure> {
    let (len3, len2) = (length(original3)?, length(original2)?);
    let (mut fixed2, mut fixed3) = (vec![false; len3 + 1], vec![false; len2 + 1]);
    let last3 = value(original3, len3)?;
    let last2 = value(original2, len2)?;
    align(original3, original2, new2, new3, &mut fixed2, &mut fixed3)?;
    align(original2, original3, new3, new2, &mut fixed3, &mut fixed2)?;
    spread(new2, &mut fixed2, last3)?;
    spread(new3, &mut fixed3, last2)
}
fn align(
    original: &Knots,
    other: &Knots,
    projected: &Knots,
    back: &Knots,
    fixed: &mut [bool],
    back_fixed: &mut [bool],
) -> Result<(), Failure> {
    for (index, flag) in fixed.iter_mut().enumerate().skip(1) {
        let Some(matched) = matching_parameter(other, value(projected, index)?)? else {
            continue;
        };
        if back_fixed[matched] {
            continue;
        }
        set_value(projected, index, value(other, matched)?)?;
        *flag = true;
        set_value(back, matched, value(original, index)?)?;
        back_fixed[matched] = true;
    }
    Ok(())
}
fn matching_parameter(knots: &Knots, parameter: f64) -> Result<Option<usize>, Failure> {
    for index in 1..=length(knots)? {
        if (parameter - value(knots, index)?).abs() < precision::p_confusion() {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
fn spread(knots: &Knots, fixed: &mut [bool], last: f64) -> Result<(), Failure> {
    for index in 1..fixed.len() - 1 {
        let distance = value(knots, index + 1)? - value(knots, index)?;
        if distance < precision::p_confusion() {
            spread_pair(knots, fixed, index)?;
        }
    }
    let length = fixed.len() - 1;
    if value(knots, length)? > last {
        repair_tail(knots, fixed)?;
    }
    Ok(())
}
fn spread_pair(knots: &Knots, fixed: &mut [bool], index: usize) -> Result<(), Failure> {
    let previous = value(knots, index + 1)?;
    set_value(
        knots,
        index + 1,
        value(knots, index)? + 2. * precision::p_confusion(),
    )?;
    if fixed[index + 1] {
        set_value(knots, index, previous)?;
        fixed[index] = true;
        fixed[index + 1] = false;
    }
    Ok(())
}
fn repair_tail(knots: &Knots, fixed: &[bool]) -> Result<(), Failure> {
    let mut index = fixed.len() - 1;
    while index > 1 && !fixed[index] {
        index -= 1;
    }
    let mut last = value(knots, index)?;
    for position in (index..fixed.len()).rev() {
        set_value(knots, position, last)?;
        last -= 2. * precision::p_confusion();
    }
    Ok(())
}
