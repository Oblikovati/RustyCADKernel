//! The operators of ShapeProcess_OperLibrary built on ShapeUpgrade_ShapeDivide tools:
//! SplitAngle, ToBezier, SplitContinuity, SplitClosedFaces and SplitClosedEdges.
use super::modifications::registrator;
use super::{operator_messages, shape_context};
use crate::shape_extend::{ShapeExtendMsgRegistrator, ShapeExtendStatus};
use crate::shape_process::{ShapeProcessContext, ShapeProcessShapeContext};
use crate::shape_upgrade::{
    ShapeDivider, ShapeUpgradeShapeConvertToBezier, ShapeUpgradeShapeDivide,
    ShapeUpgradeShapeDivideAngle, ShapeUpgradeShapeDivideClosed,
    ShapeUpgradeShapeDivideClosedEdges, ShapeUpgradeShapeDivideContinuity,
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use std::f64::consts::PI;

/// The common tail: a failed Perform with a FAIL status stops the operator; otherwise the
/// tool context is recorded and its result becomes the context result.
fn finish(
    ctx: &ShapeProcessShapeContext,
    tool: &dyn ShapeDivider,
    messages: &Handle<ShapeExtendMsgRegistrator>,
) -> Result<bool, Failure> {
    let base: &ShapeUpgradeShapeDivide = tool.shape_divide_base();
    if !tool.perform(true)? && base.status_flag(ShapeExtendStatus::Fail) {
        return Ok(false);
    }
    ctx.record_modification_reshape(&base.context(), messages)?;
    ctx.set_result(&base.result());
    Ok(true)
}

/// The operator SplitAngle: Angle (2 PI) and MaxTolerance (1).
pub(super) fn split_angle(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let tool =
        ShapeUpgradeShapeDivideAngle::with_shape(ctx.real_val("Angle", 2. * PI)?, ctx.result());
    tool.set_max_tolerance(ctx.real_val("MaxTolerance", 1.)?);
    tool.set_msg_registrator(&registrator(&messages));
    finish(ctx, &tool, &messages)
}

/// The operator ToBezier: the conversion modes, the tolerances and the edge mode.
pub(super) fn to_bezier(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let modes = BezierModes::read(ctx);
    let tool = ShapeUpgradeShapeConvertToBezier::new(ctx.result());
    tool.set_msg_registrator(&registrator(&messages));
    modes.apply(&tool);
    if let Some(tolerance) = ctx.get_real("MaxTolerance")? {
        tool.set_max_tolerance(tolerance);
    }
    if let Some(tolerance) = ctx.get_real("MinCurveLength")? {
        tool.set_min_tolerance(tolerance);
    }
    if let Some(edge_mode) = ctx.get_boolean("EdgeMode") {
        tool.set_edge_mode(i32::from(edge_mode));
    }
    finish(ctx, &tool, &messages)
}

/// The boolean parameters of ToBezier, read in the native order.
struct BezierModes {
    c3d: bool,
    c2d: bool,
    surface: bool,
    line: bool,
    circle: bool,
    conic: bool,
    segment: bool,
    plane: bool,
    revolution: bool,
    extrusion: bool,
    bspline: bool,
}

impl BezierModes {
    fn read(ctx: &ShapeProcessShapeContext) -> Self {
        Self {
            c3d: ctx.boolean_val("Curve3dMode", false),
            c2d: ctx.boolean_val("Curve2dMode", false),
            surface: ctx.boolean_val("SurfaceMode", false),
            line: ctx.boolean_val("Line3dMode", true),
            circle: ctx.boolean_val("Circle3dMode", true),
            conic: ctx.boolean_val("Conic3dMode", true),
            segment: ctx.boolean_val("SegmentSurfaceMode", true),
            plane: ctx.boolean_val("PlaneMode", true),
            revolution: ctx.boolean_val("RevolutionMode", true),
            extrusion: ctx.boolean_val("ExtrusionMode", true),
            bspline: ctx.boolean_val("BSplineMode", true),
        }
    }

    fn apply(&self, tool: &ShapeUpgradeShapeConvertToBezier) {
        tool.set_surface_segment_mode(self.segment);
        tool.set_surface_conversion(self.surface);
        tool.set_2d_conversion(self.c2d);
        tool.set_3d_conversion(self.c3d);
        if self.c3d {
            tool.set_3d_line_conversion(self.line);
            tool.set_3d_circle_conversion(self.circle);
            tool.set_3d_conic_conversion(self.conic);
        }
        if self.surface {
            tool.set_plane_mode(self.plane);
            tool.set_revolution_mode(self.revolution);
            tool.set_extrusion_mode(self.extrusion);
            tool.set_bspline_mode(self.bspline);
        }
    }
}

/// The operator SplitContinuity: the tolerances, the three criteria and MaxTolerance.
pub(super) fn split_continuity(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let tolerance = ctx.real_val("Tolerance3d", 1.0e-7)?;
    let tolerance_2d = ctx.real_val("Tolerance2d", 1.0e-9)?;
    let curve = ctx.continuity_val("CurveContinuity", Continuity::C1);
    let surface = ctx.continuity_val("SurfaceContinuity", Continuity::C1);
    let pcurve = ctx.continuity_val("Curve2dContinuity", Continuity::C1);
    let tool = ShapeUpgradeShapeDivideContinuity::new(ctx.result());
    tool.set_boundary_criterion(curve);
    tool.set_surface_criterion(surface);
    tool.set_pcurve_criterion(pcurve);
    tool.set_tolerance(tolerance);
    tool.set_tolerance_2d(tolerance_2d);
    tool.set_msg_registrator(&registrator(&messages));
    if let Some(max) = ctx.get_real("MaxTolerance")? {
        tool.set_max_tolerance(max);
    }
    finish(ctx, &tool, &messages)
}

/// The operator SplitClosedFaces: CloseTolerance, MaxTolerance, NbSplitPoints (1) and
/// SegmentSurfaceMode (true).
pub(super) fn split_closed_faces(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let tool = ShapeUpgradeShapeDivideClosed::new(ctx.result());
    tool.set_msg_registrator(&registrator(&messages));
    if let Some(tolerance) = ctx.get_real("CloseTolerance")? {
        tool.set_precision(tolerance);
    }
    if let Some(max) = ctx.get_real("MaxTolerance")? {
        tool.set_max_tolerance(max);
    }
    let count = ctx.integer_val("NbSplitPoints", 1)?;
    let segment = ctx.get_boolean("SegmentSurfaceMode").unwrap_or(true);
    tool.set_nb_split_points(count);
    tool.set_surface_segment_mode(segment);
    finish(ctx, &tool, &messages)
}

/// The operator SplitClosedEdges: NbSplitPoints (1, ignored natively by the tool).
pub(super) fn split_closed_edges(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let count = ctx.integer_val("NbSplitPoints", 1)?;
    let tool = ShapeUpgradeShapeDivideClosedEdges::new(ctx.result());
    tool.set_nb_split_points(count);
    tool.set_msg_registrator(&registrator(&messages));
    finish(ctx, &tool, &messages)
}
