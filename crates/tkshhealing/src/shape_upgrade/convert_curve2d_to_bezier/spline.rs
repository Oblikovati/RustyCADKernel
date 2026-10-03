//! B-spline, conic and arc conversion used by the 2D Bezier converter.
use super::{
    super::SplitCurve,
    ShapeUpgradeConvertCurve2dToBezier,
    compute::{append_segment, null_spline, set_status},
};
use crate::{shape_custom::ShapeCustomCurve2d, shape_extend::ShapeExtendStatus};
use rustycad_tkernel::{
    ncollection::HSequence,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dConic, Geom2dCurve, Geom2dTrimmedCurve,
};
use rustycad_tkgeombase::geom2d_convert::{
    ApproxCurve, BSplineCurveToBezierCurve, curve_to_bspline_curve,
};
use rustycad_tkmath::geom_abs::Shape;

pub(super) fn compute_spline(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let (spline, shift, first, last) = convert_basis(curve, first, last)?;
    let (first, last) = clamp_domain(tool, &spline, first, last)?;
    if let Some(bspline) = spline.get() {
        ShapeCustomCurve2d::simplify_bspline2d(bspline, precision::approximation())?;
    }
    append_spline_arcs(tool, &spline, first, last, shift)?;
    let count = tool.split_values().sequence(|s| s.len() as i32 - 1);
    tool.split_curve_data()
        .with_mut(|fields| fields.nb_curves = count);
    set_status(tool, ShapeExtendStatus::Done1);
    Ok(())
}

fn convert_basis(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(Handle<Geom2dBSplineCurve>, f64, f64, f64), Failure> {
    if curve.cast::<dyn Geom2dConic>().is_some() {
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
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(Handle<Geom2dBSplineCurve>, f64, f64, f64), Failure> {
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast();
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
    let (first, last) = (spline.first_parameter(), spline.last_parameter());
    Ok((spline, shift, first, last))
}

fn clamp_domain(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    spline: &Handle<Geom2dBSplineCurve>,
    first: f64,
    last: f64,
) -> Result<(f64, f64), Failure> {
    let curve = spline.get().ok_or_else(null_spline)?;
    let first = clamp_first(tool, first, curve.first_parameter())?;
    let last = clamp_last(tool, last, curve.last_parameter())?;
    Ok((first, last))
}

fn clamp_first(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
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
    tool: &ShapeUpgradeConvertCurve2dToBezier,
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
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    spline: &Handle<Geom2dBSplineCurve>,
    first: f64,
    last: f64,
    shift: f64,
) -> Result<(), Failure> {
    let split =
        BSplineCurveToBezierCurve::with_range(spline, first, last, precision::p_confusion())?;
    let knots = arc_knots(&split)?;
    tool.split_params().append(first + shift);
    let spline = spline.get().ok_or_else(null_spline)?;
    let mut linear_state = LinearArcState {
        spline,
        new_first: first + shift,
        new_last: first + shift,
    };
    append_arcs(tool, &split, &knots, shift, &mut linear_state)?;
    insert_bezier_knots(tool, knots.as_slice(), shift)
}

fn arc_knots(
    split: &BSplineCurveToBezierCurve,
) -> Result<rustycad_tkernel::ncollection::Array1<'static, f64>, Failure> {
    let mut knots = rustycad_tkg2d::geom2d::arrays::one_based(split.nb_arcs()? + 1)?;
    split.knots(&mut knots)?;
    Ok(knots)
}

fn append_arcs(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    split: &BSplineCurveToBezierCurve,
    knots: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    shift: f64,
    state: &mut LinearArcState<'_>,
) -> Result<(), Failure> {
    for index in 1..=split.nb_arcs()? {
        append_arc(tool, split, knots, index, shift, state)?;
    }
    Ok(())
}

struct LinearArcState<'a> {
    spline: &'a Geom2dBSplineCurve,
    new_first: f64,
    new_last: f64,
}

fn append_arc(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    split: &BSplineCurveToBezierCurve,
    knots: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    index: i32,
    shift: f64,
    state: &mut LinearArcState<'_>,
) -> Result<(), Failure> {
    let next = *knots
        .value(index + 1)
        .map_err(|error| error.into_failure("ShapeUpgrade_ConvertCurve2dToBezier::Compute"))?
        + shift;
    let previous = tool.split_params().sequence(|s| *s.value(s.len()).unwrap());
    if next - previous <= precision::p_confusion() {
        return Ok(());
    }
    let arc = split.arc(index)?;
    append_bezier_arc(tool, &arc, next, state)?;
    Ok(())
}

fn append_bezier_arc(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
    arc: &Handle<Geom2dBezierCurve>,
    next: f64,
    state: &mut LinearArcState<'_>,
) -> Result<(), Failure> {
    state.new_first = state.new_last;
    state.new_last = next;
    if is_linear_approximation(&arc.cast(), state.new_first, state.new_last)? {
        append_segment(
            tool,
            make_bezier(state.spline, state.new_first, state.new_last)?,
        );
        tool.split_params().append(state.new_last);
        return Ok(());
    }
    append_segment(tool, arc.cast());
    tool.split_params().append(next);
    Ok(())
}

fn is_linear_approximation(
    arc: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<bool, Failure> {
    let Some(line) =
        ShapeCustomCurve2d::convert_to_line2d(arc, first, last, precision::approximation())?
    else {
        return Ok(false);
    };
    Ok(line.deviation <= precision::approximation())
}

fn make_bezier(
    spline: &Geom2dBSplineCurve,
    first: f64,
    last: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut poles = rustycad_tkg2d::geom2d::arrays::one_based(2)?;
    rustycad_tkg2d::geom2d::arrays::put(&mut poles, 1, spline.value(first)?)?;
    rustycad_tkg2d::geom2d::arrays::put(&mut poles, 2, spline.value(last)?)?;
    Ok(Handle::new(Geom2dBezierCurve::new(&poles)?).cast())
}

fn insert_bezier_knots(
    tool: &ShapeUpgradeConvertCurve2dToBezier,
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
