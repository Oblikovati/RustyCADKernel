//! `AIS_Manipulator`: an interactive object of three axes translating, rotating, scaling or
//! plane-dragging the attached objects.
use super::*;
use crate::prs_mgr::*;
use crate::prs3d::Prs3dShadingAspect;
use crate::select_mgr::SelectMgrSelectableObject;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{
    Failure, Handle, RuntimeTyped, SharedState, StandardTransient, exceptions::program_error,
};
use rustycad_tkmath::gp::{self, Axis1, Axis2, Point3, Transform3};
use rustycad_tkservice::aspect::{AspectInteriorStyle, AspectTypeOfFacingModel};
use rustycad_tkservice::graphic3d::*;

/// `AIS_Manipulator::ManipulatorSkin`: the shaded 3D parts or the flat 2D ones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AisManipulatorSkin {
    /// `ManipulatorSkin_Shaded`.
    #[default]
    Shaded = 0,
    /// `ManipulatorSkin_Flat`.
    Flat = 1,
}

/// `AIS_Manipulator::OptionsForAttach`: how attaching places and sizes the manipulator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AisManipulatorOptionsForAttach {
    /// `AdjustPosition`: to the center of the object box (true by default).
    pub adjust_position: bool,
    /// `AdjustSize`: to the object box (false by default).
    pub adjust_size: bool,
    /// `EnableModes`: activate every manipulation mode (true by default).
    pub enable_modes: bool,
}
impl Default for AisManipulatorOptionsForAttach {
    fn default() -> Self {
        Self {
            adjust_position: true,
            adjust_size: false,
            enable_modes: true,
        }
    }
}
impl AisManipulatorOptionsForAttach {
    /// `SetAdjustPosition`; `options.set_adjust_position(false)`.
    pub fn set_adjust_position(&mut self, apply: bool) -> &mut Self {
        self.adjust_position = apply;
        self
    }
    /// `SetAdjustSize`; `options.set_adjust_size(true)`.
    pub fn set_adjust_size(&mut self, apply: bool) -> &mut Self {
        self.adjust_size = apply;
        self
    }
    /// `SetEnableModes`; `options.set_enable_modes(false)`.
    pub fn set_enable_modes(&mut self, apply: bool) -> &mut Self {
        self.enable_modes = apply;
        self
    }
}

/// `AIS_Manipulator::BehaviorOnTransform`: whether the manipulator follows the transformed
/// objects in each mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AisManipulatorBehaviorOnTransform {
    /// `FollowTranslation`.
    pub follow_translation: bool,
    /// `FollowRotation`.
    pub follow_rotation: bool,
    /// `FollowDragging`.
    pub follow_dragging: bool,
}
impl Default for AisManipulatorBehaviorOnTransform {
    fn default() -> Self {
        Self {
            follow_translation: true,
            follow_rotation: true,
            follow_dragging: true,
        }
    }
}
impl AisManipulatorBehaviorOnTransform {
    /// `SetFollowTranslation`; `behavior.set_follow_translation(false)`.
    pub fn set_follow_translation(&mut self, apply: bool) -> &mut Self {
        self.follow_translation = apply;
        self
    }
    /// `SetFollowRotation`; `behavior.set_follow_rotation(false)`.
    pub fn set_follow_rotation(&mut self, apply: bool) -> &mut Self {
        self.follow_rotation = apply;
        self
    }
    /// `SetFollowDragging`; `behavior.set_follow_dragging(false)`.
    pub fn set_follow_dragging(&mut self, apply: bool) -> &mut Self {
        self.follow_dragging = apply;
        self
    }
}

/// The protected fields of `AIS_Manipulator`.
#[derive(Clone)]
pub struct AisManipulatorFields {
    /// `myAxes`: the three axes.
    pub axes: [AisManipulatorAxis; 3],
    /// `myCenter`: the center sphere.
    pub center: AisManipulatorSphere,
    /// `myPosition`: the location and axes of the manipulator.
    pub position: Axis2,
    /// `myCircle`: the outer circle (flat skin).
    pub circle: AisManipulatorDisk,
    /// `myCircleGroup`.
    pub circle_group: Handle<dyn Graphic3dGroup>,
    /// `mySector`: the sector showing the rotation angle (flat skin).
    pub sector: AisManipulatorDisk,
    /// `mySectorGroup`.
    pub sector_group: Handle<dyn Graphic3dGroup>,
    /// `myCurrentIndex`: the active axis (-1 when none).
    pub current_index: i32,
    /// `myCurrentMode`: the active manipulation mode.
    pub current_mode: AisManipulatorMode,
    /// `mySkinMode`.
    pub skin_mode: AisManipulatorSkin,
    /// `myIsActivationOnDetection`: the modes activated by detection, not selection.
    pub is_activation_on_detection: bool,
    /// `myIsZoomPersistentMode`.
    pub is_zoom_persistent_mode: bool,
    /// `myBehaviorOnTransform`.
    pub behavior_on_transform: AisManipulatorBehaviorOnTransform,
    /// `myStartTrsfs`: the local transformations of the objects at the transformation start.
    pub start_trsfs: Vec<Transform3>,
    /// `myHasStartedTransformation`.
    pub has_started_transformation: bool,
    /// `myStartPosition`.
    pub start_position: Axis2,
    /// `myStartPick`: the picked point at the transformation start.
    pub start_pick: Point3,
    /// `myPrevState`: the previous rotation angle.
    pub prev_state: f64,
    /// `myHighlightAspect`.
    pub highlight_aspect: Handle<Prs3dShadingAspect>,
    /// `myDraggerHighlight`.
    pub dragger_highlight: Handle<Prs3dShadingAspect>,
}

/// `AIS_Manipulator`; `let manipulator = AisManipulator::new()?; manipulator.attach(&object, &options)?;`.
pub struct AisManipulator {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    /// The protected fields.
    pub fields: SharedState<AisManipulatorFields>,
}
rustycad_tkernel::impl_transient!(AisManipulator, "AIS_Manipulator", dyn AisInteractiveObject, transient,
    interfaces: [dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl AisManipulator {
    /// A zoom-persistent shaded manipulator of size 100 at the origin, in the topmost layer.
    /// Example: `AisManipulator::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        Self::with_position(&gp::xoy())
    }
    /// A manipulator placed at a position; `AisManipulator::with_position(&position)?`.
    pub fn with_position(position: &Axis2) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::new(AisManipulatorFields {
                axes: Default::default(),
                center: AisManipulatorSphere::default(),
                position: *position,
                circle: AisManipulatorDisk::default(),
                circle_group: Handle::default(),
                sector: AisManipulatorDisk::default(),
                sector_group: Handle::default(),
                current_index: -1,
                current_mode: AisManipulatorMode::None,
                skin_mode: AisManipulatorSkin::Shaded,
                is_activation_on_detection: false,
                is_zoom_persistent_mode: true,
                behavior_on_transform: AisManipulatorBehaviorOnTransform::default(),
                start_trsfs: Vec::new(),
                has_started_transformation: false,
                start_position: gp::xoy(),
                start_pick: Point3::new(0.0, 0.0, 0.0),
                prev_state: 0.0,
                highlight_aspect: Handle::default(),
                dragger_highlight: Handle::default(),
            }),
        });
        prs_mgr_presentable_bind(&object.cast());
        object.set_infinite_state(true);
        object.set_mutable(true);
        object.set_display_mode(AisDisplayMode::Shaded as i32);
        object.init()?;
        Ok(object)
    }

    /// `init` (protected): the colored axes, the white unlit-specular shading, the highlight
    /// aspects, size 100 and the topmost layer; `manipulator.init()?`.
    pub fn init(&self) -> Result<(), Failure> {
        {
            let mut fields = self.fields.write();
            fields.axes = [
                AisManipulatorAxis::new(&gp::ox(), Color::named(NamedColor::RED), 10.0),
                AisManipulatorAxis::new(&gp::oy(), Color::named(NamedColor::GREEN), 10.0),
                AisManipulatorAxis::new(&gp::oz(), Color::named(NamedColor::BLUE1), 10.0),
            ];
        }
        let mut material = Graphic3dMaterialAspect::default();
        material.set_specular_color(Color::named(NamedColor::BLACK));
        material.set_material_type(Graphic3dTypeOfMaterial::Aspect);
        let drawer = self.attributes();
        drawer.set_shading_aspect(Handle::new(Prs3dShadingAspect::new()?));
        let shading = drawer.shading_aspect();
        shading
            .aspect()
            .set_interior_style(AspectInteriorStyle::Solid);
        shading.set_color(
            Color::named(NamedColor::WHITE),
            AspectTypeOfFacingModel::BothSide,
        )?;
        shading.set_material(&material, AspectTypeOfFacingModel::BothSide);
        let highlight = Prs3dShadingAspect::new()?;
        highlight
            .aspect()
            .set_interior_style(AspectInteriorStyle::Solid);
        highlight
            .aspect()
            .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        highlight.set_color(
            Color::named(NamedColor::AZURE),
            AspectTypeOfFacingModel::BothSide,
        )?;
        let dragger = Prs3dShadingAspect::new()?;
        dragger
            .aspect()
            .set_interior_style(AspectInteriorStyle::Solid);
        dragger
            .aspect()
            .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        dragger.set_transparency(0.5, AspectTypeOfFacingModel::BothSide)?;
        {
            let mut fields = self.fields.write();
            fields.highlight_aspect = Handle::new(highlight);
            fields.dragger_highlight = Handle::new(dragger);
        }
        self.set_size(100.0);
        self.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
        Ok(())
    }

    /// `SetPart`: enables or disables a part of an axis; an index outside 0..=2 is a
    /// `Standard_ProgramError`; `manipulator.set_part(0, AisManipulatorMode::Rotation, false)?`.
    pub fn set_part(
        &self,
        axis_index: i32,
        mode: AisManipulatorMode,
        enabled: bool,
    ) -> Result<(), Failure> {
        if !(0..=2).contains(&axis_index) {
            return Err(program_error(
                "AIS_Manipulator::SetMode(): axis index should be between 0 and 2",
            ));
        }
        let mut fields = self.fields.write();
        let axis = &mut fields.axes[axis_index as usize];
        match mode {
            AisManipulatorMode::Translation => axis.set_translation(enabled),
            AisManipulatorMode::Rotation => axis.set_rotation(enabled),
            AisManipulatorMode::Scaling => axis.set_scaling(enabled),
            AisManipulatorMode::TranslationPlane => axis.set_dragging(enabled),
            AisManipulatorMode::None => {}
        }
        Ok(())
    }
    /// `SetPart(mode, enabled)`: the part of every axis; `manipulator.set_part_all(mode, false)`.
    pub fn set_part_all(&self, mode: AisManipulatorMode, enabled: bool) {
        for index in 0..3 {
            // The indexes are valid.
            let _ = self.set_part(index, mode, enabled);
        }
    }
    /// `SetModeActivationOnDetection`; `manipulator.set_mode_activation_on_detection(true)`.
    pub fn set_mode_activation_on_detection(&self, enable: bool) {
        self.fields.write().is_activation_on_detection = enable;
    }
    /// `IsModeActivationOnDetection`; `manipulator.is_mode_activation_on_detection()`.
    pub fn is_mode_activation_on_detection(&self) -> bool {
        self.fields.read().is_activation_on_detection
    }
    /// `IsAttached`: an owner (the attached objects) is set; `manipulator.is_attached()`.
    pub fn is_attached(&self) -> bool {
        self.has_owner()
    }
    /// `HasActiveMode`; `manipulator.has_active_mode()`.
    pub fn has_active_mode(&self) -> bool {
        self.is_attached() && self.fields.read().current_mode != AisManipulatorMode::None
    }
    /// `HasActiveTransformation`; `manipulator.has_active_transformation()`.
    pub fn has_active_transformation(&self) -> bool {
        self.fields.read().has_started_transformation
    }
    /// `StartTransformation()`: the start transformation of the first object (identity when
    /// none); `manipulator.start_transformation()`.
    pub fn start_transformation(&self) -> Transform3 {
        self.fields
            .read()
            .start_trsfs
            .first()
            .copied()
            .unwrap_or_default()
    }
    /// `StartTransformation(index)`: an index outside the objects is a `Standard_ProgramError`.
    /// Example: `manipulator.start_transformation_at(1)?`.
    pub fn start_transformation_at(&self, index: i32) -> Result<Transform3, Failure> {
        let upper = self.objects_count()?;
        if index < 1 || index > upper {
            return Err(program_error(
                "AIS_Manipulator::StartTransformation(): theIndex is out of bounds",
            ));
        }
        let fields = self.fields.read();
        if fields.start_trsfs.is_empty() {
            return Ok(Transform3::default());
        }
        fields
            .start_trsfs
            .get(index as usize - 1)
            .copied()
            .ok_or_else(|| Failure::out_of_range("AIS_Manipulator::StartTransformation"))
    }
    /// `ZoomPersistence`; `manipulator.zoom_persistence()`.
    pub fn zoom_persistence(&self) -> bool {
        self.fields.read().is_zoom_persistent_mode
    }
    /// `SkinMode`; `manipulator.skin_mode()`.
    pub fn skin_mode(&self) -> AisManipulatorSkin {
        self.fields.read().skin_mode
    }
    /// `SetSkinMode`: the skin, recomputed when changed; `manipulator.set_skin_mode(skin)`.
    pub fn set_skin_mode(&self, skin: AisManipulatorSkin) {
        if self.fields.read().skin_mode != skin {
            self.set_to_update_all();
        }
        self.fields.write().skin_mode = skin;
    }
    /// `ActiveMode`; `manipulator.active_mode()`.
    pub fn active_mode(&self) -> AisManipulatorMode {
        self.fields.read().current_mode
    }
    /// `ActiveAxisIndex`; `manipulator.active_axis_index()`.
    pub fn active_axis_index(&self) -> i32 {
        self.fields.read().current_index
    }
    /// `Position`; `manipulator.position()`.
    pub fn position(&self) -> Axis2 {
        self.fields.read().position
    }
    /// `Size`: the side length of the first axis; `manipulator.size()`.
    pub fn size(&self) -> f32 {
        self.fields.read().axes[0].size()
    }
    /// `SetSize`: the part sizes of every axis, recomputed; `manipulator.set_size(50.0)`.
    pub fn set_size(&self, side_length: f32) {
        for axis in &mut self.fields.write().axes {
            axis.set_size(side_length);
        }
        self.set_to_update_all();
    }
    /// `SetGap`: the gap between the parts of every axis; `manipulator.set_gap(1.0)`.
    pub fn set_gap(&self, value: f32) {
        for axis in &mut self.fields.write().axes {
            axis.set_indent(value);
        }
        self.set_to_update_all();
    }
    /// `SetTransformBehavior`; `manipulator.set_transform_behavior(behavior)`.
    pub fn set_transform_behavior(&self, settings: AisManipulatorBehaviorOnTransform) {
        self.fields.write().behavior_on_transform = settings;
    }
    /// `ChangeTransformBehavior`: edits the behavior in place.
    /// Example: `manipulator.change_transform_behavior(|b| { b.set_follow_rotation(false); })`.
    pub fn change_transform_behavior(
        &self,
        edit: impl FnOnce(&mut AisManipulatorBehaviorOnTransform),
    ) {
        edit(&mut self.fields.write().behavior_on_transform);
    }
    /// `TransformBehavior`; `manipulator.transform_behavior()`.
    pub fn transform_behavior(&self) -> AisManipulatorBehaviorOnTransform {
        self.fields.read().behavior_on_transform
    }
    /// `getGroup` (protected): the group of a part of an axis (null for other indexes or no
    /// mode); `manipulator.get_group(0, AisManipulatorMode::Rotation)`.
    pub fn get_group(&self, index: i32, mode: AisManipulatorMode) -> Handle<dyn Graphic3dGroup> {
        if !(0..=2).contains(&index) {
            return Handle::default();
        }
        let fields = self.fields.read();
        let axis = &fields.axes[index as usize];
        match mode {
            AisManipulatorMode::Translation => axis.translator_group.clone(),
            AisManipulatorMode::Rotation => axis.rotator_group.clone(),
            AisManipulatorMode::Scaling => axis.scaler_group.clone(),
            AisManipulatorMode::TranslationPlane => axis.dragger_group.clone(),
            AisManipulatorMode::None => Handle::default(),
        }
    }
    /// `getHighlightPresentation` (protected): the highlight presentation of the part of a
    /// manipulator owner (null for other owners).
    /// Example: `manipulator.get_highlight_presentation(&owner)`.
    pub fn get_highlight_presentation(
        &self,
        owner: &Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>,
    ) -> Handle<dyn Graphic3dStructureSource> {
        let owner = owner.cast::<AisManipulatorOwner>();
        let Some(owner) = owner.get() else {
            return Handle::default();
        };
        let fields = self.fields.read();
        let Some(axis) = fields.axes.get(owner.index() as usize) else {
            return Handle::default();
        };
        match owner.mode() {
            AisManipulatorMode::Translation => axis.highlight_translator.clone(),
            AisManipulatorMode::Rotation => axis.highlight_rotator.clone(),
            AisManipulatorMode::Scaling => axis.highlight_scaler.clone(),
            AisManipulatorMode::TranslationPlane => axis.highlight_dragger.clone(),
            AisManipulatorMode::None => Handle::default(),
        }
    }
    /// Whether an object is a manipulator (which cannot be attached).
    pub(super) fn is_manipulator(object: &Handle<dyn AisInteractiveObject>) -> bool {
        object.is_kind(Self::static_type())
    }
    /// The axis of a manipulator position: X, Y or the main direction.
    pub(super) fn axis_of(position: &Axis2, index: i32) -> Result<Axis1, Failure> {
        match index {
            0 => Ok(Axis1::new(position.location(), position.x_direction())),
            1 => Ok(Axis1::new(position.location(), position.y_direction())),
            2 => Ok(position.axis()),
            _ => Err(program_error("AIS_Manipulator - Invalid axis index")),
        }
    }
}
