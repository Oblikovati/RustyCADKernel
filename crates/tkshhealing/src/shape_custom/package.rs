// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The package functions of ShapeCustom: `ApplyModifier` (with the sharing of assemblies
//! and the ShapeBuild_ReShape history) and the one-call modifications.
use super::{
    ShapeCustomBSplineRestriction, ShapeCustomConvertToBSpline, ShapeCustomConvertToRevolution,
    ShapeCustomDirectModification, ShapeCustomRestrictionParameters, ShapeCustomSweptToElementary,
    ShapeCustomTrsfModification,
};
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, ReShape};
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;

/// The map of `ApplyModifier` from processed shapes (without location) to their results.
pub type ShapeCustomContext = DataMap<Shape, Shape, ShapeMapHasher>;

/// The package `ShapeCustom` (static functions only).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_custom::ShapeCustom;
/// assert!(ShapeCustom::direct_faces(&Shape::default()).is_err());
/// ```
pub struct ShapeCustom;

/// The arguments of `ShapeCustom::BSplineRestriction` after the shape.
#[derive(Clone)]
pub struct ShapeCustomRestrictionRequest {
    /// Tol3d.
    pub tol_3d: f64,
    /// Tol2d.
    pub tol_2d: f64,
    /// MaxDegree.
    pub max_degree: i32,
    /// MaxNbSegment.
    pub max_nb_segment: i32,
    /// Continuity3d.
    pub continuity_3d: Continuity,
    /// Continuity2d.
    pub continuity_2d: Continuity,
    /// Degree: priority to the degree.
    pub degree: bool,
    /// Rational.
    pub rational: bool,
    /// aParameters.
    pub parameters: Handle<ShapeCustomRestrictionParameters>,
}

impl ShapeCustom {
    /// `ApplyModifier(S, M, context, MD, theProgress, aReShape)`: the shape rebuilt by the
    /// modification; a compound is processed child by child so that shared children
    /// (`context`, keyed without location) are rebuilt once, and the result is recorded in
    /// the optional reshape. The input is returned unchanged when nothing changed, when the
    /// modifier fails or when the progress is cancelled.
    /// Example: `ShapeCustom::apply_modifier(&s, &m, &mut context, &mut modifier, &range, &reshape)?`.
    pub fn apply_modifier(
        shape: &Shape,
        modification: &Handle<dyn BRepToolsModification>,
        context: &mut ShapeCustomContext,
        modifier: &mut Modifier,
        progress: &ProgressRange<'_>,
        reshape: &Handle<ShapeBuildReShape>,
    ) -> Result<Shape, Failure> {
        // "protect against INTERNAL/EXTERNAL shapes".
        let forward = shape.oriented(Orientation::Forward);
        if forward.shape_type()? == ShapeEnum::Compound {
            return Self::apply_to_compound(
                shape,
                forward,
                modification,
                context,
                modifier,
                progress,
                reshape,
            );
        }
        let scope = ProgressScope::with_literal(progress, "Modify the Shape", 1.0, false);
        modifier.init(&forward)?;
        modifier.perform(modification, &scope.next(1.0))?;
        if !scope.more() || !modifier.is_done() {
            return Ok(shape.clone());
        }
        let mut result = modifier.modified_shape(&forward)?;
        result.set_orientation(shape.orientation());
        if !forward.is_same(&result) {
            let identity = Location::default();
            context.bind(
                shape.located(&identity, false)?,
                result.located(&identity, false)?,
            );
        }
        if let Some(reshape) = reshape.get() {
            update_shape_build(&forward, modifier, reshape)?;
        }
        Ok(result)
    }

    // "Process COMPOUNDs separately in order to handle sharing in assemblies".
    #[allow(clippy::too_many_arguments)]
    fn apply_to_compound(
        shape: &Shape,
        mut forward: Shape,
        modification: &Handle<dyn BRepToolsModification>,
        context: &mut ShapeCustomContext,
        modifier: &mut Modifier,
        progress: &ProgressRange<'_>,
        reshape: &Handle<ShapeBuildReShape>,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        forward.set_location(&Location::default(), false)?;
        let count = f64::from(forward.nb_children());
        let scope =
            ProgressScope::with_literal(progress, "Applying Modifier For Solids", count, false);
        let mut modified = false;
        let mut children = ShapeIterator::new(&forward, true, true);
        for child in children.by_ref() {
            if !scope.more() {
                break;
            }
            let range = scope.next(1.0);
            let result =
                Self::child_result(&child, modification, context, modifier, &range, reshape)?;
            modified |= !result.is_same(&child);
            builder.add(&mut compound, &result)?;
        }
        if !scope.more() || !modified {
            return Ok(shape.clone());
        }
        forward.set_orientation(shape.orientation());
        let mut result = compound.into_shape();
        result.set_orientation(shape.orientation());
        context.bind(forward, result.clone());
        result.set_location(shape.location(), false)?;
        if let Some(reshape) = reshape.get() {
            ReShape::replace(reshape, shape, &result)?;
        }
        Ok(result)
    }

    fn child_result(
        child: &Shape,
        modification: &Handle<dyn BRepToolsModification>,
        context: &mut ShapeCustomContext,
        modifier: &mut Modifier,
        range: &ProgressRange<'_>,
        reshape: &Handle<ShapeBuildReShape>,
    ) -> Result<Shape, Failure> {
        let without_location = child.located(&Location::default(), false)?;
        let Some(found) = context.seek(&without_location).cloned() else {
            return Self::apply_modifier(child, modification, context, modifier, range, reshape);
        };
        let mut result = found;
        result.set_orientation(child.orientation());
        result.set_location(child.location(), false)?;
        Ok(result)
    }

    fn apply_fresh(
        shape: &Shape,
        modification: Handle<dyn BRepToolsModification>,
    ) -> Result<Shape, Failure> {
        let mut context = ShapeCustomContext::default();
        let mut modifier = Modifier::new(false);
        Self::apply_modifier(
            shape,
            &modification,
            &mut context,
            &mut modifier,
            &ProgressRange::default(),
            &Handle::default(),
        )
    }

    /// `DirectFaces(S)`: the shape without indirect surfaces
    /// (ShapeCustom_DirectModification); `ShapeCustom::direct_faces(&shape)?`.
    pub fn direct_faces(shape: &Shape) -> Result<Shape, Failure> {
        Self::apply_fresh(
            shape,
            Handle::new(ShapeCustomDirectModification::new()).cast(),
        )
    }

    /// `ScaleShape(S, scale)`: the shape scaled about the origin
    /// (ShapeCustom_TrsfModification); `ShapeCustom::scale_shape(&shape, 2.0)?`.
    pub fn scale_shape(shape: &Shape, scale: f64) -> Result<Shape, Failure> {
        let mut transform = Transform3::default();
        transform.set_scale(Point3::new(0., 0., 0.), scale)?;
        Self::apply_fresh(
            shape,
            Handle::new(ShapeCustomTrsfModification::new(transform)).cast(),
        )
    }

    /// `BSplineRestriction(S, Tol3d, Tol2d, MaxDegree, MaxNbSegment, Continuity3d,
    /// Continuity2d, Degree, Rational, aParameters)`: a default restriction with the request
    /// applied through its setters; `ShapeCustom::bspline_restriction(&shape, &request)?`.
    pub fn bspline_restriction(
        shape: &Shape,
        request: &ShapeCustomRestrictionRequest,
    ) -> Result<Shape, Failure> {
        let restriction = ShapeCustomBSplineRestriction::new();
        restriction.set_tol_3d(request.tol_3d);
        restriction.set_tol_2d(request.tol_2d);
        restriction.set_max_degree(request.max_degree);
        restriction.set_max_nb_segments(request.max_nb_segment);
        restriction.set_continuity_3d(request.continuity_3d);
        restriction.set_continuity_2d(request.continuity_2d);
        restriction.set_priority(request.degree);
        restriction.set_conv_rational(request.rational);
        restriction.set_restriction_parameters(&request.parameters);
        Self::apply_fresh(shape, Handle::new(restriction).cast())
    }

    /// `ConvertToRevolution(S)`; `ShapeCustom::convert_to_revolution(&shape)?`.
    pub fn convert_to_revolution(shape: &Shape) -> Result<Shape, Failure> {
        Self::apply_fresh(
            shape,
            Handle::new(ShapeCustomConvertToRevolution::new()).cast(),
        )
    }

    /// `SweptToElementary(S)`; `ShapeCustom::swept_to_elementary(&shape)?`.
    pub fn swept_to_elementary(shape: &Shape) -> Result<Shape, Failure> {
        Self::apply_fresh(
            shape,
            Handle::new(ShapeCustomSweptToElementary::new()).cast(),
        )
    }

    /// `ConvertToBSpline(S, extrMode, revolMode, offsetMode, planeMode)`;
    /// `ShapeCustom::convert_to_bspline(&shape, true, true, true, false)?`.
    pub fn convert_to_bspline(
        shape: &Shape,
        extrusion: bool,
        revolution: bool,
        offset: bool,
        plane: bool,
    ) -> Result<Shape, Failure> {
        let modification = ShapeCustomConvertToBSpline::new();
        modification.set_extrusion_mode(extrusion);
        modification.set_revolution_mode(revolution);
        modification.set_offset_mode(offset);
        modification.set_plane_mode(plane);
        Self::apply_fresh(shape, Handle::new(modification).cast())
    }
}

/// `UpdateHistoryShape`: records the modified image of `shape` (a shape the modifier does
/// not know, Standard_NoSuchObject, has none); true when it was replaced.
fn update_history_shape(
    shape: &Shape,
    modifier: &Modifier,
    reshape: &ShapeBuildReShape,
) -> Result<bool, Failure> {
    let result = match modifier.modified_shape(shape) {
        Ok(result) => result,
        Err(failure) if failure.is_kind("Standard_NoSuchObject") => Shape::default(),
        Err(failure) => return Err(failure),
    };
    if result.is_null() || shape.is_same(&result) {
        return Ok(false);
    }
    ReShape::replace(reshape, shape, &result)?;
    Ok(true)
}

/// `UpdateHistory`: the children (without cumulated orientation) recursively, below
/// each replaced one.
fn update_history(
    shape: &Shape,
    modifier: &Modifier,
    reshape: &ShapeBuildReShape,
) -> Result<(), Failure> {
    for child in ShapeIterator::new(shape, false, true) {
        if update_history_shape(&child, modifier, reshape)? {
            update_history(&child, modifier, reshape)?;
        }
    }
    Ok(())
}

fn update_shape_build(
    shape: &Shape,
    modifier: &Modifier,
    reshape: &ShapeBuildReShape,
) -> Result<(), Failure> {
    update_history_shape(shape, modifier, reshape)?;
    update_history(shape, modifier, reshape)
}
