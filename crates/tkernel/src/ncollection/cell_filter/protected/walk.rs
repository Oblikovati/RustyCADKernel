//! Preserve the native recursive cursor even when an inner range is reversed or a callback fails.
use crate::standard::Failure;
pub(super) fn walk_cell_indices(
    axis: i32,
    cursor: &mut [i32],
    min: &[i32],
    max: &[i32],
    mut visit: impl FnMut(&[i32]),
) -> Result<(), Failure> {
    if validate_cell_range(axis, cursor.len(), min, max)? {
        recurse_cell_indices(axis as usize, cursor, min, max, &mut visit);
    }
    Ok(())
}
fn validate_cell_range(
    axis: i32,
    cursor: usize,
    min: &[i32],
    max: &[i32],
) -> Result<bool, Failure> {
    if axis < 0 || axis as usize >= min.len().min(max.len()) {
        return Err(recursion_bounds_error(axis, [cursor, min.len(), max.len()]));
    }
    if min[axis as usize] > max[axis as usize] {
        return Ok(false);
    }
    if axis as usize >= cursor {
        return Err(recursion_bounds_error(axis, [cursor, min.len(), max.len()]));
    }
    Ok(true)
}
fn recursion_bounds_error(axis: i32, [cursor, min, max]: [usize; 3]) -> Failure {
    let available = cursor.min(min).min(max);
    Failure::out_of_range(&format!(
        "CellFilter recursion axis {axis}, lengths ({cursor}, {min}, {max}): expected 0 <= axis < {available} for accessed storage"
    ))
}
fn recurse_cell_indices(
    axis: usize,
    cursor: &mut [i32],
    min: &[i32],
    max: &[i32],
    visit: &mut impl FnMut(&[i32]),
) {
    for value in min[axis]..=max[axis] {
        cursor[axis] = value;
        if axis == 0 {
            visit(cursor);
        } else {
            recurse_cell_indices(axis - 1, cursor, min, max, visit);
        }
    }
}
