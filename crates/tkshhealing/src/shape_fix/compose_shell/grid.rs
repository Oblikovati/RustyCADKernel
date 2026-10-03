use crate::shape_analysis::adjust_to_period;
use crate::shape_extend::JointValues;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkernel::standard::Failure;

pub(super) fn patch_index(
    parameter: f64,
    joints: &JointValues,
    is_closed: bool,
) -> Result<i32, Failure> {
    let values = joints.array1();
    let upper = values.upper();
    let first = *values
        .value(1)
        .map_err(|error| error.into_failure("ShapeFix_ComposeShell::GetPatchIndex"))?;
    let last = *values
        .value(upper)
        .map_err(|error| error.into_failure("ShapeFix_ComposeShell::GetPatchIndex"))?;
    let period = last - first;
    let shift = if is_closed {
        adjust_to_period(parameter, first, last)
    } else {
        0.0
    };
    let shifted = parameter + shift;
    let mut index = patch_containing(shifted, &values, upper)?;
    let periods = shift / period;
    let rounded_periods = (if periods < 0.0 {
        periods - 0.5
    } else {
        periods + 0.5
    }) as i32;
    index -= rounded_periods * (upper - 1);
    Ok(index)
}

fn patch_containing(
    parameter: f64,
    joints: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    upper: i32,
) -> Result<i32, Failure> {
    for index in 2..upper {
        if parameter
            < *joints
                .value(index)
                .map_err(|error| error.into_failure("ShapeFix_ComposeShell::GetPatchIndex"))?
        {
            return Ok(index - 1);
        }
    }
    Ok(upper - 1)
}

pub(super) fn define_patch(
    wire: &ShapeFixWireSegment,
    code: i32,
    cut_by_u: bool,
    cut_index: i32,
    edge_index: Option<i32>,
) -> Result<(), Failure> {
    let edge = edge_index.unwrap_or(wire.nb_edges());
    if cut_by_u {
        define_u_patch(wire, code, cut_index, edge)?;
    } else {
        define_v_patch(wire, code, cut_index, edge)?;
    }
    Ok(())
}

fn define_u_patch(
    wire: &ShapeFixWireSegment,
    code: i32,
    cut: i32,
    edge: i32,
) -> Result<(), Failure> {
    if code & 1 == 0 {
        wire.define_u_min(edge, cut)?;
    }
    if code & 2 == 0 {
        wire.define_u_max(edge, cut)?;
    }
    Ok(())
}

fn define_v_patch(
    wire: &ShapeFixWireSegment,
    code: i32,
    cut: i32,
    edge: i32,
) -> Result<(), Failure> {
    if code & 2 == 0 {
        wire.define_v_min(edge, cut)?;
    }
    if code & 1 == 0 {
        wire.define_v_max(edge, cut)?;
    }
    Ok(())
}
