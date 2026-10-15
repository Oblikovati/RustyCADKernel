//! Martensite concrete AIS_TexturedShape, with the inherited AIS_Shape selection and HLR.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{message::Messenger, quantity::Color, standard::*};
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;

/// Native `base_type` identity; `object.cast::<AisTexturedShapeParent>()` retains its AIS_Shape interface.
pub type AisTexturedShapeParent = dyn AisShapeSource;

/// Real native textured presentation object, with bounded inherited shape behavior.
/// Example: `let object=AisTexturedShape::new(&shape); object.set_texture_pix_map(&pixels);`.
pub struct AisTexturedShape {
    transient: StandardTransient,
    base: AisTexturedShapeBase,
}
rustycad_tkernel::impl_transient!(AisTexturedShape,"AIS_TexturedShape",dyn AisShapeSource,transient,interfaces:[dyn AisTexturedShapeSource,dyn AisShapeSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisTexturedShape {
    /// Retain actual topology and initialize native fields; `AisTexturedShape::new(&shape)`.
    pub fn new(shape: &Shape) -> Handle<Self> {
        let base = AisTexturedShapeBase::new(shape);
        Self::from_base(base)
    }
    /// Inject diagnostics and actual texture creation/availability; `AisTexturedShape::with_services(&shape,messenger,factory)`.
    pub fn with_services(
        shape: &Shape,
        messenger: Arc<Messenger>,
        factory: Arc<dyn AisTexturedTextureFactory>,
    ) -> Handle<Self> {
        Self::from_base(AisTexturedShapeBase::with_services(
            shape, messenger, factory,
        ))
    }
    fn from_base(base: AisTexturedShapeBase) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base,
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}
impl AisTexturedShapeSource for AisTexturedShape {
    fn textured_shape_base(&self) -> &AisTexturedShapeBase {
        // Native source hooks share the same retained fields used by concrete Compute.
        &self.base
    }
}
impl AisShapeSource for AisTexturedShape {
    fn shape_base(&self) -> &AisShapeBase {
        // Topology and bounds remain owned by the actual inherited shape base.
        &self.base.shape
    }
}
impl AisInteractiveObject for AisTexturedShape {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // Native AIS ownership must survive casts to the interactive interface.
        &self.base.shape.interactive
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        // The native AIS_Shape identity remains fixed for this concrete subclass.
        AisKindOfInteractive::Shape
    }
    fn signature(&self) -> i32 {
        // Native AIS_Shape signature is retained rather than introducing a port-specific identity.
        0
    }
}
impl PrsMgrPresentableObject for AisTexturedShape {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Presentation-manager binding and Compute operate on the same inherited base.
        &self.base.shape.interactive.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        // Textured mode 3 extends the inherited display-mode range.
        (0..=3).contains(&mode)
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::textured_shape_compute::ais_textured_shape_compute(self, &prs.observe(), mode)
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
        // Use the textured override so existing mode-3 presentations receive native aspect refresh.
        ais_textured_shape_set_color(self, color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        // Native UnsetColor deliberately bypasses copied mode-3 aspect refresh.
        ais_textured_shape_unset_color(self)
    }
    fn set_material(&self, material: &Graphic3dMaterialAspect) -> Result<(), Failure> {
        // Use the textured override so existing mode-3 presentations receive native aspect refresh.
        ais_textured_shape_set_material(self, material)
    }
    fn unset_material(&self) -> Result<(), Failure> {
        // Use the textured override so existing mode-3 presentations receive native aspect refresh.
        ais_textured_shape_unset_material(self)
    }
    fn set_width(&self, value: f64) -> Result<(), Failure> {
        // Use the inherited shape body instead of the simpler presentable-object default.
        ais_shape_set_width(self, value)
    }
    fn unset_width(&self) -> Result<(), Failure> {
        // Use the inherited shape body instead of the simpler presentable-object default.
        ais_shape_unset_width(self)
    }
    fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        // Use the inherited shape body instead of the simpler presentable-object default.
        ais_shape_set_transparency(self, value)
    }
    fn unset_transparency(&self) -> Result<(), Failure> {
        // Use the inherited shape body instead of the simpler presentable-object default.
        ais_shape_unset_transparency(self)
    }
    fn compute_hlr(
        &self,
        projector: &Handle<Graphic3dCamera>,
        trsf: &Handle<rustycad_tkmath::toploc::Datum3>,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        // AIS_TexturedShape inherits the AIS_Shape hidden line presentation.
        super::ais_shape_compute_hlr(self, projector, trsf, prs)
    }
}
impl SelectMgrSelectableObject for AisTexturedShape {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // Selection ownership is retained even while inherited selection building stays partial.
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        // AIS_TexturedShape inherits the AIS_Shape selection.
        super::ais_shape_compute_selection(self, selection, mode)
    }
}
