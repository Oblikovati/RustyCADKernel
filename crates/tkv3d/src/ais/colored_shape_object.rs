//! Concrete owned ColoredShape lifecycle and presentation; selection remains in progress.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dMaterialAspect;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
/// Genuine native parent interface; `let parent: Handle<AisColoredShapeParent> = object.cast();`.
pub type AisColoredShapeParent = dyn AisShapeSource;
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
    /// Copy native topology and owned global appearance, leaving custom nodes independent; `AisColoredShape::from_shape(&source)?`.
    pub fn from_shape(
        source: &dyn HandleReference<dyn AisShapeSource>,
    ) -> Result<Handle<Self>, Failure> {
        let shape = source.observe().shape_ref().clone_shape();
        let object = Self::new(&shape);
        drop(shape);
        rustycad_tkernel::standard::catch_native_failure(|| {
            super::colored_shape_copy_appearance::copy_colored_shape_appearance(&object, source)
        })?;
        Ok(object)
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
    fn compute_hlr(
        &self,
        projector: &Handle<rustycad_tkservice::graphic3d::Graphic3dCamera>,
        trsf: &Handle<rustycad_tkmath::toploc::Datum3>,
        prs: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        // AIS_ColoredShape inherits the AIS_Shape hidden line presentation.
        super::ais_shape_compute_hlr(self, projector, trsf, prs)
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        // The genuine inherited AIS_Shape predicate includes modes zero through two.
        (0..=2).contains(&mode)
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::ais_colored_shape_compute(self, prs, mode)
    }
    fn color(&self) -> Color {
        // Native AIS_Shape selects actual shading color using its current facing model.
        super::shape_appearance_queries::shape_color_value(self)
    }
    fn material(&self) -> rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial {
        // Native inherited material queries select the current shading face.
        super::ais_shape_material(self)
    }
    fn transparency(&self) -> f64 {
        // Preserve native tiny values and NaN instead of the presentable drawer threshold.
        super::ais_shape_transparency(self)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        ais_colored_shape_set_color(self, color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        ais_shape_unset_color(self)
    }
    fn set_width(&self, width: f64) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        ais_colored_shape_set_width(self, width)
    }
    fn unset_width(&self) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        self.set_width(1.0)
    }
    fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        ais_colored_shape_set_transparency(self, value)
    }
    fn unset_transparency(&self) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        self.set_transparency(0.0)
    }
    fn set_material(&self, material: &Graphic3dMaterialAspect) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        ais_colored_shape_set_material(self, material)
    }
    fn unset_material(&self) -> Result<(), Failure> {
        // Preserve native ColoredShape dispatch and inherited appearance policy.
        ais_shape_unset_material(self)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisColoredShape {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // No substitute selection fields are allocated by the lifecycle slice.
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::ais_colored_shape_compute_selection(self, selection, mode)
    }
}
