//! Native AIS light state for application subclasses; concrete presentations remain unported.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrTypeOfPresentation3d};
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};

pub(super) struct LightSourceParameters {
    pub light: Handle<Graphic3dCLight>,
    pub display_name: bool,
    pub display_range: bool,
    pub size: f64,
    pub arc_size: i32,
    pub zoomable: bool,
    pub draggable: bool,
    pub switch_on_click: bool,
    pub arrows: i32,
    pub quadric_splits: i32,
    pub arrow_splits: i32,
    pub marker_images: [Handle<Graphic3dMarkerImage>; 2],
    pub marker_types: [AspectTypeOfMarker; 2],
}
/// Embeddable native state; `AisLightSourceBase::new(&light)` initializes fields, not presentation aspects.
pub struct AisLightSourceBase {
    /// Inherited interactive state for application subclasses.
    pub interactive: AisInteractiveObjectBase,
    pub(super) parameters: SharedState<LightSourceParameters>,
    pub(super) visual: SharedState<super::light_source_aspects::LightSourceVisual>,
}
impl AisLightSourceBase {
    /// Retain the live light and initialize native fields; `AisLightSourceBase::new(&light)`.
    pub fn new(light: &Handle<Graphic3dCLight>) -> Self {
        let kind = light.light_type();
        Self {
            interactive: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            parameters: SharedState::new(LightSourceParameters::new(light, kind)),
            visual: SharedState::default(),
        }
    }
}
impl LightSourceParameters {
    fn new(light: &Handle<Graphic3dCLight>, kind: Graphic3dTypeOfLightSource) -> Self {
        let zoomable = matches!(
            kind,
            Graphic3dTypeOfLightSource::Positional | Graphic3dTypeOfLightSource::Spot
        );
        let quadric_splits = if kind == Graphic3dTypeOfLightSource::Ambient {
            10
        } else {
            30
        };
        Self {
            light: light.clone(),
            zoomable,
            draggable: kind == Graphic3dTypeOfLightSource::Directional,
            quadric_splits,
            ..Self::default()
        }
    }
}
impl Default for LightSourceParameters {
    fn default() -> Self {
        Self {
            light: Handle::default(),
            display_name: true,
            display_range: true,
            size: 50.0,
            arc_size: 25,
            zoomable: false,
            draggable: false,
            switch_on_click: true,
            arrows: 5,
            quadric_splits: 30,
            arrow_splits: 20,
            marker_images: [Handle::default(), Handle::default()],
            marker_types: [AspectTypeOfMarker::OX, AspectTypeOfMarker::OPoint],
        }
    }
}
/// Real light operations for AIS subclasses. Native drawing/aspect hooks remain the subclass's responsibility.
/// Example: implement `light_source_base()` alongside `AisInteractiveObject` for a light presentation.
pub trait AisLightSourceSource: AisInteractiveObject {
    /// Embedded light state; `object.light_source_base()`.
    fn light_source_base(&self) -> &AisLightSourceBase;
    /// Refresh live presentation aspects after a light change; `object.update_light_aspects()?`.
    fn update_light_aspects(&self) -> Result<(), rustycad_tkernel::standard::Failure>;
    /// Refresh persistence through an overridable native hook; `object.update_light_transform_persistence()?`.
    fn update_light_transform_persistence(
        &self,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        super::ais_light_source_update_persistence(self)
    }
    /// Refresh the light-derived frame through an overridable hook; `object.update_light_local_transformation()?`.
    fn update_light_local_transformation(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
        super::ais_light_source_update_local_transform(self)
    }
    /// Native ambient geometry hook; `object.compute_ambient(&presentation,0)?`.
    fn compute_ambient(
        &self,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        super::light_source_arrows::compute_light_ambient(self, presentation, mode)
    }
    /// Native directional geometry hook; `object.compute_directional(&presentation,0)?`.
    fn compute_directional(
        &self,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        super::light_source_arrows::compute_light_directional(self, presentation, mode)
    }
    /// Native positional geometry hook; `object.compute_positional(&presentation,0)?`.
    fn compute_positional(
        &self,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        super::light_source_geometry::compute_light_positional(self, presentation, mode)
    }
    /// Native spot geometry hook; `object.compute_spot(&presentation,0)?`.
    fn compute_spot(
        &self,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        super::light_source_geometry::compute_light_spot(self, presentation, mode)
    }
    /// Retain the current live light; `object.light()`.
    fn light(&self) -> Handle<Graphic3dCLight> {
        self.light_source_base().parameters.read().light.clone()
    }
    /// Replace the light and mark every presentation dirty, even for identical handles; `object.set_light(&light)`.
    fn set_light(&self, light: &Handle<Graphic3dCLight>) {
        self.light_source_base().parameters.write().light = light.clone();
        self.set_to_update_all();
    }
    /// Set whether directional dragging is allowed without invalidating presentations; `object.set_draggable(false)`.
    fn set_draggable(&self, draggable: bool) {
        self.light_source_base().parameters.write().draggable = draggable;
    }
    /// Whether unmodified left clicks toggle the light; `object.to_switch_on_click()`.
    fn to_switch_on_click(&self) -> bool {
        self.light_source_base().parameters.read().switch_on_click
    }
    /// Toggle click handling without scheduling redisplay; `object.set_switch_on_click(false)`.
    fn set_switch_on_click(&self, enabled: bool) {
        self.light_source_base().parameters.write().switch_on_click = enabled;
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisLightSourceSource,
    "AIS_LightSource",
    dyn AisInteractiveObject
);
