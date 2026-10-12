//! Native AIS_Shape state needed by textured presentations; selection and HLR remain partial.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::prs_mgr::PrsMgrTypeOfPresentation3d;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point2};

/// Protected AIS_Shape fields; `object.shape_base().fields.read().uv_repeat`.
#[derive(Clone)]
pub struct AisShapeFields {
    /// `myshape`: retains the actual located and oriented topology reference.
    pub shape: Shape,
    /// `myBB`: lazily recomputed native topology bounds.
    pub bounds: BoundingBox3,
    /// `myUVOrigin`.
    pub uv_origin: Point2,
    /// `myUVRepeat`.
    pub uv_repeat: Point2,
    /// `myUVScale`.
    pub uv_scale: Point2,
    /// `myInitAng`.
    pub initial_angle: f64,
    /// `myCompBB`.
    pub recompute_bounds: bool,
}
impl AisShapeFields {
    fn new(shape: &Shape) -> Self {
        Self {
            shape: shape.clone(),
            bounds: BoundingBox3::default(),
            uv_origin: Point2::new(0., 0.),
            uv_repeat: Point2::new(1., 1.),
            uv_scale: Point2::new(1., 1.),
            initial_angle: 0.,
            recompute_bounds: true,
        }
    }
}
/// Genuine inherited shape state; `AisShapeBase::new(&shape)` initializes projector-dependent presentation.
pub struct AisShapeBase {
    /// Native shape and parameter fields.
    /// Rust drops fields forward: native AIS_Shape releases topology before its AIS parent.
    pub fields: SharedState<AisShapeFields>,
    /// Inherited AIS state, still alive during topology destruction.
    pub interactive: AisInteractiveObjectBase,
}
impl AisShapeBase {
    /// Retain native shape/UV defaults without claiming the whole parent family; `AisShapeBase::new(&shape)`.
    pub fn new(shape: &Shape) -> Self {
        Self {
            interactive: AisInteractiveObjectBase::new(
                PrsMgrTypeOfPresentation3d::ProjectorDependent,
            ),
            fields: SharedState::new(AisShapeFields::new(shape)),
        }
    }
}
/// Native AIS_Shape inheritance seam; `object.shape_base()` exposes real parent state.
pub trait AisShapeSource: AisInteractiveObject {
    /// Embedded parent state; `object.shape_base().fields.read().shape`.
    fn shape_base(&self) -> &AisShapeBase;
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisShapeSource,
    "AIS_Shape",
    dyn AisInteractiveObject
);
