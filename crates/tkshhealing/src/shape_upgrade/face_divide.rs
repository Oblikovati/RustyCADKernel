//! Face subdivision orchestration from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{ShapeUpgradeSplitSurface, ShapeUpgradeTool, ShapeUpgradeWireDivide};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    standard::{Failure, Handle, SharedState, Transient},
};
mod curves;
mod surface;

/// Native protected face subdivision state; `FaceDivideData::default()`.
pub struct FaceDivideData {
    fields: SharedState<FaceDivideFields>,
}
/// Working face, result, modes, and configured criteria.
pub struct FaceDivideFields {
    /// Original supporting face.
    pub face: Face,
    /// Current intermediate/result shape.
    pub result: Shape,
    /// Whether split surfaces are segmented.
    pub segment: bool,
    /// Native ShapeExtend status bits.
    pub status: i32,
    surface_tool: Handle<ShapeUpgradeSplitSurface>,
    wire_tool: Handle<ShapeUpgradeWireDivide>,
}
impl Default for FaceDivideData {
    fn default() -> Self {
        Self {
            fields: SharedState::new(FaceDivideFields {
                face: Face::default(),
                result: Shape::default(),
                segment: true,
                status: 0,
                surface_tool: Handle::new(ShapeUpgradeSplitSurface::new()),
                wire_tool: Handle::new(ShapeUpgradeWireDivide::new()),
            }),
        }
    }
}
impl FaceDivideData {
    /// Read protected state; `state.with(|fields| fields.segment)`.
    pub fn with<R>(&self, read: impl FnOnce(&FaceDivideFields) -> R) -> R {
        read(&self.fields.read())
    }
    /// Update protected state for a derived policy; `state.with_mut(|fields| fields.status = 0)`.
    pub fn with_mut<R>(&self, write: impl FnOnce(&mut FaceDivideFields) -> R) -> R {
        write(&mut self.fields.write())
    }
    fn add_status(&self, status: ShapeExtendStatus) {
        self.with_mut(|fields| fields.status |= ShapeExtend::encode_status(status));
    }
}

/// Virtual surface and boundary subdivision policy; `divider.perform(0.)?`.
pub trait FaceDivider: Transient {
    /// Shared protected state; `divider.face_divide_data()`.
    fn face_divide_data(&self) -> &FaceDivideData;
    /// Shared tolerance and reshape context; `divider.face_divide_base()`.
    fn face_divide_base(&self) -> &ShapeUpgradeTool;
    /// Split surfaces before curves and retain aggregate status; `divider.perform(0.)?`.
    fn perform(&self, area: f64) -> Result<bool, Failure> {
        perform_face_division(self, area)
    }
    /// Split the supporting surface and compose its patch faces; `divider.split_surface(0.)?`.
    fn split_surface(&self, area: f64) -> Result<bool, Failure> {
        surface::split_surface(self, area)
    }
    /// Split every boundary wire through its configured policy; `divider.split_curves()?`.
    fn split_curves(&self) -> Result<bool, Failure> {
        curves::split_curves(self)
    }
    /// Virtual surface criterion selection; `divider.split_surface_tool()`.
    fn split_surface_tool(&self) -> Handle<ShapeUpgradeSplitSurface> {
        self.face_divide_data()
            .with(|fields| fields.surface_tool.clone())
    }
    /// Virtual wire criterion selection; `divider.wire_divide_tool()`.
    fn wire_divide_tool(&self) -> Handle<ShapeUpgradeWireDivide> {
        self.face_divide_data()
            .with(|fields| fields.wire_tool.clone())
    }
}
pub(super) fn perform_face_division(
    tool: &(impl FaceDivider + ?Sized),
    area: f64,
) -> Result<bool, Failure> {
    let state = tool.face_divide_data();
    state.with_mut(|fields| fields.status = 0);
    let face = state.with(|fields| fields.face.clone());
    if face.is_null() {
        return Ok(false);
    }
    let base = tool.face_divide_base();
    if base.context().is_null() {
        base.set_context(&Handle::new(crate::shape_build::ShapeBuildReShape::new()));
    }
    state.with_mut(|fields| fields.result = face.into_shape());
    tool.split_surface(area)?;
    tool.split_curves()?;
    Ok(state.with(|fields| ShapeExtend::decode_status(fields.status, ShapeExtendStatus::Done)))
}

impl_transient_interface!(dyn FaceDivider, "ShapeUpgrade_FaceDivide", ShapeUpgradeTool);

/// Divides a face using configurable surface and wire criteria.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeFaceDivide;
/// let divider = ShapeUpgradeFaceDivide::new(Face::default());
/// assert!(!divider.perform(0.0)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct ShapeUpgradeFaceDivide {
    parent: ShapeUpgradeTool,
    state: FaceDivideData,
}
impl_transient!(ShapeUpgradeFaceDivide, "ShapeUpgrade_FaceDivide", ShapeUpgradeTool, parent, interfaces:[dyn FaceDivider]);
impl FaceDivider for ShapeUpgradeFaceDivide {
    fn face_divide_data(&self) -> &FaceDivideData {
        &self.state
    }
    fn face_divide_base(&self) -> &ShapeUpgradeTool {
        &self.parent
    }
}
impl std::ops::Deref for ShapeUpgradeFaceDivide {
    type Target = ShapeUpgradeTool;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl ShapeUpgradeFaceDivide {
    /// Create a divider without a face; `ShapeUpgradeFaceDivide::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Construct with the face and result initialized to `face`; `ShapeUpgradeFaceDivide::new(face)`.
    pub fn new(face: Face) -> Self {
        let tool = Self::default();
        tool.init(&face);
        tool
    }
    /// Replace working face and result without clearing statuses; `divider.init(&face)`.
    pub fn init(&self, face: &Face) {
        self.state.with_mut(|fields| {
            fields.face = face.clone();
            fields.result = face.clone().into_shape();
        });
    }
    /// Dispatch the configured subdivision policies; `divider.perform(0.)?`.
    pub fn perform(&self, area: f64) -> Result<bool, Failure> {
        FaceDivider::perform(self, area)
    }
    /// Return the current intermediate/result shape; `divider.result()`.
    pub fn result(&self) -> Shape {
        self.state.with(|fields| fields.result.clone())
    }
    /// Query native subdivision flags; `divider.status_flag(ShapeExtendStatus::Done)`.
    pub fn status_flag(&self, status: ShapeExtendStatus) -> bool {
        self.state
            .with(|fields| ShapeExtend::decode_status(fields.status, status))
    }
    /// Select geometric segmentation; `divider.set_surface_segment_mode(false)`.
    pub fn set_surface_segment_mode(&self, enabled: bool) {
        self.state.with_mut(|fields| fields.segment = enabled);
    }
    /// Replace the surface criterion, allowing a null tool; `divider.set_split_surface_tool(&tool)`.
    pub fn set_split_surface_tool(&self, tool: &Handle<ShapeUpgradeSplitSurface>) {
        self.state
            .with_mut(|fields| fields.surface_tool = tool.clone());
    }
    /// Replace the wire criterion, allowing a null tool; `divider.set_wire_divide_tool(&tool)`.
    pub fn set_wire_divide_tool(&self, tool: &Handle<ShapeUpgradeWireDivide>) {
        self.state
            .with_mut(|fields| fields.wire_tool = tool.clone());
    }
}
#[cfg(test)]
#[path = "face_divide/legacy_tests.rs"]
mod legacy_tests;
