//! Native Compute branches for 3D curves, including trimmed recursion.
use super::{super::SplitCurve, ShapeUpgradeConvertCurve3dToBezier};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::{
    ncollection::HSequence,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomConic, GeomCurve, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkgeombase::geom_convert::{
    ApproxCurve, BSplineCurveToBezierCurve, curve_to_bspline_curve,
};
use rustycad_tkmath::geom_abs::Shape;

pub(super) fn compute(tool: &ShapeUpgradeConvertCurve3dToBezier) -> Result<(), Failure> {
    clear_outputs(tool);
    let (first, last) = endpoints(tool)?;
    let curve = tool.parent.source_curve();
    if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
        return recurse_trimmed(tool, &trimmed.basis_curve(), first, last);
    }
    if let Some(bezier) = curve.cast::<GeomBezierCurve>().get() {
        return compute_bezier(tool, bezier, first, last);
    }
    if let Some(line) = curve.cast::<GeomLine>().get() {
        return compute_line(tool, line, first, last);
    }
    if skipped_conic(tool, &curve) {
        return compute_unchanged(tool, curve, first, last);
    }
    compute_spline(tool, &curve, first, last)
}

fn clear_outputs(tool: &ShapeUpgradeConvertCurve3dToBezier) {
    let state = tool.state.write();
    state
        .segments
        .change_sequence(|sequence| sequence.clear(None));
    state
        .split_params
        .change_sequence(|sequence| sequence.clear(None));
}

fn endpoints(tool: &ShapeUpgradeConvertCurve3dToBezier) -> Result<(f64, f64), Failure> {
    let values = tool.split_values();
    values.sequence(|s| Ok((*s.value(1)?, *s.value(s.len())?)))
}

fn recurse_trimmed(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    basis: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let nested = ShapeUpgradeConvertCurve3dToBezier::new();
    nested.init_curve_range(basis, first, last)?;
    nested.set_split_values(&tool.split_values())?;
    nested.compute()?;
    transfer_trimmed_result(tool, &nested);
    Ok(())
}

fn transfer_trimmed_result(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    nested: &ShapeUpgradeConvertCurve3dToBezier,
) {
    replace_sequence(&tool.split_values(), &nested.split_values());
    copy_segments(tool, nested);
    let (nested_status, nested_count) = nested
        .split_curve_data()
        .with(|fields| (fields.status, fields.nb_curves));
    tool.split_curve_data().with_mut(|fields| {
        fields.nb_curves = nested_count;
        fields.status |= nested_status;
    });
    copy_parameters(tool, nested);
}

fn compute_bezier(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    bezier: &GeomBezierCurve,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    initialize_direct(tool, first, last)?;
    if first < precision::p_confusion() && last > 1. - precision::p_confusion() {
        append_segment(tool, tool.parent.source_curve());
        set_status(tool, ShapeExtendStatus::Ok);
        return Ok(());
    }
    let copy = Handle::new(GeomBezierCurve::copy_of(bezier)?);
    copy.segment(first, last)?;
    append_segment(tool, copy.cast());
    set_status(tool, ShapeExtendStatus::Done2);
    Ok(())
}

fn compute_line(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    line: &GeomLine,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    initialize_direct(tool, first, last)?;
    if !tool.line_mode() {
        append_segment(tool, tool.parent.source_curve());
        set_status(tool, ShapeExtendStatus::Ok);
        return Ok(());
    }
    let bezier = line_bezier(line, first, last)?;
    append_segment(tool, Handle::new(bezier).cast());
    set_status(tool, ShapeExtendStatus::Done1);
    Ok(())
}

fn line_bezier(line: &GeomLine, first: f64, last: f64) -> Result<GeomBezierCurve, Failure> {
    let poles = [line.value(first)?, line.value(last)?];
    let mut array = one_based(2)?;
    rustycad_tkg2d::geom2d::arrays::put(&mut array, 1, poles[0])?;
    rustycad_tkg2d::geom2d::arrays::put(&mut array, 2, poles[1])?;
    GeomBezierCurve::new(&array)
}

fn skipped_conic(tool: &ShapeUpgradeConvertCurve3dToBezier, curve: &Handle<dyn GeomCurve>) -> bool {
    (curve.cast::<dyn GeomConic>().is_some() && !tool.conic_mode())
        || (curve.cast::<rustycad_tkg3d::geom::GeomCircle>().is_some() && !tool.circle_mode())
}

fn compute_unchanged(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    curve: Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    initialize_direct(tool, first, last)?;
    append_segment(tool, curve);
    set_status(tool, ShapeExtendStatus::Ok);
    Ok(())
}

fn initialize_direct(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
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

fn compute_spline(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let (spline, shift, first, last) = convert_basis(curve, first, last)?;
    let (first, last) = clamp_domain(tool, &spline, first, last)?;
    append_spline_arcs(tool, &spline, first, last, shift)?;
    let count = tool.split_values().sequence(|s| s.len() as i32 - 1);
    tool.split_curve_data()
        .with_mut(|fields| fields.nb_curves = count);
    set_status(tool, ShapeExtendStatus::Done1);
    Ok(())
}

fn convert_basis(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(Handle<GeomBSplineCurve>, f64, f64, f64), Failure> {
    if curve.cast::<dyn GeomConic>().is_some() {
        return approximate_conic(curve, first, last);
    }
    Ok((
        curve_to_bspline_curve(
            curve,
            rustycad_tkmath::convert::ParameterisationType::QuasiAngular,
        )?,
        0.,
        first,
        last,
    ))
}

fn approximate_conic(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(Handle<GeomBSplineCurve>, f64, f64, f64), Failure> {
    let trimmed = Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast();
    let approx = ApproxCurve::from_curve(&trimmed, precision::approximation(), Shape::C1, 100, 6)?;
    let spline = if approx.has_result() {
        approx.curve()
    } else {
        curve_to_bspline_curve(
            &trimmed,
            rustycad_tkmath::convert::ParameterisationType::QuasiAngular,
        )?
    };
    let shift = first - spline.first_parameter();
    let first = spline.first_parameter();
    let last = spline.last_parameter();
    Ok((spline, shift, first, last))
}

fn clamp_domain(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    spline: &Handle<GeomBSplineCurve>,
    first: f64,
    last: f64,
) -> Result<(f64, f64), Failure> {
    let curve = spline.get().ok_or_else(null_spline)?;
    let first = clamp_first(tool, first, curve.first_parameter())?;
    let last = clamp_last(tool, last, curve.last_parameter())?;
    Ok((first, last))
}

fn clamp_first(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    mut parameter: f64,
    lower: f64,
) -> Result<f64, Failure> {
    let tolerance = precision::p_confusion();
    if (parameter - lower).abs() < tolerance {
        return Ok(lower);
    }
    if parameter < lower {
        parameter = lower;
        tool.split_values()
            .change_sequence(|s| s.set_value(1, parameter))?;
    }
    Ok(parameter)
}

fn clamp_last(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    mut parameter: f64,
    upper: f64,
) -> Result<f64, Failure> {
    let tolerance = precision::p_confusion();
    if (parameter - upper).abs() < tolerance {
        return Ok(upper);
    }
    if parameter > upper {
        parameter = upper;
        tool.split_values()
            .change_sequence(|s| s.set_value(s.len(), parameter))?;
    }
    Ok(parameter)
}

fn append_spline_arcs(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    spline: &Handle<GeomBSplineCurve>,
    first: f64,
    last: f64,
    shift: f64,
) -> Result<(), Failure> {
    let split =
        BSplineCurveToBezierCurve::with_range(spline, first, last, precision::p_confusion())?;
    let count = split.nb_arcs()?;
    let knots = one_based(count + 1)?;
    let mut knots = knots;
    split.knots(&mut knots)?;
    tool.split_params().append(first + shift);
    for index in 1..=count {
        append_arc_if_wide(tool, &split, &knots, index, shift)?;
    }
    insert_bezier_knots(tool, knots.as_slice(), shift)?;
    Ok(())
}

fn append_arc_if_wide(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    split: &BSplineCurveToBezierCurve,
    knots: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    index: i32,
    shift: f64,
) -> Result<(), Failure> {
    let next = *knots
        .value(index + 1)
        .map_err(|error| error.into_failure("ShapeUpgrade_ConvertCurve3dToBezier::Compute"))?
        + shift;
    let previous = tool.split_params().sequence(|s| *s.value(s.len()).unwrap());
    if next - previous <= precision::p_confusion() {
        return Ok(());
    }
    append_segment(tool, split.arc(index)?.cast());
    tool.split_params().append(next);
    Ok(())
}

fn insert_bezier_knots(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    knots: &[f64],
    shift: f64,
) -> Result<(), Failure> {
    let values = tool.split_values();
    let mut first = values.sequence(|s| *s.value(1).unwrap());
    let mut index = 2;
    while index <= values.sequence(|s| s.len()) {
        let last = values.sequence(|s| *s.value(index).unwrap());
        index = insert_interval_knots(&values, knots, KnotInterval { first, last, shift }, index)?;
        first = last;
        index += 1;
    }
    Ok(())
}

struct KnotInterval {
    first: f64,
    last: f64,
    shift: f64,
}

fn insert_interval_knots(
    values: &Handle<HSequence<f64>>,
    knots: &[f64],
    interval: KnotInterval,
    mut index: usize,
) -> Result<usize, Failure> {
    let precision = precision::p_confusion();
    for knot in knots.iter().skip(1).map(|value| value + interval.shift) {
        if knot <= interval.first + precision {
            continue;
        }
        if knot >= interval.last - precision {
            break;
        }
        values.change_sequence(|s| s.insert_before(index, knot).map(|_| ()))?;
        index += 1;
    }
    Ok(index)
}

fn append_segment(tool: &ShapeUpgradeConvertCurve3dToBezier, curve: Handle<dyn GeomCurve>) {
    tool.state.read().segments.append(curve);
}

fn copy_segments(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    nested: &ShapeUpgradeConvertCurve3dToBezier,
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

fn copy_parameters(
    tool: &ShapeUpgradeConvertCurve3dToBezier,
    nested: &ShapeUpgradeConvertCurve3dToBezier,
) {
    let parameters = nested
        .split_params()
        .sequence(|s| s.iter().copied().collect::<Vec<_>>());
    for parameter in parameters {
        tool.split_params().append(parameter);
    }
}

fn replace_sequence(destination: &Handle<HSequence<f64>>, source: &Handle<HSequence<f64>>) {
    let values = source.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    destination.change_sequence(|s| {
        s.clear(None);
        for value in values {
            s.append(value);
        }
    });
}

fn set_status(tool: &ShapeUpgradeConvertCurve3dToBezier, status: ShapeExtendStatus) {
    tool.split_curve_data()
        .with_mut(|fields| fields.status = ShapeExtend::encode_status(status));
}

fn null_spline() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_ConvertCurve3dToBezier: null B-spline; expected conversion result",
        "",
    )
}
