//! Native conversion from split parameters into an inherited result array.
use super::{super::SplitCurve, ShapeUpgradeConvertCurve3dToBezier};
use rustycad_tkernel::{
    ncollection::HArray1,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};

pub(super) fn build(tool: &ShapeUpgradeConvertCurve3dToBezier) -> Result<(), Failure> {
    let values = tool.split_values();
    let count = values.sequence(|s| s.len() as i32 - 1);
    let result = HArray1::with_value(1, count, Handle::<dyn GeomCurve>::default())
        .map_err(|error| error.into_failure("ShapeUpgrade_ConvertCurve3dToBezier::Build"))?;
    let result = Handle::new(result);
    tool.parent.set_result_curves(result.clone());
    fill_result(tool, &result)
}

fn fill_result(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    result: &Handle<HArray1<Handle<dyn GeomCurve>>>,
) -> Result<(), Failure> {
    let mut previous_parameter = 0.;
    let mut segment_index = 2;
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
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    result: &Handle<HArray1<Handle<dyn GeomCurve>>>,
    value_index: usize,
    segment_index: usize,
    previous_parameter: &mut f64,
) -> Result<usize, Failure> {
    let parameter = split_value(tool, value_index);
    let segment_index = seek_segment(tool, segment_index, parameter, previous_parameter)?;
    let interval = BuildInterval {
        value_index,
        segment_index,
        parameter,
    };
    let curve = converted_piece(tool, interval, previous_parameter)?;
    store_result(result, value_index, curve)?;
    Ok(segment_index)
}

fn split_value(tool: &ShapeUpgradeConvertCurve3dToBezier, index: usize) -> f64 {
    tool.split_values().sequence(|s| *s.value(index).unwrap())
}

struct BuildInterval {
    value_index: usize,
    segment_index: usize,
    parameter: f64,
}

fn store_result(
    result: &Handle<HArray1<Handle<dyn GeomCurve>>>,
    value_index: usize,
    curve: Handle<dyn GeomCurve>,
) -> Result<(), Failure> {
    result
        .change_array1()
        .set_value(value_index as i32 - 1, curve)
        .map_err(|error| error.into_failure("ShapeUpgrade_ConvertCurve3dToBezier::Build"))
}

fn seek_segment(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
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
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    interval: BuildInterval,
    previous_parameter: &mut f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let source = source_segment(tool, interval.segment_index)?;
    let bezier_handle = source.cast::<GeomBezierCurve>();
    let Some(bezier) = bezier_handle.get() else {
        return source.copy().map(|copy| copy.cast());
    };
    segment_bezier(tool, bezier, interval, previous_parameter)
}

fn source_segment(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    segment_index: usize,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    tool.state
        .read()
        .segments
        .sequence(|s| s.value(segment_index - 1).cloned())
}

fn segment_bezier(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    bezier: &GeomBezierCurve,
    interval: BuildInterval,
    previous_parameter: &mut f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let start = tool
        .split_values()
        .sequence(|s| *s.value(interval.value_index - 1).unwrap());
    let fact = tool.split_params().sequence(|s| {
        *s.value(interval.segment_index).unwrap() - *s.value(interval.segment_index - 1).unwrap()
    });
    let length = (interval.parameter - start) / fact;
    let curve = Handle::new(GeomBezierCurve::copy_of(bezier)?);
    curve.segment(*previous_parameter, *previous_parameter + length)?;
    *previous_parameter += length;
    Ok(curve.cast())
}

fn missing_segment(index: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!(
            "ShapeUpgrade_ConvertCurve3dToBezier::Build: segment {index}; expected a computed segment"
        ),
        "",
    )
}
