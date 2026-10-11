//! AIS_TextLabel at OCCT 3d097a0: text placement and retained annotation state.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::{prs_mgr::*, prs3d::Prs3dTextAspect, select_mgr::*};
use rustycad_tkernel::{
    standard::{Handle, SharedState, StandardTransient},
    tcollection::ExtendedString,
};
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::{
    font::FontTextFormatter, graphic3d::Graphic3dPresentationAttributesSource,
};

/// Native protected annotation state; `label.fields.read().has_flipping`.
#[derive(Clone)]
pub struct AisTextLabelFields {
    /// `myFormatter`: null delegates to the standard formatter.
    pub formatter: Handle<FontTextFormatter>,
    /// `myText`: retained UTF-16 text.
    pub text: ExtendedString,
    /// `myOrientation3D`: includes the label's position even in screen mode.
    pub orientation_3d: Axis2,
    /// `myHasOrientation3D`: use a model-space plane.
    pub has_orientation_3d: bool,
    /// `myHasOwnAnchorPoint`: anchor policy overridden by flipping.
    pub has_own_anchor_point: bool,
    /// `myHasFlipping`: orient the plane toward the camera.
    pub has_flipping: bool,
}
/// Interactive annotation with native text and selection placement; `AisTextLabel::new()`.
pub struct AisTextLabel {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    /// Shared native annotation fields.
    pub fields: SharedState<AisTextLabelFields>,
}
rustycad_tkernel::impl_transient!(AisTextLabel,"AIS_TextLabel",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisTextLabel {
    /// Create a yellow serif question-mark label in display mode zero; `AisTextLabel::new()`.
    pub fn new() -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::new(AisTextLabelFields {
                formatter: Handle::default(),
                text: ExtendedString::new("?"),
                orientation_3d: Axis2::default(),
                has_orientation_3d: false,
                has_own_anchor_point: true,
                has_flipping: false,
            }),
        });
        object
            .attributes()
            .set_text_aspect(Handle::new(Prs3dTextAspect::new()));
        object.attributes().set_display_mode(0);
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    /// Read the retained annotation property; `label.text()`.
    pub fn text(&self) -> ExtendedString {
        self.fields.read().text.clone()
    }
    /// Read the retained annotation property; `label.orientation_3d()`.
    pub fn orientation_3d(&self) -> Axis2 {
        self.fields.read().orientation_3d
    }
    /// Read the retained annotation property; `label.has_orientation_3d()`.
    pub fn has_orientation_3d(&self) -> bool {
        self.fields.read().has_orientation_3d
    }
    /// Read the retained annotation property; `label.has_flipping()`.
    pub fn has_flipping(&self) -> bool {
        self.fields.read().has_flipping
    }
    /// Read the retained annotation property; `label.has_own_anchor_point()`.
    pub fn has_own_anchor_point(&self) -> bool {
        self.fields.read().has_own_anchor_point
    }
    /// Read the retained annotation property; `label.text_formatter()`.
    pub fn text_formatter(&self) -> Handle<FontTextFormatter> {
        self.fields.read().formatter.clone()
    }
    /// Assign without scheduling redisplay; `label.set_text(value)`.
    pub fn set_text(&self, value: &ExtendedString) {
        self.fields.write().text = value.clone();
    }
    /// Assign without scheduling redisplay; `label.set_flipping(value)`.
    pub fn set_flipping(&self, value: bool) {
        self.fields.write().has_flipping = value;
    }
    /// Assign without scheduling redisplay; `label.set_own_anchor_point(value)`.
    pub fn set_own_anchor_point(&self, value: bool) {
        self.fields.write().has_own_anchor_point = value;
    }
    /// Assign without scheduling redisplay; `label.set_text_formatter(value)`.
    pub fn set_text_formatter(&self, value: &Handle<FontTextFormatter>) {
        self.fields.write().formatter = value.clone();
    }
    /// Position is the orientation's origin in both modes; `label.position()`.
    pub fn position(&self) -> Point3 {
        self.orientation_3d().location()
    }
    /// Move without changing orientation mode; `label.set_position(point)`.
    pub fn set_position(&self, point: Point3) {
        self.fields.write().orientation_3d.set_location(point);
    }
    /// Enable model-space placement with supplied axes; `label.set_orientation_3d(axes)`.
    pub fn set_orientation_3d(&self, axes: Axis2) {
        let mut fields = self.fields.write();
        fields.has_orientation_3d = true;
        fields.orientation_3d = axes;
    }
    /// Disable plane placement while retaining its origin and axes; `label.unset_orientation_3d()`.
    pub fn unset_orientation_3d(&self) {
        self.fields.write().has_orientation_3d = false;
    }
}
impl AisInteractiveObject for AisTextLabel {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}
