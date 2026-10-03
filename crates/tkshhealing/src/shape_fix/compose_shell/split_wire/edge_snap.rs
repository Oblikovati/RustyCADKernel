use super::super::ShapeFixComposeShell;
use super::edge_state::EdgeSplitState;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Point3};

pub(super) struct CutVertex {
    pub(super) vertex: Vertex,
    pub(super) point: Point3,
    pub(super) parameter: f64,
    pub(super) action: CutAction,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum CutAction {
    Split,
    Skip,
    Finish,
}

pub(super) fn choose_cut_vertex(
    state: &EdgeSplitState,
    parameter: f64,
    grid: Handle<crate::shape_extend::ShapeExtendCompositeSurface>,
    location: rustycad_tkmath::toploc::Location,
    composer: &ShapeFixComposeShell,
    cut_by_u: bool,
    cut_index: i32,
) -> Result<CutVertex, Failure> {
    if (parameter - state.last_parameter).abs() < precision::p_confusion() {
        return Ok(CutVertex::endpoint(
            &state.last_vertex,
            state.last_grid_point,
            parameter,
            CutAction::Finish,
        ));
    }
    if (parameter - state.previous_parameter).abs() < precision::p_confusion() {
        return Ok(CutVertex::endpoint(
            &state.first_vertex,
            state.previous_grid_point,
            parameter,
            CutAction::Skip,
        ));
    }
    let uv = state.pcurve.value(parameter)?;
    let point = grid.value_at(uv)?;
    let snap = EndpointSnapContext {
        state,
        parameter,
        uv,
        point,
        grid: &grid,
        location: &location,
        composer,
        cut_by_u,
        cut_index,
    };
    let last = endpoint_snap(&snap, true)?;
    if let Some(endpoint) = last {
        return Ok(endpoint);
    }
    let first = endpoint_snap(&snap, false)?;
    if let Some(endpoint) = first {
        return Ok(endpoint);
    }
    Ok(CutVertex {
        vertex: Vertex::default(),
        point,
        parameter,
        action: CutAction::Split,
    })
}

impl CutVertex {
    fn endpoint(vertex: &Vertex, point: Point3, parameter: f64, action: CutAction) -> Self {
        Self {
            vertex: vertex.clone(),
            point,
            parameter,
            action,
        }
    }
}

struct EndpointSnapContext<'a> {
    state: &'a EdgeSplitState,
    parameter: f64,
    uv: Point2,
    point: Point3,
    grid: &'a Handle<crate::shape_extend::ShapeExtendCompositeSurface>,
    location: &'a rustycad_tkmath::toploc::Location,
    composer: &'a ShapeFixComposeShell,
    cut_by_u: bool,
    cut_index: i32,
}

fn endpoint_snap(
    context: &EndpointSnapContext<'_>,
    is_last: bool,
) -> Result<Option<CutVertex>, Failure> {
    let endpoint = endpoint_values(context.state, is_last);
    if !near_endpoint(context, &endpoint)? {
        return Ok(None);
    }
    let action = if is_last {
        CutAction::Finish
    } else {
        CutAction::Skip
    };
    Ok(Some(CutVertex::endpoint(
        endpoint.vertex,
        endpoint.grid_point,
        context.parameter,
        action,
    )))
}

struct EndpointData<'a> {
    vertex: &'a Vertex,
    point: Point3,
    grid_point: Point3,
    uv: Point2,
    tolerance: f64,
    parameter: f64,
}

fn endpoint_values(state: &EdgeSplitState, is_last: bool) -> EndpointData<'_> {
    if is_last {
        return EndpointData {
            vertex: &state.last_vertex,
            point: state.last_point,
            grid_point: state.last_grid_point,
            uv: state.last_uv,
            tolerance: state.last_vertex_tolerance,
            parameter: state.last_parameter,
        };
    }
    EndpointData {
        vertex: &state.first_vertex,
        point: state.first_point,
        grid_point: state.previous_grid_point,
        uv: state.previous_uv,
        tolerance: state.first_vertex_tolerance,
        parameter: state.previous_parameter,
    }
}

fn near_endpoint(
    context: &EndpointSnapContext<'_>,
    endpoint: &EndpointData<'_>,
) -> Result<bool, Failure> {
    if context.point.distance(&endpoint.point) > endpoint.tolerance {
        return Ok(false);
    }
    let curve_parameter = interpolate_3d_parameter(context.state, context.parameter);
    if !check_by_curve3d(
        context.state,
        &endpoint.point,
        curve_parameter,
        context.location,
        endpoint.tolerance + 2.0 * precision::confusion(),
    )? {
        return Ok(false);
    }
    let midpoint_parameter = 0.5 * (context.parameter + endpoint.parameter);
    let midpoint_uv = context.state.pcurve.value(midpoint_parameter)?;
    if endpoint
        .grid_point
        .distance(&context.grid.value_at(midpoint_uv)?)
        > endpoint.tolerance
    {
        return Ok(false);
    }
    let (ures, vres) = endpoint_resolutions(
        context.composer,
        context.cut_by_u,
        context.cut_index,
        endpoint.tolerance,
    )?;
    Ok(
        is_coincided(endpoint.uv, context.uv, ures, vres, endpoint.tolerance)
            && is_coincided(endpoint.uv, midpoint_uv, ures, vres, endpoint.tolerance),
    )
}

fn check_by_curve3d(
    state: &EdgeSplitState,
    position: &Point3,
    parameter: f64,
    location: &rustycad_tkmath::toploc::Location,
    tolerance: f64,
) -> Result<bool, Failure> {
    let Some(curve) = state.curve_3d.get() else {
        return Ok(true);
    };
    let point = curve
        .value(parameter)?
        .transformed(location.inverted()?.transformation());
    Ok(position.square_distance(&point) <= tolerance * tolerance)
}

fn interpolate_3d_parameter(state: &EdgeSplitState, parameter: f64) -> f64 {
    state.first_3d_parameter
        + (parameter - state.first_parameter) * (state.last_3d_parameter - state.first_3d_parameter)
            / (state.last_parameter - state.first_parameter)
}

fn endpoint_resolutions(
    composer: &ShapeFixComposeShell,
    cut_by_u: bool,
    cut_index: i32,
    tolerance: f64,
) -> Result<(f64, f64), Failure> {
    let fields = composer.fields.read();
    let mut ures = fields.u_resolution;
    let mut vres = fields.v_resolution;
    let joints = if cut_by_u {
        fields.grid.u_joint_values()
    } else {
        fields.grid.v_joint_values()
    };
    let resolution = get_grid_resolution(&joints, cut_index)? / tolerance;
    if cut_by_u {
        ures = ures.min(resolution);
    } else {
        vres = vres.min(resolution);
    }
    Ok((ures, vres))
}

fn get_grid_resolution(
    joints: &crate::shape_extend::JointValues,
    cut_index: i32,
) -> Result<f64, Failure> {
    let values = joints.array1();
    let length = values.length();
    let value = |index: i32| {
        values
            .value(index)
            .copied()
            .map_err(|error| error.into_failure("ShapeFix_ComposeShell::GetGridResolution"))
    };
    let left = if cut_index > 1 {
        value(cut_index)? - value(cut_index - 1)?
    } else {
        value(length)? - value(length - 1)?
    };
    let right = if cut_index < length {
        value(cut_index + 1)? - value(cut_index)?
    } else {
        value(2)? - value(1)?
    };
    Ok(left.min(right) / 3.0)
}

fn is_coincided(first: Point2, second: Point2, ures: f64, vres: f64, tolerance: f64) -> bool {
    (first.x() - second.x()).abs() <= 1.0e-10_f64.max(ures * tolerance)
        && (first.y() - second.y()).abs() <= 1.0e-10_f64.max(vres * tolerance)
}

pub(super) fn adjust_periodic_parameter(parameter: f64, state: &EdgeSplitState) -> f64 {
    let lower = state.first_parameter.min(state.last_parameter);
    let upper = state.first_parameter.max(state.last_parameter);
    if !state.periodic
        || (parameter <= upper + precision::p_confusion()
            && parameter >= lower - precision::p_confusion())
    {
        return parameter;
    }
    parameter
        + crate::shape_analysis::adjust_by_period(
            parameter,
            0.5 * (state.first_parameter + state.last_parameter),
            state.period,
        )
}
