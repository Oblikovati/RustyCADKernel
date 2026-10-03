//! Native Compute branches for planar curves, including trimmed recursion.
use super::{super::SplitCurve, ShapeUpgradeConvertCurve2dToBezier};
use crate::{
    shape_custom::ShapeCustomCurve2d,
    shape_extend::{ShapeExtend, ShapeExtendStatus},
};
use rustycad_tkernel::{
    ncollection::HSequence,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve,
};

pub(super) fn compute(tool: &ShapeUpgradeConvertCurve2dToBezier) -> Result<(), Failure> {
    clear_outputs(tool);
    let (first, last) = endpoints(tool)?;
    let curve = tool.parent.source_curve();
    if try_linear_bezier(tool, &curve, first, last)? {
        return Ok(());
    }
    if let Some(trimmed) = curve.cast::<Geom2dTrimmedCurve>().get() {
        return recurse_trimmed(tool, &trimmed.basis_curve(), first, last);
    }
    if let Some(bezier) = curve.cast::<Geom2dBezierCurve>().get() {
        return compute_bezier(tool, bezier, first, last);
    }
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        return compute_line(tool, line, first, last);
    }
    super::spline::compute_spline(tool, &curve, first, last)
}

fn clear_outputs(tool: &ShapeUpgradeConvertCurve2dToBezier) {
    let state = tool.state.write();
    state
        .segments
        .change_sequence(|sequence| sequence.clear(None));
    state
        .split_params
        .change_sequence(|sequence| sequence.clear(None));
}

fn endpoints(tool: &ShapeUpgradeConvertCurve2dToBezier) -> Result<(f64, f64), Failure> {
    let values = tool.split_values();
    values.sequence(|s| Ok((*s.value(1)?, *s.value(s.len())?)))
}

fn try_linear_bezier(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<bool, Failure> {
    if !is_spline_or_bezier(curve) {
        return Ok(false);
    }
    let Some(line) = linear_form(curve, first, last)? else {
        return Ok(false);
    };
    if line.deviation > precision::approximation() {
        return Ok(false);
    }
    append_line_result(tool, &line, first, last)?;
    Ok(true)
}

fn append_line_result(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    line: &crate::shape_custom::Line2dConversion,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let bezier = line_bezier(&line.line.cast(), line.first, line.last)?;
    initialize_direct(tool, first, last)?;
    append_segment(tool, Handle::new(bezier).cast());
    set_status(tool, ShapeExtendStatus::Done1);
    Ok(())
}

fn is_spline_or_bezier(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dBSplineCurve>().is_null() || !curve.cast::<Geom2dBezierCurve>().is_null()
}

fn linear_form(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Option<crate::shape_custom::Line2dConversion>, Failure> {
    ShapeCustomCurve2d::convert_to_line2d(curve, first, last, precision::approximation())
}

fn line_bezier(
    line: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Geom2dBezierCurve, Failure> {
    let mut poles = rustycad_tkg2d::geom2d::arrays::one_based(2)?;
    rustycad_tkg2d::geom2d::arrays::put(
        &mut poles,
        1,
        line.get().ok_or_else(null_curve)?.value(first)?,
    )?;
    rustycad_tkg2d::geom2d::arrays::put(
        &mut poles,
        2,
        line.get().ok_or_else(null_curve)?.value(last)?,
    )?;
    Geom2dBezierCurve::new(&poles)
}

fn recurse_trimmed(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    basis: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let nested = ShapeUpgradeConvertCurve2dToBezier::new();
    nested.init_curve_range(basis, first, last)?;
    nested.set_split_values(&tool.split_values())?;
    nested.compute()?;
    replace_values(&tool.split_values(), &nested.split_values());
    copy_nested_segments(tool, &nested);
    copy_nested_params(tool, &nested);
    merge_nested_state(tool, &nested);
    Ok(())
}

fn merge_nested_state(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    nested: &ShapeUpgradeConvertCurve2dToBezier,
) {
    let (status, count) = nested
        .split_curve_data()
        .with(|fields| (fields.status, fields.nb_curves));
    tool.split_curve_data().with_mut(|fields| {
        fields.nb_curves = count;
        fields.status |= status;
    });
}

fn compute_bezier(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    bezier: &Geom2dBezierCurve,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    initialize_direct(tool, first, last)?;
    if first < precision::p_confusion() && last > 1. - precision::p_confusion() {
        append_segment(tool, tool.parent.source_curve());
        set_status(tool, ShapeExtendStatus::Ok);
        return Ok(());
    }
    let curve = Handle::new(Geom2dBezierCurve::copy_of(bezier)?);
    curve.segment(first, last)?;
    append_segment(tool, curve.cast());
    set_status(tool, ShapeExtendStatus::Done2);
    Ok(())
}

fn compute_line(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    _line: &Geom2dLine,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    initialize_direct(tool, first, last)?;
    let curve = tool.parent.source_curve();
    let bezier = line_bezier(&curve, first, last)?;
    append_segment(tool, Handle::new(bezier).cast());
    set_status(tool, ShapeExtendStatus::Done1);
    Ok(())
}

fn initialize_direct(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    tool.split_params().append(first);
    tool.split_params().append(last);
    let count = tool.split_values().sequence(|s| s.len() as i32 - 1);
    tool.split_curve_data()
        .with_mut(|fields| fields.nb_curves = count);
    Ok(())
}

pub(super) fn append_segment(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    curve: Handle<dyn Geom2dCurve>,
) {
    tool.state.read().segments.append(curve);
}

fn copy_nested_segments(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    nested: &ShapeUpgradeConvertCurve2dToBezier,
) {
    let segments = nested
        .state
        .read()
        .segments
        .sequence(|s| s.iter().cloned().collect::<Vec<_>>());
    for segment in segments {
        append_segment(tool, segment);
    }
}

fn copy_nested_params(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    nested: &ShapeUpgradeConvertCurve2dToBezier,
) {
    let params = nested
        .split_params()
        .sequence(|s| s.iter().copied().collect::<Vec<_>>());
    for parameter in params {
        tool.split_params().append(parameter);
    }
}

fn replace_values(destination: &Handle<HSequence<f64>>, source: &Handle<HSequence<f64>>) {
    let values = source.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    destination.change_sequence(|s| {
        s.clear(None);
        for value in values {
            s.append(value);
        }
    });
}

pub(super) fn set_status(tool: &ShapeUpgradeConvertCurve2dToBezier, status: ShapeExtendStatus) {
    tool.split_curve_data()
        .with_mut(|fields| fields.status = ShapeExtend::encode_status(status));
}

pub(super) fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_ConvertCurve2dToBezier: null curve; expected initialized source",
        "",
    )
}

pub(super) fn null_spline() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_ConvertCurve2dToBezier: null B-spline; expected conversion result",
        "",
    )
}
