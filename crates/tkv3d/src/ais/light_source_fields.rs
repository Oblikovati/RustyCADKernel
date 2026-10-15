//! Read access to the protected fields of `AIS_LightSource` (`AisLightSourceBase`).
use super::AisLightSourceBase;
use crate::select3d::Select3DSensitiveSphere;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkservice::aspect::AspectTypeOfMarker;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectLine3d, Graphic3dAspectMarker3d, Graphic3dCLight, Graphic3dMarkerImage,
};

impl AisLightSourceBase {
    /// `myLightSource`; `base.light_source()`.
    pub fn light_source(&self) -> Handle<Graphic3dCLight> {
        self.parameters.read().light.clone()
    }
    /// `myToDisplayName`; `base.to_display_name_field()`.
    pub fn to_display_name_field(&self) -> bool {
        self.parameters.read().display_name
    }
    /// `myToDisplayRange`; `base.to_display_range_field()`.
    pub fn to_display_range_field(&self) -> bool {
        self.parameters.read().display_range
    }
    /// `mySize`; `base.size_field()`.
    pub fn size_field(&self) -> f64 {
        self.parameters.read().size
    }
    /// `mySensSphereArcSize`; `base.sens_sphere_arc_size()`.
    pub fn sens_sphere_arc_size(&self) -> i32 {
        self.parameters.read().arc_size
    }
    /// `myIsZoomable`; `base.is_zoomable_field()`.
    pub fn is_zoomable_field(&self) -> bool {
        self.parameters.read().zoomable
    }
    /// `myIsDraggable`; `base.is_draggable()`.
    pub fn is_draggable(&self) -> bool {
        self.parameters.read().draggable
    }
    /// `myToSwitchOnClick`; `base.to_switch_on_click_field()`.
    pub fn to_switch_on_click_field(&self) -> bool {
        self.parameters.read().switch_on_click
    }
    /// `myNbArrows`; `base.nb_arrows_field()`.
    pub fn nb_arrows_field(&self) -> i32 {
        self.parameters.read().arrows
    }
    /// `myNbSplitsQuadric`; `base.nb_splits_quadric_field()`.
    pub fn nb_splits_quadric_field(&self) -> i32 {
        self.parameters.read().quadric_splits
    }
    /// `myNbSplitsArrow`; `base.nb_splits_arrow_field()`.
    pub fn nb_splits_arrow_field(&self) -> i32 {
        self.parameters.read().arrow_splits
    }
    /// `myMarkerImages`: of the disabled (0) and enabled (1) light; `base.marker_images()`.
    pub fn marker_images(&self) -> [Handle<Graphic3dMarkerImage>; 2] {
        self.parameters.read().marker_images.clone()
    }
    /// `myMarkerTypes`: of the disabled (0) and enabled (1) light; `base.marker_types()`.
    pub fn marker_types(&self) -> [AspectTypeOfMarker; 2] {
        self.parameters.read().marker_types
    }
    /// `myCodirMarkerType`: the arrow marker co-directional with the camera (X, never changed).
    /// Example: `base.codir_marker_type()`.
    pub fn codir_marker_type(&self) -> AspectTypeOfMarker {
        AspectTypeOfMarker::X
    }
    /// `myOpposMarkerType`: the arrow marker opposite to the camera (O_POINT, never changed).
    /// Example: `base.oppos_marker_type()`.
    pub fn oppos_marker_type(&self) -> AspectTypeOfMarker {
        AspectTypeOfMarker::OPoint
    }
    /// `myDisabledMarkerAspect`; `base.disabled_marker_aspect()`.
    pub fn disabled_marker_aspect(&self) -> Handle<Graphic3dAspectMarker3d> {
        self.visual.read().disabled_marker.clone()
    }
    /// `myArrowLineAspectShadow`; `base.arrow_line_aspect_shadow()`.
    pub fn arrow_line_aspect_shadow(&self) -> Handle<Graphic3dAspectLine3d> {
        self.visual.read().arrow_shadow.clone()
    }
    /// `mySensSphere`; `base.sens_sphere()`.
    pub fn sens_sphere(&self) -> Handle<Select3DSensitiveSphere> {
        self.visual.read().sensitive_sphere.clone()
    }
    /// `myLocTrsfStart`: the local transformation at the start of dragging; `base.loc_trsf_start()`.
    pub fn loc_trsf_start(&self) -> Transform3 {
        self.visual.read().start_transform
    }
}
