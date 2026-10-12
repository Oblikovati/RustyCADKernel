//! Concrete owned ColoredShape lifecycle; rendering/selection are explicitly unported.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::exceptions::not_implemented;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
/// Native per-subshape appearance owner; `let object=AisColoredShape::new(&shape);`.
pub struct AisColoredShape {
    transient: StandardTransient,
    base: AisColoredShapeBase,
}
rustycad_tkernel::impl_transient!(AisColoredShape,"AIS_ColoredShape",dyn AisShapeSource,transient,
    interfaces:[dyn AisColoredShapeSource,dyn AisShapeSource,dyn AisInteractiveObject,
    dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisColoredShape {
    /// Retain topology and initialize native custom-map/line aliases; `AisColoredShape::new(&shape)`.
    pub fn new(shape: &Shape) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisColoredShapeBase::new(shape),
        });
        prs_mgr_presentable_bind(&object.clone().cast());
        object
    }
}
impl AisColoredShapeSource for AisColoredShape {
    fn colored_shape_base(&self) -> &AisColoredShapeBase {
        // All runtime projections access the same actual subclass fields.
        &self.base
    }
}
impl AisShapeSource for AisColoredShape {
    fn shape_base(&self) -> &AisShapeBase {
        // There is one owned native parent, not a copied presentation facade.
        &self.base.shape
    }
}
impl AisInteractiveObject for AisColoredShape {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // Parent field ownership is shared with the exact AIS_Shape projection.
        &self.base.shape.interactive
    }
    fn signature(&self) -> i32 {
        // AIS_ColoredShape inherits native AIS_Shape signature zero.
        0
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        // Native shape categorization is independent of the custom map.
        AisKindOfInteractive::Shape
    }
}
impl PrsMgrPresentableObject for AisColoredShape {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Native invalidation flags belong to this inherited actual parent.
        &self.base.shape.interactive.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        // The genuine inherited AIS_Shape predicate includes modes zero through two.
        (0..=2).contains(&mode)
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        _: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        Err(not_implemented(&format!(
            "AIS_ColoredShape Compute mode={mode}; expected ported per-subshape renderer"
        )))
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisColoredShape {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // No substitute selection fields are allocated by the lifecycle slice.
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, mode: i32) -> Result<(), Failure> {
        Err(not_implemented(&format!(
            "AIS_ColoredShape ComputeSelection mode={mode}; expected ported per-subshape selection"
        )))
    }
}
