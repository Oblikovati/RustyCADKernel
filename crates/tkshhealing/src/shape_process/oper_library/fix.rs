//! The operators of ShapeProcess_OperLibrary built on ShapeFix tools: SameParameter,
//! SetTolerance, FixWireGaps, FixFaceSize, DropSmallSolids, DropSmallEdges and
//! SplitCommonVertex.
use super::modifications::registrator;
use super::{operator_messages, shape_context};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::ShapeExtendMsgRegistrator;
use crate::shape_fix::{
    FixRoot, ShapeFix, ShapeFixFixSmallFace, ShapeFixFixSmallSolid, ShapeFixShapeTolerance,
    ShapeFixSplitCommonVertex, ShapeFixWireframe,
};
use crate::shape_process::{ShapeProcessContext, ShapeProcessShapeContext};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tktopalgo::brep_lib;

/// Records the reshape and the new result when the result changed.
fn record_changed(
    ctx: &ShapeProcessShapeContext,
    result: &Shape,
    reshape: &Handle<ShapeBuildReShape>,
    messages: &Handle<ShapeExtendMsgRegistrator>,
) -> Result<bool, Failure> {
    if *result != ctx.result() {
        ctx.record_modification_reshape(reshape, messages)?;
        ctx.set_result(result);
    }
    Ok(true)
}

/// The operator SameParameter: Force (false) and Tolerance3d (Precision::Confusion);
/// the messages are recorded through an empty reshape.
pub(super) fn same_parameter(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let force = ctx.boolean_val("Force", false);
    let tolerance = ctx.real_val("Tolerance3d", precision::confusion())?;
    ShapeFix::same_parameter(
        &ctx.result(),
        force,
        tolerance,
        &ProgressRange::default(),
        &registrator(&messages),
    )?;
    if !messages.is_null() {
        ctx.record_modification_reshape(&Handle::new(ShapeBuildReShape::new()), &messages)?;
    }
    Ok(true)
}

/// The operator SetTolerance: with Mode > 0 and a Value, the tolerances limited to
/// [Value / Ratio, Value * Ratio] (Ratio >= 1); then UpdateTolerances and, with a
/// Regularity, EncodeRegularity.
pub(super) fn set_tolerance(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    if ctx.integer_val("Mode", 0)? > 0
        && let Some(value) = ctx.get_real("Value")?
    {
        let ratio = ctx.real_val("Ratio", 1.)?;
        if ratio >= 1. {
            ShapeFixShapeTolerance::new().limit_tolerance(
                &ctx.result(),
                value / ratio,
                value * ratio,
                rustycad_tkg3d::top_abs::ShapeEnum::Shape,
            )?;
        }
    }
    brep_lib::update_tolerances(&ctx.result(), true)?;
    if let Some(regularity) = ctx.get_real("Regularity")? {
        brep_lib::encode_regularity(&ctx.result(), regularity)?;
    }
    Ok(true)
}

/// The operator FixFaceSize: Tolerance, then ShapeFix_FixSmallFace.
pub(super) fn fix_face_size(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let reshape = Handle::new(ShapeBuildReShape::new());
    let fixer = ShapeFixFixSmallFace::default();
    fixer.set_context(&reshape);
    fixer.init(ctx.result());
    fixer.set_msg_registrator(&registrator(&messages));
    if let Some(tolerance) = ctx.get_real("Tolerance")? {
        fixer.set_precision(tolerance);
    }
    fixer.perform()?;
    record_changed(ctx, &fixer.shape(), &reshape, &messages)
}

/// The operator FixWireGaps: Tolerance3d, then ShapeFix_Wireframe::FixWireGaps.
pub(super) fn fix_wire_gaps(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let tolerance = ctx.real_val("Tolerance3d", precision::confusion())?;
    let reshape = Handle::new(ShapeBuildReShape::new());
    let fixer = ShapeFixWireframe::from_shape(&ctx.result());
    fixer.set_msg_registrator(&registrator(&messages));
    fixer.set_context(&reshape);
    fixer.set_precision(tolerance);
    fixer.fix_wire_gaps()?;
    record_changed(ctx, &fixer.shape(), &reshape, &messages)
}

/// The operator DropSmallEdges (native mergesmalledges): Tolerance3d, then
/// ShapeFix_Wireframe::FixSmallEdges; only the reshape is recorded (the result is kept).
pub(super) fn merge_small_edges(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let tolerance = ctx.real_val("Tolerance3d", precision::confusion())?;
    let reshape = Handle::new(ShapeBuildReShape::new());
    let fixer = ShapeFixWireframe::from_shape(&ctx.result());
    fixer.set_context(&reshape);
    fixer.set_precision(tolerance);
    fixer.set_msg_registrator(&registrator(&messages));
    if fixer.fix_small_edges()? {
        ctx.record_modification_reshape(&reshape, &messages)?;
    }
    Ok(true)
}

/// The operator DropSmallSolids: FixMode, VolumeThreshold, WidthFactorThreshold and
/// MergeSolids (false: the small solids are removed).
pub(super) fn drop_small_solids(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_msg_registrator(&registrator(&messages));
    if let Some(mode) = ctx.get_integer("FixMode")? {
        fixer.set_fix_mode(mode);
    }
    if let Some(threshold) = ctx.get_real("VolumeThreshold")? {
        fixer.set_volume_threshold(threshold);
    }
    if let Some(threshold) = ctx.get_real("WidthFactorThreshold")? {
        fixer.set_width_factor_threshold(threshold);
    }
    let merge = ctx.get_boolean("MergeSolids").unwrap_or(false);
    let reshape = Handle::new(ShapeBuildReShape::new());
    let result = if merge {
        fixer.merge(&ctx.result(), &reshape)?
    } else {
        fixer.remove(&ctx.result(), &reshape)?
    };
    record_changed(ctx, &result, &reshape, &messages)
}

/// The operator SplitCommonVertex: ShapeFix_SplitCommonVertex on the result.
pub(super) fn split_common_vertex(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let reshape = Handle::new(ShapeBuildReShape::new());
    let fixer = ShapeFixSplitCommonVertex::new();
    fixer.set_context(&reshape);
    fixer.init(&ctx.result())?;
    fixer.set_msg_registrator(&registrator(&messages));
    fixer.perform()?;
    record_changed(ctx, &fixer.shape()?, &reshape, &messages)
}
