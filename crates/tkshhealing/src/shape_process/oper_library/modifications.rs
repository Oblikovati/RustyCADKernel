//! The operators of ShapeProcess_OperLibrary that apply a ShapeCustom modification:
//! DirectFaces, BSplineRestriction, ElementaryToRevolution, SweptToElementary and
//! SurfaceToBSpline.
use super::{ShapeProcessOperLibrary, operator_messages, shape_context};
use crate::shape_custom::{
    ShapeCustomBSplineRestriction, ShapeCustomBSplineRestrictionSettings,
    ShapeCustomConvertToBSpline, ShapeCustomConvertToRevolution, ShapeCustomDirectModification,
    ShapeCustomModification, ShapeCustomRestrictionParameters, ShapeCustomSweptToElementary,
};
use crate::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};
use crate::shape_process::{ShapeProcessContext, ShapeProcessShapeContext, ShapeProcessShapeMap};
use rustycad_tkbrep::brep_tools::BRepToolsModification;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The common body: the modification (with the operator registrator) applied to the
/// result, recorded, and made the new result.
fn apply<
    M: ShapeCustomModification
        + BRepToolsModification
        + rustycad_tkernel::standard::RuntimeTyped
        + 'static,
>(
    context: &Handle<ShapeProcessContext>,
    build: impl FnOnce(&ShapeProcessShapeContext) -> Result<M, Failure>,
) -> Result<bool, Failure> {
    let ctx = shape_context(context);
    let Some(ctx) = ctx.get() else {
        return Ok(false);
    };
    let messages = operator_messages(ctx);
    let modification = build(ctx)?;
    modification.set_msg_registrator(&registrator(&messages));
    let modification: Handle<dyn BRepToolsModification> = Handle::new(modification).cast();
    let mut map = ShapeProcessShapeMap::default();
    let result = ShapeProcessOperLibrary::apply_modifier(
        &ctx.result(),
        ctx,
        &modification,
        &mut map,
        &messages,
        true,
    )?;
    ctx.record_modification_map(&map, &messages)?;
    ctx.set_result(&result);
    Ok(true)
}

pub(super) fn registrator(
    messages: &Handle<ShapeExtendMsgRegistrator>,
) -> Handle<dyn BasicMsgRegistrator> {
    messages.clone().cast::<dyn BasicMsgRegistrator>()
}

/// The operator DirectFaces.
pub(super) fn direct_faces(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    apply(context, |_| Ok(ShapeCustomDirectModification::new()))
}

/// The operator ElementaryToRevolution.
pub(super) fn to_revolution(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    apply(context, |_| Ok(ShapeCustomConvertToRevolution::new()))
}

/// The operator SweptToElementary.
pub(super) fn swept_to_elementary(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    apply(context, |_| Ok(ShapeCustomSweptToElementary::new()))
}

/// The operator SurfaceToBSpline: LinearExtrusionMode, RevolutionMode and OffsetMode.
pub(super) fn to_bspline(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    apply(context, |ctx| {
        let modification = ShapeCustomConvertToBSpline::new();
        modification.set_extrusion_mode(ctx.boolean_val("LinearExtrusionMode", true));
        modification.set_revolution_mode(ctx.boolean_val("RevolutionMode", true));
        modification.set_offset_mode(ctx.boolean_val("OffsetMode", true));
        Ok(modification)
    })
}

/// The operator BSplineRestriction: the restriction settings and the restriction
/// parameters read from the context.
pub(super) fn bspline_restriction(
    context: &Handle<ShapeProcessContext>,
    _: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    apply(context, |ctx| {
        let settings = ShapeCustomBSplineRestrictionSettings {
            approx_surface: ctx.boolean_val("SurfaceMode", true),
            approx_curve_3d: ctx.boolean_val("Curve3dMode", true),
            approx_curve_2d: ctx.boolean_val("Curve2dMode", true),
            tol_3d: ctx.real_val("Tolerance3d", 0.01)?,
            tol_2d: ctx.real_val("Tolerance2d", 1e-06)?,
            continuity_3d: ctx.continuity_val("Continuity3d", Continuity::C1),
            continuity_2d: ctx.continuity_val("Continuity2d", Continuity::C2),
            max_degree: ctx.integer_val("RequiredDegree", 9)?,
            nb_max_seg: ctx.integer_val("RequiredNbSegments", 10000)?,
            degree_priority: ctx.boolean_val("PreferDegree", true),
            rational: ctx.boolean_val("RationalToPolynomial", false),
        };
        let parameters = Handle::new(ShapeCustomRestrictionParameters::new());
        read_restriction_parameters(ctx, &parameters)?;
        Ok(ShapeCustomBSplineRestriction::with_settings_and_modes(
            settings,
            &parameters,
        ))
    })
}

type FlagSetter = fn(&ShapeCustomRestrictionParameters, bool);

/// The `GetInteger`/`GetBoolean` calls that overwrite a restriction parameter only when
/// the context defines it.
fn read_restriction_parameters(
    ctx: &ShapeProcessShapeContext,
    p: &ShapeCustomRestrictionParameters,
) -> Result<(), Failure> {
    if let Some(value) = ctx.get_integer("MaxDegree")? {
        p.set_g_max_degree(value);
    }
    if let Some(value) = ctx.get_integer("MaxNbSegments")? {
        p.set_g_max_seg(value);
    }
    const FLAGS: [(&str, FlagSetter); 14] = [
        (
            "OffsetSurfaceMode",
            ShapeCustomRestrictionParameters::set_convert_offset_surf,
        ),
        (
            "OffsetCurve3dMode",
            ShapeCustomRestrictionParameters::set_convert_offset_curv3d,
        ),
        (
            "OffsetCurve2dMode",
            ShapeCustomRestrictionParameters::set_convert_offset_curv2d,
        ),
        (
            "LinearExtrusionMode",
            ShapeCustomRestrictionParameters::set_convert_extrusion_surf,
        ),
        (
            "RevolutionMode",
            ShapeCustomRestrictionParameters::set_convert_revolution_surf,
        ),
        (
            "SegmentSurfaceMode",
            ShapeCustomRestrictionParameters::set_segment_surface_mode,
        ),
        (
            "ConvCurve3dMode",
            ShapeCustomRestrictionParameters::set_convert_curve3d,
        ),
        (
            "ConvCurve2dMode",
            ShapeCustomRestrictionParameters::set_convert_curve2d,
        ),
        (
            "BezierMode",
            ShapeCustomRestrictionParameters::set_convert_bezier_surf,
        ),
        (
            "PlaneMode",
            ShapeCustomRestrictionParameters::set_convert_plane,
        ),
        (
            "ConicalSurfMode",
            ShapeCustomRestrictionParameters::set_convert_conical_surf,
        ),
        (
            "CylindricalSurfMode",
            ShapeCustomRestrictionParameters::set_convert_cylindrical_surf,
        ),
        (
            "ToroidalSurfMode",
            ShapeCustomRestrictionParameters::set_convert_toroidal_surf,
        ),
        (
            "SphericalSurfMode",
            ShapeCustomRestrictionParameters::set_convert_spherical_surf,
        ),
    ];
    for (name, set) in FLAGS {
        if let Some(value) = ctx.get_boolean(name) {
            set(p, value);
        }
    }
    Ok(())
}
