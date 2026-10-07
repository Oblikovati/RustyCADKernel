//! `Aspect_GridParams`: the appearance of the shader grid of `V3d_View::GridDisplay`.
use super::AspectGridDrawMode;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::negative_value;
use rustycad_tkmath::gp::Point3;

/// Grid color, scale, bounds, arc, draw mode, background and adaptive flags.
///
/// Example: `let mut params = AspectGridParams::default(); params.set_scale(0.5)?;`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectGridParams {
    color: Color,
    accent_color: Color,
    origin: Point3,
    scale: f64,
    scale_y: f64,
    accent_scale_x: f64,
    accent_scale_y: f64,
    accent_angular_scale: f64,
    line_thickness: f64,
    rotation_angle: f64,
    size_x: f64,
    size_y: f64,
    radius: f64,
    z_offset: f64,
    angle_start: f64,
    angle_end: f64,
    angular_divisions: i32,
    draw_mode: AspectGridDrawMode,
    is_background: bool,
    is_draw_axis: bool,
    is_view_adaptive: bool,
}

impl Default for AspectGridParams {
    /// Gray lines at the plane origin, axis coloring on, 1/100 spacing, overlay mode,
    /// unbounded, as `Aspect_GridParams()`.
    fn default() -> Self {
        Self {
            color: Color::named(NamedColor::GRAY70),
            accent_color: Color::named(NamedColor::GRAY50),
            origin: Point3::new(0.0, 0.0, 0.0),
            scale: 0.01,
            scale_y: 0.0,
            accent_scale_x: 0.0,
            accent_scale_y: 0.0,
            accent_angular_scale: 0.0,
            line_thickness: 0.01,
            rotation_angle: 0.0,
            size_x: 0.0,
            size_y: 0.0,
            radius: 0.0,
            z_offset: 0.0,
            angle_start: 0.0,
            angle_end: 0.0,
            angular_divisions: 0,
            draw_mode: AspectGridDrawMode::Lines,
            is_background: false,
            is_draw_axis: true,
            is_view_adaptive: false,
        }
    }
}

/// `Standard_NegativeValue_Raise_if(value < 0.0, ...)` with the offending value.
fn non_negative(value: f64, what: &str) -> Result<f64, Failure> {
    if value < 0.0 {
        return Err(negative_value(&format!(
            "invalid grid {what} {value}; expected >= 0"
        )));
    }
    Ok(value)
}

impl AspectGridParams {
    /// The grid line color; `params.color()`.
    pub fn color(&self) -> Color {
        self.color
    }
    /// Sets the grid line color; `params.set_color(red)`.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    /// The offset of the grid origin within the plane; `params.origin()`.
    pub fn origin(&self) -> Point3 {
        self.origin
    }
    /// Sets the origin offset; `params.set_origin(Point3::new(1.0, 2.0, 0.0))`.
    pub fn set_origin(&mut self, origin: Point3) {
        self.origin = origin;
    }
    /// The accent (every tenth line) color; `params.accent_color()`.
    pub fn accent_color(&self) -> Color {
        self.accent_color
    }
    /// Sets the accent color; `params.set_accent_color(blue)`.
    pub fn set_accent_color(&mut self, color: Color) {
        self.accent_color = color;
    }
    /// The accent scale along X or the radial direction (0 disables it); `params.accent_scale_x()`.
    pub fn accent_scale_x(&self) -> f64 {
        self.accent_scale_x
    }
    /// Sets the accent scale along X (not validated natively); `params.set_accent_scale_x(10.0)`.
    pub fn set_accent_scale_x(&mut self, scale: f64) {
        self.accent_scale_x = scale;
    }
    /// The accent scale along Y (0 disables it); `params.accent_scale_y()`.
    pub fn accent_scale_y(&self) -> f64 {
        self.accent_scale_y
    }
    /// Sets the accent scale along Y; `params.set_accent_scale_y(10.0)`.
    pub fn set_accent_scale_y(&mut self, scale: f64) {
        self.accent_scale_y = scale;
    }
    /// The angular accent scale of circular-grid spokes; `params.accent_angular_scale()`.
    pub fn accent_angular_scale(&self) -> f64 {
        self.accent_angular_scale
    }
    /// Sets the angular accent scale; `params.set_accent_angular_scale(2.0)`.
    pub fn set_accent_angular_scale(&mut self, scale: f64) {
        self.accent_angular_scale = scale;
    }
    /// The grid scale along X (cells per plane unit); `params.scale()`.
    pub fn scale(&self) -> f64 {
        self.scale
    }
    /// Sets the X scale; `Standard_NegativeValue` when negative; `params.set_scale(0.5)?`.
    pub fn set_scale(&mut self, scale: f64) -> Result<(), Failure> {
        self.scale = non_negative(scale, "scale")?;
        Ok(())
    }
    /// The explicit Y scale (0 means isotropic); `params.scale_y()`.
    pub fn scale_y(&self) -> f64 {
        self.scale_y
    }
    /// Sets the Y scale; `Standard_NegativeValue` when negative; `params.set_scale_y(0.2)?`.
    pub fn set_scale_y(&mut self, scale: f64) -> Result<(), Failure> {
        self.scale_y = non_negative(scale, "Y-scale")?;
        Ok(())
    }
    /// The Y scale the renderer uses: the explicit one when positive, else the X scale.
    ///
    /// Example: `AspectGridParams::default().effective_scale_y() == 0.01`.
    pub fn effective_scale_y(&self) -> f64 {
        if self.scale_y > 0.0 {
            self.scale_y
        } else {
            self.scale
        }
    }
    /// The line thickness in plane units; `params.line_thickness()`.
    pub fn line_thickness(&self) -> f64 {
        self.line_thickness
    }
    /// Sets the line thickness; `Standard_NegativeValue` when negative.
    ///
    /// Example: `params.set_line_thickness(0.02)?`.
    pub fn set_line_thickness(&mut self, thickness: f64) -> Result<(), Failure> {
        self.line_thickness = non_negative(thickness, "line thickness")?;
        Ok(())
    }
    /// The in-plane rotation angle in radians; `params.rotation_angle()`.
    pub fn rotation_angle(&self) -> f64 {
        self.rotation_angle
    }
    /// Sets the in-plane rotation angle; `params.set_rotation_angle(0.5)`.
    pub fn set_rotation_angle(&mut self, angle: f64) {
        self.rotation_angle = angle;
    }
    /// The angular divisions of the half circle (0 means rectangular); `params.angular_divisions()`.
    pub fn angular_divisions(&self) -> i32 {
        self.angular_divisions
    }
    /// Sets the angular divisions (not validated natively); `params.set_angular_divisions(8)`.
    pub fn set_angular_divisions(&mut self, divisions: i32) {
        self.angular_divisions = divisions;
    }
    /// True for a circular (polar) grid; `params.is_circular()`.
    pub fn is_circular(&self) -> bool {
        self.angular_divisions > 0
    }
    /// The bounded extent along X (0 means unbounded); `params.size_x()`.
    pub fn size_x(&self) -> f64 {
        self.size_x
    }
    /// Sets the X extent; `Standard_NegativeValue` when negative; `params.set_size_x(8.0)?`.
    pub fn set_size_x(&mut self, size: f64) -> Result<(), Failure> {
        self.size_x = non_negative(size, "X-size")?;
        Ok(())
    }
    /// The bounded extent along Y (0 means unbounded); `params.size_y()`.
    pub fn size_y(&self) -> f64 {
        self.size_y
    }
    /// Sets the Y extent; `Standard_NegativeValue` when negative; `params.set_size_y(4.0)?`.
    pub fn set_size_y(&mut self, size: f64) -> Result<(), Failure> {
        self.size_y = non_negative(size, "Y-size")?;
        Ok(())
    }
    /// The bounded radius (0 means unbounded); `params.radius()`.
    pub fn radius(&self) -> f64 {
        self.radius
    }
    /// Sets the radius; `Standard_NegativeValue` when negative; `params.set_radius(2.5)?`.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), Failure> {
        self.radius = non_negative(radius, "radius")?;
        Ok(())
    }
    /// The signed offset along the plane normal; `params.z_offset()`.
    pub fn z_offset(&self) -> f64 {
        self.z_offset
    }
    /// Sets the normal offset; `params.set_z_offset(-0.005)`.
    pub fn set_z_offset(&mut self, offset: f64) {
        self.z_offset = offset;
    }
    /// The arc start angle; `params.angle_start()`.
    pub fn angle_start(&self) -> f64 {
        self.angle_start
    }
    /// The arc end angle; `params.angle_end()`.
    pub fn angle_end(&self) -> f64 {
        self.angle_end
    }
    /// Restricts a circular grid to `[start, end]`; equal angles mean the full circle.
    ///
    /// Example: `params.set_arc_range(0.0, std::f64::consts::PI)`.
    pub fn set_arc_range(&mut self, start: f64, end: f64) {
        self.angle_start = start;
        self.angle_end = end;
    }
    /// True for a bounded rectangle or disc; `params.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        self.size_x > 0.0 || self.size_y > 0.0 || self.radius > 0.0
    }
    /// True when the circular grid is restricted to an arc; `params.is_arc()`.
    pub fn is_arc(&self) -> bool {
        self.angle_start != self.angle_end
    }
    /// The draw mode; `params.draw_mode()`.
    pub fn draw_mode(&self) -> AspectGridDrawMode {
        self.draw_mode
    }
    /// Sets the draw mode; `params.set_draw_mode(AspectGridDrawMode::Points)`.
    pub fn set_draw_mode(&mut self, mode: AspectGridDrawMode) {
        self.draw_mode = mode;
    }
    /// True when the grid is drawn as a view background; `params.is_background()`.
    pub fn is_background(&self) -> bool {
        self.is_background
    }
    /// Sets background rendering; `params.set_is_background(true)`.
    pub fn set_is_background(&mut self, is_background: bool) {
        self.is_background = is_background;
    }
    /// True when the axis lines are colored; `params.is_draw_axis()`.
    pub fn is_draw_axis(&self) -> bool {
        self.is_draw_axis
    }
    /// Sets axis coloring; `params.set_is_draw_axis(false)`.
    pub fn set_is_draw_axis(&mut self, is_draw_axis: bool) {
        self.is_draw_axis = is_draw_axis;
    }
    /// True when the spacing adapts to the camera; `params.is_view_adaptive()`.
    pub fn is_view_adaptive(&self) -> bool {
        self.is_view_adaptive
    }
    /// Sets view-adaptive spacing; `params.set_is_view_adaptive(true)`.
    pub fn set_is_view_adaptive(&mut self, is_view_adaptive: bool) {
        self.is_view_adaptive = is_view_adaptive;
    }
}
