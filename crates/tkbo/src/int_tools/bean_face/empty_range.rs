//! Native zero-width result insertion shared by the solver and private oracle.
use crate::int_tools::MarkedRangeSet;
use rustycad_tkernel::standard::Failure;
pub(crate) fn set_empty_result(
    parameter: f64,
    ranges: &mut MarkedRangeSet,
) -> Result<bool, Failure> {
    let indices = ranges.get_indices(parameter)?.to_vec();
    if indices.is_empty() {
        return Ok(false);
    }
    for index in indices {
        if ranges.flag(index)? == 2 {
            return Ok(false);
        }
    }
    ranges.insert_range(parameter, parameter, 2)?;
    Ok(true)
}
