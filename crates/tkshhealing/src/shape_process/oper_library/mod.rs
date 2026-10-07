// Rust translation of ModelingAlgorithms/TKShHealing/ShapeProcess/ShapeProcess_OperLibrary.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeProcess_OperLibrary`: the predefined operators of ShapeProcess (registered by
//! `Init`) and the modifier application they share.
mod fix;
mod fix_shape;
mod modifications;
mod upgrade;

use super::{
    ShapeProcess, ShapeProcessContext, ShapeProcessOperFunc, ShapeProcessOperator,
    ShapeProcessShapeContext, ShapeProcessShapeMap, ShapeProcessUOperator,
};
use crate::shape_extend::{ShapeExtend, ShapeExtendMsgRegistrator};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier};
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;
use std::sync::Once;

/// The package-like class `ShapeProcess_OperLibrary`.
///
/// ```
/// use rustycad_tkshhealing::shape_process::{ShapeProcess, ShapeProcessOperLibrary};
/// ShapeProcessOperLibrary::init().unwrap();
/// assert!(ShapeProcess::find_operator("FixShape").is_some());
/// ```
pub struct ShapeProcessOperLibrary;

/// The native static `done` of `Init`.
static INIT: Once = Once::new();

/// The predefined operators, in the native registration order.
const OPERATORS: [(&str, ShapeProcessOperFunc); 18] = [
    ("DirectFaces", modifications::direct_faces),
    ("SameParameter", fix::same_parameter),
    ("SetTolerance", fix::set_tolerance),
    ("SplitAngle", upgrade::split_angle),
    ("BSplineRestriction", modifications::bspline_restriction),
    ("ElementaryToRevolution", modifications::to_revolution),
    ("SweptToElementary", modifications::swept_to_elementary),
    ("SurfaceToBSpline", modifications::to_bspline),
    ("ToBezier", upgrade::to_bezier),
    ("SplitContinuity", upgrade::split_continuity),
    ("SplitClosedFaces", upgrade::split_closed_faces),
    ("FixWireGaps", fix::fix_wire_gaps),
    ("FixFaceSize", fix::fix_face_size),
    ("DropSmallSolids", fix::drop_small_solids),
    ("DropSmallEdges", fix::merge_small_edges),
    ("FixShape", fix_shape::fix_shape),
    ("SplitClosedEdges", upgrade::split_closed_edges),
    ("SplitCommonVertex", fix::split_common_vertex),
];

impl ShapeProcessOperLibrary {
    /// `Init()`: once per process, loads the Shape Healing messages (`ShapeExtend::Init`)
    /// and registers the predefined operators; `ShapeProcessOperLibrary::init()?`.
    pub fn init() -> Result<(), Failure> {
        let mut outcome = Ok(());
        INIT.call_once(|| {
            outcome = ShapeExtend::init();
            for (name, function) in OPERATORS {
                let operator: Handle<dyn ShapeProcessOperator> =
                    Handle::new(ShapeProcessUOperator::new(function)).cast();
                ShapeProcess::register_operator(name, &operator);
            }
        });
        outcome
    }

    /// `ApplyModifier(S, context, M, map, msg, theMutableInput)`: the shape modified by
    /// `M`; a compound is processed child by child (without location, shared children
    /// once through `map`, and without messages, as natively), any other shape with a
    /// BRepTools_Modifier whose modifications are recorded in the context.
    /// Example: `ShapeProcessOperLibrary::apply_modifier(&s, &ctx, &m, &mut map, &msg, true)?`.
    pub fn apply_modifier(
        shape: &Shape,
        context: &ShapeProcessShapeContext,
        modification: &Handle<dyn BRepToolsModification>,
        map: &mut ShapeProcessShapeMap,
        messages: &Handle<ShapeExtendMsgRegistrator>,
        mutable_input: bool,
    ) -> Result<Shape, Failure> {
        let forward = shape.oriented(Orientation::Forward);
        if forward.shape_type()? == ShapeEnum::Compound {
            return Self::apply_to_compound(
                shape,
                forward,
                context,
                modification,
                map,
                mutable_input,
            );
        }
        let mut modifier = Modifier::with_shape(&forward)?;
        modifier.set_mutable_input(mutable_input);
        modifier.perform(modification, &ProgressRange::default())?;
        context.record_modification_modifier(&forward, &modifier, messages)?;
        Ok(modifier
            .modified_shape(&forward)?
            .oriented(shape.orientation()))
    }

    fn apply_to_compound(
        shape: &Shape,
        forward: Shape,
        context: &ShapeProcessShapeContext,
        modification: &Handle<dyn BRepToolsModification>,
        map: &mut ShapeProcessShapeMap,
        mutable_input: bool,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut modified = false;
        for child in ShapeIterator::new(&forward, true, true) {
            let location = child.location().clone();
            let unlocated = child.located(&Location::default(), false)?;
            let mut result = match map.seek(&unlocated) {
                Some(found) => found.oriented(unlocated.orientation()),
                None => {
                    let result = Self::apply_modifier(
                        &unlocated,
                        context,
                        modification,
                        map,
                        &Handle::default(),
                        mutable_input,
                    )?;
                    map.bind(unlocated.clone(), result.clone());
                    result
                }
            };
            modified |= !result.is_same(&unlocated);
            result.set_location(&location, false)?;
            builder.add(&mut compound, &result)?;
        }
        if !modified {
            return Ok(shape.clone());
        }
        let compound = compound.into_shape();
        map.bind(forward, compound.clone());
        Ok(compound.oriented(shape.orientation()))
    }
}

/// The `occ::down_cast<ShapeProcess_ShapeContext>(context)` that starts every operator.
fn shape_context(context: &Handle<ShapeProcessContext>) -> Handle<ShapeProcessShapeContext> {
    context.cast::<ShapeProcessShapeContext>()
}

/// A new message registrator when the context keeps messages, else a null one.
fn operator_messages(context: &ShapeProcessShapeContext) -> Handle<ShapeExtendMsgRegistrator> {
    if context.messages().is_null() {
        return Handle::default();
    }
    Handle::new(ShapeExtendMsgRegistrator::new())
}
