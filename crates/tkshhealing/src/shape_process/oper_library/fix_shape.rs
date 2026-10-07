//! The operator FixShape of ShapeProcess_OperLibrary: a ShapeFix_Shape configured from
//! the context parameters, run once more without the tail fix when FixTailMode is 1.
use super::modifications::registrator;
use super::{operator_messages, shape_context};
use crate::shape_extend::ShapeExtendMsgRegistrator;
use crate::shape_fix::{FixRoot, ShapeFixFace, ShapeFixShape, ShapeFixWire};
use crate::shape_process::{ShapeProcessContext, ShapeProcessShapeContext};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use std::f64::consts::PI;

type ModeSetter<T> = fn(&T, i32);

/// The operator FixShape.
pub(super) fn fix_shape(
    context: &Handle<ShapeProcessContext>,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let fixer = Handle::new(ShapeFixShape::empty());
    configure_shape(ctx, &fixer, &messages)?;
    configure_face(ctx, &fixer.fix_face_tool())?;
    let wire = fixer.fix_wire_tool();
    configure_wire(ctx, &wire)?;
    let scope = ProgressScope::unnamed(progress, 2., false);
    if wire.fix_tail_mode() == 1 {
        wire.set_fix_tail_mode(0);
        fixer.init(ctx.result());
        fixer.perform_with_progress(&scope.next(1.0))?;
        wire.set_fix_tail_mode(1);
        if scope.user_break() {
            return Ok(false);
        }
        record_if_changed(ctx, &fixer, &messages)?;
    }
    fixer.init(ctx.result());
    fixer.perform_with_progress(&scope.next(1.0))?;
    if scope.user_break() {
        return Ok(false);
    }
    record_if_changed(ctx, &fixer, &messages)?;
    Ok(true)
}

fn record_if_changed(
    ctx: &ShapeProcessShapeContext,
    fixer: &ShapeFixShape,
    messages: &Handle<ShapeExtendMsgRegistrator>,
) -> Result<(), Failure> {
    let result = fixer.shape();
    let has_messages = messages
        .get()
        .is_some_and(|m| m.map_shape(|map| map.extent()) != 0);
    if result != ctx.result() || has_messages {
        ctx.record_modification_reshape(&fixer.context(), messages)?;
        ctx.set_result(&result);
    }
    Ok(())
}

fn configure_shape(
    ctx: &ShapeProcessShapeContext,
    fixer: &ShapeFixShape,
    messages: &Handle<ShapeExtendMsgRegistrator>,
) -> Result<(), Failure> {
    FixRoot::set_msg_registrator(fixer, &registrator(messages));
    FixRoot::set_precision(fixer, ctx.real_val("Tolerance3d", precision::confusion())?);
    FixRoot::set_min_tolerance(
        fixer,
        ctx.real_val("MinTolerance3d", precision::confusion())?,
    );
    FixRoot::set_max_tolerance(
        fixer,
        ctx.real_val("MaxTolerance3d", precision::confusion())?,
    );
    let modes: [(&str, i32, ModeSetter<ShapeFixShape>); 7] = [
        (
            "FixFreeShellMode",
            -1,
            ShapeFixShape::set_fix_free_shell_mode,
        ),
        ("FixFreeFaceMode", -1, ShapeFixShape::set_fix_free_face_mode),
        ("FixFreeWireMode", -1, ShapeFixShape::set_fix_free_wire_mode),
        (
            "FixSameParameterMode",
            -1,
            ShapeFixShape::set_fix_same_parameter_mode,
        ),
        ("FixSolidMode", -1, ShapeFixShape::set_fix_solid_mode),
        (
            "FixVertexPositionMode",
            0,
            ShapeFixShape::set_fix_vertex_position_mode,
        ),
        (
            "FixVertexToleranceMode",
            -1,
            ShapeFixShape::set_fix_vertex_tol_mode,
        ),
    ];
    for (name, default, set) in modes {
        set(fixer, ctx.integer_val(name, default)?);
    }
    let solid = fixer.fix_solid_tool();
    solid.set_fix_shell_mode(ctx.integer_val("FixShellMode", -1)?);
    solid.set_fix_shell_orientation_mode(ctx.integer_val("FixShellOrientationMode", -1)?);
    solid.set_create_open_solid_mode(ctx.boolean_val("CreateOpenSolidMode", true));
    let shell = fixer.fix_shell_tool();
    shell.set_fix_face_mode(ctx.integer_val("FixFaceMode", -1)?);
    shell.set_non_manifold_flag(ctx.is_non_manifold());
    shell.set_fix_orientation_mode(ctx.integer_val("FixFaceOrientationMode", -1)?);
    Ok(())
}

fn configure_face(ctx: &ShapeProcessShapeContext, face: &ShapeFixFace) -> Result<(), Failure> {
    let modes: [(&str, ModeSetter<ShapeFixFace>); 9] = [
        ("FixWireMode", ShapeFixFace::set_fix_wire_mode),
        ("FixOrientationMode", ShapeFixFace::set_fix_orientation_mode),
        (
            "FixAddNaturalBoundMode",
            ShapeFixFace::set_fix_add_natural_bound_mode,
        ),
        (
            "FixMissingSeamMode",
            ShapeFixFace::set_fix_missing_seam_mode,
        ),
        (
            "FixSmallAreaWireMode",
            ShapeFixFace::set_fix_small_area_wire_mode,
        ),
        (
            "RemoveSmallAreaFaceMode",
            ShapeFixFace::set_remove_small_area_face_mode,
        ),
        (
            "FixIntersectingWiresMode",
            ShapeFixFace::set_fix_intersecting_wires_mode,
        ),
        ("FixLoopWiresMode", ShapeFixFace::set_fix_loop_wires_mode),
        ("FixSplitFaceMode", ShapeFixFace::set_fix_split_face_mode),
    ];
    for (name, set) in modes {
        set(face, ctx.integer_val(name, -1)?);
    }
    Ok(())
}

fn configure_wire(ctx: &ShapeProcessShapeContext, wire: &ShapeFixWire) -> Result<(), Failure> {
    wire.set_topology_mode(ctx.boolean_val("ModifyTopologyMode", false));
    wire.set_geometry_mode(ctx.boolean_val("ModifyGeometryMode", true));
    wire.set_closed_mode(ctx.boolean_val("ClosedWireMode", true));
    wire.set_preference_pcurve_mode(ctx.boolean_val("PreferencePCurveMode", true));
    let modes: [(&str, i32, ModeSetter<ShapeFixWire>); 18] = [
        ("FixReorderMode", -1, ShapeFixWire::set_fix_reorder_mode),
        ("FixSmallMode", -1, ShapeFixWire::set_fix_small_mode),
        ("FixConnectedMode", -1, ShapeFixWire::set_fix_connected_mode),
        (
            "FixEdgeCurvesMode",
            -1,
            ShapeFixWire::set_fix_edge_curves_mode,
        ),
        (
            "FixDegeneratedMode",
            -1,
            ShapeFixWire::set_fix_degenerated_mode,
        ),
        ("FixLackingMode", -1, ShapeFixWire::set_fix_lacking_mode),
        (
            "FixSelfIntersectionMode",
            -1,
            ShapeFixWire::set_fix_self_intersection_mode,
        ),
        ("RemoveLoopMode", -1, ShapeFixWire::set_remove_loop_mode),
        (
            "FixReversed2dMode",
            -1,
            ShapeFixWire::set_fix_reversed_2d_mode,
        ),
        (
            "FixRemovePCurveMode",
            -1,
            ShapeFixWire::set_fix_remove_pcurve_mode,
        ),
        (
            "FixRemoveCurve3dMode",
            -1,
            ShapeFixWire::set_fix_remove_curve_3d_mode,
        ),
        (
            "FixAddPCurveMode",
            -1,
            ShapeFixWire::set_fix_add_pcurve_mode,
        ),
        (
            "FixAddCurve3dMode",
            -1,
            ShapeFixWire::set_fix_add_curve_3d_mode,
        ),
        ("FixShiftedMode", -1, ShapeFixWire::set_fix_shifted_mode),
        ("FixSeamMode", -1, ShapeFixWire::set_fix_seam_mode),
        (
            "FixEdgeSameParameterMode",
            -1,
            ShapeFixWire::set_fix_same_parameter_mode,
        ),
        (
            "FixNotchedEdgesMode",
            -1,
            ShapeFixWire::set_fix_notched_edges_mode,
        ),
        ("FixTailMode", 0, ShapeFixWire::set_fix_tail_mode),
    ];
    for (name, default, set) in modes {
        set(wire, ctx.integer_val(name, default)?);
    }
    wire.set_max_tail_angle(ctx.real_val("MaxTailAngle", 0.)? * (PI / 180.));
    wire.set_max_tail_width(ctx.real_val("MaxTailWidth", -1.)?);
    wire.set_fix_self_intersecting_edge_mode(ctx.integer_val("FixSelfIntersectingEdgeMode", -1)?);
    wire.set_fix_intersecting_edges_mode(ctx.integer_val("FixIntersectingEdgesMode", -1)?);
    wire.set_fix_nonadjacent_intersecting_edges_mode(
        ctx.integer_val("FixNonAdjacentIntersectingEdgesMode", -1)?,
    );
    Ok(())
}
