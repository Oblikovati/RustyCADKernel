//! Native p-curve conversion from split parameters into the base result array.
use super::{super::SplitCurve, ShapeUpgradeConvertCurve2dToBezier};
use rustycad_tkernel::{
    ncollection::HArray1,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};

pub(super) fn build(tool: &ShapeUpgradeConvertCurve2dToBezier) -> Result<(), Failure> {
    let count = tool.split_values().sequence(|s| s.len() as i32 - 1);
    let result = HArray1::with_value(1, count, Handle::<dyn Geom2dCurve>::default())
        .map_err(|error| error.into_failure("ShapeUpgrade_ConvertCurve2dToBezier::Build"))?;
    let result = Handle::new(result);
    tool.parent.set_result_curves(result.clone());
    fill_result(tool, &result)
}

fn fill_result(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    result: &Handle<HArray1<Handle<dyn Geom2dCurve>>>,
) -> Result<(), Failure> {
    let (mut previous_parameter, mut segment_index) = (0., 2);
    let count = tool.split_values().sequence(|s| s.len());
    for value_index in 2..=count {
        segment_index = build_interval(
            tool,
            result,
            value_index,
            segment_index,
            &mut previous_parameter,
        )?;
    }
    Ok(())
}

fn build_interval(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    result: &Handle<HArray1<Handle<dyn Geom2dCurve>>>,
    value_index: usize,
    segment_index: usize,
    previous_parameter: &mut f64,
) -> Result<usize, Failure> {
    let parameter = split_value(tool, value_index);
    let segment_index = seek_segment(tool, segment_index, parameter, previous_parameter)?;
    let curve = converted_piece(
        tool,
        value_index,
        segment_index,
        parameter,
        previous_parameter,
    )?;
    store_result(result, value_index, curve)?;
    Ok(segment_index)
}

fn split_value(tool: &ShapeUpgradeConvertCurve2dToBezier, index: usize) -> f64 {
    tool.split_values().sequence(|s| *s.value(index).unwrap())
}

fn store_result(
    result: &Handle<HArray1<Handle<dyn Geom2dCurve>>>,
    value_index: usize,
    curve: Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    result
        .change_array1()
        .set_value(value_index as i32 - 1, curve)
        .map_err(|error| error.into_failure("ShapeUpgrade_ConvertCurve2dToBezier::Build"))
}

fn seek_segment(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    mut index: usize,
    parameter: f64,
    previous_parameter: &mut f64,
) -> Result<usize, Failure> {
    let params = tool.split_params();
    while index <= params.sequence(|s| s.len()) {
        if params.sequence(|s| *s.value(index).unwrap()) + precision::p_confusion() > parameter {
            return Ok(index);
        }
        *previous_parameter = 0.;
        index += 1;
    }
    Err(missing_segment(index))
}

fn converted_piece(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    value_index: usize,
    segment_index: usize,
    parameter: f64,
    previous_parameter: &mut f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let source = source_segment(tool, segment_index)?;
    let bezier = source.cast::<Geom2dBezierCurve>();
    let bezier = bezier.get().ok_or_else(|| missing_segment(segment_index))?;
    segment_bezier(
        tool,
        bezier,
        value_index,
        segment_index,
        parameter,
        previous_parameter,
    )
}

fn source_segment(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    segment_index: usize,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    tool.state
        .read()
        .segments
        .sequence(|s| s.value(segment_index - 1).cloned())
}

fn segment_bezier(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    bezier: &Geom2dBezierCurve,
    value_index: usize,
    segment_index: usize,
    parameter: f64,
    previous_parameter: &mut f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let start = tool
        .split_values()
        .sequence(|s| *s.value(value_index - 1).unwrap());
    let fact = tool
        .split_params()
        .sequence(|s| *s.value(segment_index).unwrap() - *s.value(segment_index - 1).unwrap());
    let length = (parameter - start) / fact;
    let curve = Handle::new(Geom2dBezierCurve::copy_of(bezier)?);
    curve.segment(*previous_parameter, *previous_parameter + length)?;
    *previous_parameter += length;
    Ok(curve.cast())
}

fn missing_segment(index: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!(
            "ShapeUpgrade_ConvertCurve2dToBezier::Build: segment {index}; expected a computed segment"
        ),
        "",
    )
}
