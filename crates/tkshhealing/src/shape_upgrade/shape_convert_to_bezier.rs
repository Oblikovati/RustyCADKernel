//! Whole-shape Bezier conversion from OCCT ShapeUpgrade_ShapeConvertToBezier.
use super::{
    ShapeDivider, ShapeUpgradeConvertCurve2dToBezier, ShapeUpgradeConvertCurve3dToBezier,
    ShapeUpgradeConvertSurfaceToBezierBasis, ShapeUpgradeFaceDivide, ShapeUpgradeFixSmallCurves,
    ShapeUpgradeShapeDivide, ShapeUpgradeWireDivide,
};
use crate::shape_extend::BasicMsgRegistrator;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::Msg,
    standard::{Failure, Handle, SharedState},
    tcollection::AsciiString,
};

mod perform;
mod postprocess;

#[derive(Clone, Copy)]
struct ConversionModes {
    curves_2d: bool,
    curves_3d: bool,
    surfaces: bool,
    lines_3d: bool,
    circles_3d: bool,
    conics_3d: bool,
    planes: bool,
    revolutions: bool,
    extrusions: bool,
    bspline_surfaces: bool,
}
impl Default for ConversionModes {
    fn default() -> Self {
        Self {
            curves_2d: false,
            curves_3d: false,
            surfaces: false,
            lines_3d: true,
            circles_3d: true,
            conics_3d: true,
            planes: true,
            revolutions: true,
            extrusions: true,
            bspline_surfaces: true,
        }
    }
}

/// Convert selected shape curves and surfaces to Bezier representations.
/// Example: `let converter = ShapeUpgradeShapeConvertToBezier::new(shape);`.
pub struct ShapeUpgradeShapeConvertToBezier {
    parent: ShapeUpgradeShapeDivide,
    modes: SharedState<ConversionModes>,
    level: SharedState<i32>,
}
impl Default for ShapeUpgradeShapeConvertToBezier {
    fn default() -> Self {
        Self::from_parent(ShapeUpgradeShapeDivide::empty())
    }
}
impl std::ops::Deref for ShapeUpgradeShapeConvertToBezier {
    type Target = ShapeUpgradeShapeDivide;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl ShapeDivider for ShapeUpgradeShapeConvertToBezier {
    fn shape_divide_base(&self) -> &ShapeUpgradeShapeDivide {
        &self.parent
    }
    fn perform(&self, new_context: bool) -> Result<bool, Failure> {
        perform::perform(self, new_context)
    }
    fn split_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        self.new_face_tool()
    }
    fn face_msg(&self) -> Msg {
        message("ShapeDivide.FaceConvertToBezier.MSG0")
    }
    fn wire_msg(&self) -> Msg {
        message("ShapeDivide.WireConvertToBezier.MSG0")
    }
    fn edge_msg(&self) -> Msg {
        message("ShapeDivide.EdgeConvertToBezier.MSG0")
    }
    fn set_msg_registrator(&self, registrator: &Handle<dyn BasicMsgRegistrator>) {
        self.parent.set_msg_registrator(registrator);
    }
}
impl ShapeUpgradeShapeConvertToBezier {
    /// Construct without an input shape; `ShapeUpgradeShapeConvertToBezier::empty()`.
    pub fn empty() -> Self {
        Self::default()
    }
    /// Construct for a shape; `ShapeUpgradeShapeConvertToBezier::new(shape)`.
    pub fn new(shape: Shape) -> Self {
        Self::from_parent(ShapeUpgradeShapeDivide::new(shape))
    }
    /// Perform enabled conversions; `converter.perform(true)?`.
    pub fn perform(&self, new_context: bool) -> Result<bool, Failure> {
        ShapeDivider::perform(self, new_context)
    }
    /// Enable or disable p-curve conversion; `converter.set_2d_conversion(true)`.
    pub fn set_2d_conversion(&self, enabled: bool) {
        self.modes.write().curves_2d = enabled;
    }
    /// Read p-curve conversion mode; `assert!(!converter.get_2d_conversion())`.
    pub fn get_2d_conversion(&self) -> bool {
        self.modes.read().curves_2d
    }
    /// Enable or disable spatial curve conversion; `converter.set_3d_conversion(true)`.
    pub fn set_3d_conversion(&self, enabled: bool) {
        self.modes.write().curves_3d = enabled;
    }
    /// Read spatial curve conversion mode; `assert!(!converter.get_3d_conversion())`.
    pub fn get_3d_conversion(&self) -> bool {
        self.modes.read().curves_3d
    }
    /// Enable or disable surface conversion; `converter.set_surface_conversion(true)`.
    pub fn set_surface_conversion(&self, enabled: bool) {
        self.modes.write().surfaces = enabled;
    }
    /// Read surface conversion mode; `assert!(!converter.get_surface_conversion())`.
    pub fn get_surface_conversion(&self) -> bool {
        self.modes.read().surfaces
    }
    /// Enable or disable line conversion; `converter.set_3d_line_conversion(false)`.
    pub fn set_3d_line_conversion(&self, enabled: bool) {
        self.modes.write().lines_3d = enabled;
    }
    /// Read line conversion mode; `converter.get_3d_line_conversion()`.
    pub fn get_3d_line_conversion(&self) -> bool {
        self.modes.read().lines_3d
    }
    /// Enable or disable circle conversion; `converter.set_3d_circle_conversion(false)`.
    pub fn set_3d_circle_conversion(&self, enabled: bool) {
        self.modes.write().circles_3d = enabled;
    }
    /// Read circle conversion mode; `converter.get_3d_circle_conversion()`.
    pub fn get_3d_circle_conversion(&self) -> bool {
        self.modes.read().circles_3d
    }
    /// Enable or disable other conic conversion; `converter.set_3d_conic_conversion(false)`.
    pub fn set_3d_conic_conversion(&self, enabled: bool) {
        self.modes.write().conics_3d = enabled;
    }
    /// Read other conic conversion mode; `converter.get_3d_conic_conversion()`.
    pub fn get_3d_conic_conversion(&self) -> bool {
        self.modes.read().conics_3d
    }
    /// Enable or disable plane conversion; `converter.set_plane_mode(false)`.
    pub fn set_plane_mode(&self, enabled: bool) {
        self.modes.write().planes = enabled;
    }
    /// Read plane conversion mode; `converter.get_plane_mode()`.
    pub fn get_plane_mode(&self) -> bool {
        self.modes.read().planes
    }
    /// Enable or disable revolution conversion; `converter.set_revolution_mode(false)`.
    pub fn set_revolution_mode(&self, enabled: bool) {
        self.modes.write().revolutions = enabled;
    }
    /// Read revolution conversion mode; `converter.get_revolution_mode()`.
    pub fn get_revolution_mode(&self) -> bool {
        self.modes.read().revolutions
    }
    /// Enable or disable extrusion conversion; `converter.set_extrusion_mode(false)`.
    pub fn set_extrusion_mode(&self, enabled: bool) {
        self.modes.write().extrusions = enabled;
    }
    /// Read extrusion conversion mode; `converter.get_extrusion_mode()`.
    pub fn get_extrusion_mode(&self) -> bool {
        self.modes.read().extrusions
    }
    /// Enable or disable B-spline surface conversion; `converter.set_bspline_mode(false)`.
    pub fn set_bspline_mode(&self, enabled: bool) {
        self.modes.write().bspline_surfaces = enabled;
    }
    /// Read B-spline surface conversion mode; `converter.get_bspline_mode()`.
    pub fn get_bspline_mode(&self) -> bool {
        self.modes.read().bspline_surfaces
    }
    fn modes(&self) -> ConversionModes {
        *self.modes.read()
    }
    fn set_modes(&self, modes: ConversionModes) {
        *self.modes.write() = modes;
    }
    fn shape_divide_level(&self) -> i32 {
        *self.level.read()
    }
    fn set_shape_divide_level(&self, level: i32) {
        *self.level.write() = level;
    }
    fn run_base_divide(&self, new_context: bool) -> Result<bool, Failure> {
        let face_tool = self.new_face_tool();
        self.parent.set_split_face_tool(&face_tool);
        self.parent.perform(new_context)
    }
    fn set_source_to_result(&self) {
        self.parent
            .with_mut(|fields| fields.shape = fields.result.clone());
    }
    fn new_face_tool(&self) -> Handle<ShapeUpgradeFaceDivide> {
        let modes = self.modes();
        let face = Handle::new(ShapeUpgradeFaceDivide::empty());
        let wire = Handle::new(ShapeUpgradeWireDivide::new());
        wire.set_edge_mode(self.parent.with(|fields| fields.edge_mode));
        wire.set_fix_small_curve_tool(&Handle::new(ShapeUpgradeFixSmallCurves::new()));
        configure_curve_tools(&wire, modes);
        face.set_wire_divide_tool(&wire);
        configure_surface_tool(&face, modes);
        face
    }
    fn from_parent(parent: ShapeUpgradeShapeDivide) -> Self {
        Self {
            parent,
            modes: SharedState::new(ConversionModes::default()),
            level: SharedState::new(0),
        }
    }
}
fn configure_curve_tools(wire: &ShapeUpgradeWireDivide, modes: ConversionModes) {
    if modes.curves_3d {
        let converter = Handle::new(ShapeUpgradeConvertCurve3dToBezier::new());
        converter.set_line_mode(modes.lines_3d);
        converter.set_circle_mode(modes.circles_3d);
        converter.set_conic_mode(modes.conics_3d);
        wire.set_split_curve_3d_tool(&converter.cast());
    }
    if modes.curves_2d {
        let converter = Handle::new(ShapeUpgradeConvertCurve2dToBezier::new());
        wire.set_split_curve_2d_tool(&converter.cast());
    }
}
fn configure_surface_tool(face: &ShapeUpgradeFaceDivide, modes: ConversionModes) {
    if !modes.surfaces {
        return;
    }
    let converter = Handle::new(ShapeUpgradeConvertSurfaceToBezierBasis::new());
    converter.set_plane_mode(modes.planes);
    converter.set_revolution_mode(modes.revolutions);
    converter.set_extrusion_mode(modes.extrusions);
    converter.set_bspline_mode(modes.bspline_surfaces);
    face.set_split_surface_tool(&converter.cast());
}
fn message(key: &str) -> Msg {
    Msg::from_key(&AsciiString::from(key))
}
