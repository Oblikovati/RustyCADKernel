use super::*;
use rustycad_tkernel::precision;
/// Preserve native endpoint tests and multiplication order for either point dimension.
pub(super) fn split_parameters(
    first: f64,
    last: f64,
    step: f64,
    mut insert: impl FnMut(f64) -> Result<(), Failure>,
) -> Result<bool, Failure> {
    let reversed = first > last;
    let mut updated = false;
    let mut index = 1_i32;
    loop {
        let current = first + f64::from(index) * step * if reversed { -1.0 } else { 1.0 };
        if reaches_endpoint(current - last, reversed) {
            break;
        }
        insert(current)?;
        updated = true;
        index = index.wrapping_add(1);
    }
    Ok(updated)
}
fn reaches_endpoint(distance: f64, reversed: bool) -> bool {
    if reversed {
        return distance < precision::p_confusion();
    }
    distance.partial_cmp(&-precision::p_confusion()) != Some(std::cmp::Ordering::Less)
}
