use super::{ShapeFixComposeShell, geometry};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Line2, Point2};

const LEFT: i32 = 1;
const RIGHT: i32 = 2;
const BOTH: i32 = LEFT | RIGHT;
const SAMPLE_COUNT: i32 = 5;

pub(super) fn compute_code(
    composer: &ShapeFixComposeShell,
    wire: &ShapeExtendWireData,
    line: Line2,
    begin_edge: i32,
    end_edge: i32,
    begin_parameter: f64,
    end_parameter: f64,
    internal: bool,
) -> Result<i32, Failure> {
    let fields = composer.fields.read().clone();
    let special = special_segment(
        wire,
        begin_edge,
        end_edge,
        begin_parameter,
        end_parameter,
        &fields,
        internal,
    )?;
    let result = sample_segment(
        composer,
        &fields,
        wire,
        line,
        begin_edge,
        end_edge,
        begin_parameter,
        end_parameter,
        special,
    )?;
    finish_code(composer, &fields, result, line)
}

fn special_segment(
    wire: &ShapeExtendWireData,
    begin_edge: i32,
    end_edge: i32,
    begin_parameter: f64,
    end_parameter: f64,
    fields: &super::ComposeShellFields,
    internal: bool,
) -> Result<i32, Failure> {
    let edge = wire.edge(begin_edge)?;
    let same_direction = matches!(
        edge.orientation(),
        Orientation::Forward | Orientation::Internal
    ) == (begin_parameter > end_parameter);
    if begin_edge == end_edge && same_direction {
        return Ok(1);
    }
    Ok(i32::from(
        begin_edge == end_edge
            && begin_parameter == end_parameter
            && (fields.closed_mode || internal),
    ))
}

fn sample_segment(
    composer: &ShapeFixComposeShell,
    fields: &super::ComposeShellFields,
    wire: &ShapeExtendWireData,
    line: Line2,
    begin_edge: i32,
    end_edge: i32,
    begin_parameter: f64,
    end_parameter: f64,
    mut special: i32,
) -> Result<SampleResult, Failure> {
    let mut result = SampleResult::default();
    let mut edge_index = begin_edge;
    loop {
        sample_edge(
            composer,
            fields,
            wire,
            line,
            &mut result,
            edge_index,
            begin_edge,
            end_edge,
            begin_parameter,
            end_parameter,
            special,
        )?;
        if result.stopped {
            break;
        }
        if edge_index == end_edge {
            if special <= 0 {
                break;
            }
            special = -1;
        }
        edge_index = next_edge(edge_index, wire.nb_edges());
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn sample_edge(
    composer: &ShapeFixComposeShell,
    fields: &super::ComposeShellFields,
    wire: &ShapeExtendWireData,
    line: Line2,
    result: &mut SampleResult,
    edge_index: i32,
    begin_edge: i32,
    end_edge: i32,
    begin_parameter: f64,
    end_parameter: f64,
    special: i32,
) -> Result<(), Failure> {
    let edge = wire.edge(edge_index)?;
    let (curve, first, last) =
        ShapeAnalysisEdge::new().pcurve_on_face(&edge, &fields.face, false)?;
    let Some(curve) = curve.get() else {
        mark_failure(composer, ShapeExtendStatus::Fail3);
        return Ok(());
    };
    result.tolerance = composer.limit_tolerance(BRepTool::edge_tolerance(&edge)?);
    let (start, end) = sample_parameters(
        edge_index,
        begin_edge,
        end_edge,
        begin_parameter,
        end_parameter,
        special,
        edge.orientation(),
        first,
        last,
    );
    sample_parameters_on_edge(fields, line, result, curve, start, end)?;
    Ok(())
}

fn sample_parameters_on_edge(
    fields: &super::ComposeShellFields,
    line: Line2,
    result: &mut SampleResult,
    curve: &dyn rustycad_tkg2d::geom2d::Geom2dCurve,
    start: f64,
    end: f64,
) -> Result<(), Failure> {
    let step = (end - start) / (SAMPLE_COUNT - 1) as f64;
    let count = if step.abs() < precision::p_confusion() {
        1
    } else {
        SAMPLE_COUNT
    };
    for index in 0..count {
        let mut point = curve.value(start + step * index as f64)?;
        adjust_sample(fields, line, result, &mut point, index == 0);
        let position = geometry::point_line_position(point, line).0;
        if position == 0 {
            continue;
        }
        if !coincides_with_line(point, line, fields, result.tolerance) {
            if !fields.closed_mode {
                result.code = position;
                result.stopped = true;
                return Ok(());
            }
            result.code |= position;
        }
    }
    Ok(())
}

fn sample_parameters(
    index: i32,
    begin_edge: i32,
    end_edge: i32,
    begin: f64,
    end: f64,
    special: i32,
    orientation: Orientation,
    first: f64,
    last: f64,
) -> (f64, f64) {
    let reversed = orientation == Orientation::Reversed;
    let start = if index == begin_edge && special >= 0 {
        begin
    } else if reversed {
        last
    } else {
        first
    };
    let stop = if index == end_edge && special <= 0 {
        end
    } else if reversed {
        first
    } else {
        last
    };
    (start, stop)
}

fn adjust_sample(
    fields: &super::ComposeShellFields,
    line: Line2,
    result: &mut SampleResult,
    point: &mut Point2,
    first_in_segment: bool,
) {
    if !fields.closed_mode {
        result.previous_point = *point;
        result.begin = false;
        return;
    }
    adjust_closed_axis(
        point,
        line,
        fields.u_closed,
        fields.u_period,
        true,
        result,
        first_in_segment,
    );
    adjust_closed_axis(
        point,
        line,
        fields.v_closed,
        fields.v_period,
        false,
        result,
        first_in_segment,
    );
    result.begin = false;
    result.previous_point = *point;
}

fn adjust_closed_axis(
    point: &mut Point2,
    line: Line2,
    closed: bool,
    period: f64,
    is_u: bool,
    result: &mut SampleResult,
    first_in_segment: bool,
) {
    if !closed || !closed_line_direction(line, is_u) {
        return;
    }
    let value = axis_value(*point, is_u);
    let old_value = axis_value(result.previous_point, is_u);
    let origin = axis_value(line.location(), is_u);
    let shift = if result.begin {
        crate::shape_analysis::adjust_by_period(value, origin, period)
    } else if first_in_segment {
        crate::shape_analysis::adjust_by_period(value - old_value, 0.0, period)
    } else if is_u {
        result.u_shift
    } else {
        result.v_shift
    };
    set_axis_value(point, is_u, value + shift);
    if is_u {
        result.u_shift = shift;
    } else {
        result.v_shift = shift;
    }
}

fn closed_line_direction(line: Line2, is_u: bool) -> bool {
    let component = if is_u {
        line.direction().x()
    } else {
        line.direction().y()
    };
    component.abs() < precision::p_confusion()
}

fn axis_value(point: Point2, is_u: bool) -> f64 {
    if is_u { point.x() } else { point.y() }
}

fn set_axis_value(point: &mut Point2, is_u: bool, value: f64) {
    if is_u {
        *point = Point2::new(value, point.y());
    } else {
        *point = Point2::new(point.x(), value);
    }
}

fn coincides_with_line(
    point: Point2,
    line: Line2,
    fields: &super::ComposeShellFields,
    tolerance: f64,
) -> bool {
    let projected = geometry::project_point_on_line(point, line);
    let u_tol = larger(INTERSECTION_TOLERANCE, fields.u_resolution * tolerance);
    let v_tol = larger(INTERSECTION_TOLERANCE, fields.v_resolution * tolerance);
    (point.x() - projected.x()).abs() <= u_tol && (point.y() - projected.y()).abs() <= v_tol
}

const INTERSECTION_TOLERANCE: f64 = 1.0e-10;
fn larger(first: f64, second: f64) -> f64 {
    if second > first { second } else { first }
}
fn next_edge(index: i32, count: i32) -> i32 {
    if index == count { 1 } else { index + 1 }
}

fn finish_code(
    composer: &ShapeFixComposeShell,
    fields: &super::ComposeShellFields,
    mut sample: SampleResult,
    line: Line2,
) -> Result<i32, Failure> {
    if fields.closed_mode {
        return Ok(close_periodic_code(
            sample.code,
            sample.previous_point,
            sample.begin,
            line,
            fields,
        ));
    }
    if !sample.stopped {
        return Ok(0);
    }
    if sample.code == BOTH {
        sample.code = LEFT;
        mark_failure(composer, ShapeExtendStatus::Fail2);
    }
    Ok(sample.code)
}

fn close_periodic_code(
    code: i32,
    point: Point2,
    begin: bool,
    line: Line2,
    fields: &super::ComposeShellFields,
) -> i32 {
    if code == 0 || begin {
        return code;
    }
    let mut result = code;
    let deviation = geometry::point_line_deviation(point, line);
    if fields.u_closed && closed_line_direction(line, true) {
        result = periodic_axis_code(result, deviation, fields.u_period);
    }
    if fields.v_closed && closed_line_direction(line, false) {
        result = periodic_axis_code(result, deviation, fields.v_period);
    }
    result
}

fn periodic_axis_code(code: i32, deviation: f64, period: f64) -> i32 {
    if (deviation.abs() - period).abs() < 0.1 * period {
        BOTH | if deviation > 0.0 { 4 } else { 0 }
    } else if code == BOTH {
        0
    } else {
        code
    }
}

fn mark_failure(composer: &ShapeFixComposeShell, status: ShapeExtendStatus) {
    let mut fields = composer.fields.write();
    fields.status |= ShapeExtend::encode_status(status);
}

struct SampleResult {
    code: i32,
    stopped: bool,
    begin: bool,
    previous_point: Point2,
    tolerance: f64,
    u_shift: f64,
    v_shift: f64,
}

impl Default for SampleResult {
    fn default() -> Self {
        Self {
            code: 0,
            stopped: false,
            begin: true,
            previous_point: Point2::default(),
            tolerance: 0.0,
            u_shift: 0.0,
            v_shift: 0.0,
        }
    }
}
