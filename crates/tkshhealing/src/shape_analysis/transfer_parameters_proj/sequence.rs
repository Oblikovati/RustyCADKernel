//! Sequential projection bounds and the native closed-curve tail repair are intentional.
use super::*;
pub(super) fn transfer(
    tool: &ShapeAnalysisTransferParametersProj,
    state: &ProjectionTransferState,
    parameters: &Handle<HSequence<f64>>,
    to_2d: bool,
) -> Result<Handle<HSequence<f64>>, Failure> {
    let values = parameter_values(parameters)?;
    let bounds = tool.target_bounds(state, to_2d)?;
    let (mut projected, maximum) = project_values(tool, state, &values, to_2d, bounds)?;
    if state.spatial.is_closed()? {
        let last = if to_2d {
            state.adaptor.last_parameter()?
        } else {
            state.spatial.last_parameter()
        };
        repair_closed_tail(&mut projected, maximum, last);
    }
    let result = Handle::new(HSequence::<f64>::default());
    for value in projected {
        result.change_sequence(|s| {
            s.append(projection::clamp_parameter(value, bounds));
        });
    }
    Ok(result)
}
fn project_values(
    tool: &ShapeAnalysisTransferParametersProj,
    state: &ProjectionTransferState,
    values: &[f64],
    to_2d: bool,
    bounds: (f64, f64),
) -> Result<(Vec<f64>, f64), Failure> {
    let mut projected = Vec::with_capacity(values.len());
    let (mut maximum, mut previous) = (bounds.0, bounds.0);
    for &value in values {
        let parameter = projection::segment(tool, state, value, to_2d, (previous, bounds.1))?;
        previous = parameter;
        if previous > bounds.1 {
            previous -= 2. * precision::p_confusion();
        }
        projected.push(parameter);
        if parameter > maximum {
            maximum = parameter;
        }
    }
    Ok((projected, maximum))
}
fn repair_closed_tail(values: &mut [f64], maximum: f64, last: f64) {
    let count = values.len();
    for index in (0..count).rev() {
        if values[index].partial_cmp(&maximum) != Some(std::cmp::Ordering::Less) {
            break;
        }
        values[index] = last - (count - index - 1) as f64 * (2. * precision::p_confusion());
    }
}

fn parameter_values(parameters: &Handle<HSequence<f64>>) -> Result<Vec<f64>, Failure> {
    Ok(parameters
        .get()
        .ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "TransferParametersProj: null parameters; expected HSequence<double>",
                "",
            )
        })?
        .sequence(|sequence| sequence.iter().copied().collect::<Vec<_>>()))
}
