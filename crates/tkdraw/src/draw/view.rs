//! `Draw_View`: a window of the viewer with its projection (type, matrix, zoom, pan,
//! focal distance). The C++ class derives from `Draw_Window`; here it owns one.
use super::color::DrawColorKind;
use super::display::{DisplaySurface, DrawXSegment};
use super::window::DrawWindow;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};
use std::f64::consts::PI;

/// A view (`Draw_View`).
pub struct DrawView {
    window: Box<dyn DrawWindow>,
    view_type: String,
    is_perspective: bool,
    is_2d: bool,
    focal_distance: f64,
    zoom: f64,
    matrix: Transform3,
    dx: i32,
    dy: i32,
    frame: (i32, i32, i32, i32),
}

/// A rotation about an axis through the origin, by `angle` radians.
fn rotation(axis: Direction3, angle: f64) -> Transform3 {
    let mut transform = Transform3::default();
    // A unit direction through the origin is always a valid rotation axis.
    let _ = transform.set_rotation(Axis1::new(Point3::new(0.0, 0.0, 0.0), axis), angle);
    transform
}

/// The axis directions of the view-type table.
fn axis_x() -> Direction3 {
    rustycad_tkmath::gp::dx()
}
fn axis_y() -> Direction3 {
    rustycad_tkmath::gp::dy()
}
fn axis_z() -> Direction3 {
    rustycad_tkmath::gp::dz()
}

/// The view matrix of a 3D view type as `Init` builds it: one rotation, or a rotation
/// multiplied by a second one; `None` for an unknown type. Also tells perspective.
fn type_matrix(view_type: &str) -> Option<Transform3> {
    let one = |axis: Direction3, angle: f64| Some(rotation(axis, angle));
    let two = |a1: Direction3, t1: f64, a2: Direction3, t2: f64| {
        Some(rotation(a1, t1).multiplied(rotation(a2, t2)))
    };
    match view_type {
        "+X+Y" => Some(Transform3::default()),
        "-Y+X" => one(axis_z(), 0.5 * PI),
        "-X-Y" => one(axis_z(), PI),
        "+Y-X" => one(axis_z(), -0.5 * PI),
        "+Y+X" => two(axis_z(), -0.5 * PI, axis_y(), PI),
        "-X+Y" => one(axis_y(), PI),
        "-Y-X" => two(axis_z(), -0.5 * PI, axis_x(), PI),
        "+X-Y" => one(axis_x(), PI),
        "+X+Z" => one(axis_x(), -0.5 * PI),
        "-Z+X" => two(axis_x(), -0.5 * PI, axis_y(), -0.5 * PI),
        "-X-Z" => two(axis_x(), -0.5 * PI, axis_y(), -PI),
        "+Z-X" => two(axis_x(), -0.5 * PI, axis_y(), 0.5 * PI),
        "+Z+X" => two(axis_x(), 0.5 * PI, axis_y(), 0.5 * PI),
        "-X+Z" => two(axis_x(), 0.5 * PI, axis_y(), PI),
        "-Z-X" => two(axis_x(), 0.5 * PI, axis_y(), -0.5 * PI),
        "+X-Z" => one(axis_x(), 0.5 * PI),
        "+Y+Z" => two(axis_y(), -0.5 * PI, axis_x(), -0.5 * PI),
        "-Z+Y" => one(axis_y(), -0.5 * PI),
        "-Y-Z" => two(axis_y(), -0.5 * PI, axis_x(), 0.5 * PI),
        "+Z-Y" => two(axis_y(), -0.5 * PI, axis_x(), PI),
        "+Z+Y" => one(axis_y(), 0.5 * PI),
        "-Y+Z" => two(axis_y(), 0.5 * PI, axis_x(), -0.5 * PI),
        "-Z-Y" => two(axis_y(), 0.5 * PI, axis_x(), PI),
        "+Y-Z" => two(axis_y(), 0.5 * PI, axis_x(), 0.5 * PI),
        "AXON" | "PERS" => two(axis_x(), -0.25 * PI, axis_z(), -0.25 * PI),
        _ => None,
    }
}

impl DrawView {
    /// `Draw_View(id, viewer, X, Y, W, H, window)` / `Draw_View(id, viewer, title)`: a
    /// view on `window`, not initialized (zoom 0, no type).
    pub fn new(window: Box<dyn DrawWindow>) -> Self {
        Self {
            window,
            view_type: String::new(),
            is_perspective: false,
            is_2d: false,
            focal_distance: 0.0,
            zoom: 0.0,
            matrix: Transform3::default(),
            dx: 0,
            dy: 0,
            frame: (0, 0, 0, 0),
        }
    }

    /// `Init`: sets the defaults (focal 500, zoom 1) and the matrix of `view_type`
    /// (`-2D-`, `+X+Y`, ..., `AXON`, `PERS`); false for an unknown type. The defaults
    /// are applied even then, as upstream.
    pub fn init(&mut self, view_type: &str) -> bool {
        self.focal_distance = 500.0;
        self.is_perspective = false;
        self.is_2d = view_type == "-2D-";
        self.zoom = 1.0;
        self.reset_frame();
        let matrix = if self.is_2d {
            Some(Transform3::default())
        } else {
            type_matrix(view_type)
        };
        let Some(matrix) = matrix else {
            return false;
        };
        self.matrix = matrix;
        self.is_perspective = view_type == "PERS";
        // `myType` is `char[5]`: the type name is at most four characters.
        self.view_type = view_type.chars().take(4).collect();
        true
    }

    /// `Type`.
    pub fn view_type(&self) -> &str {
        &self.view_type
    }

    /// `Is2D`.
    pub fn is_2d(&self) -> bool {
        self.is_2d
    }

    /// `SetZoom`.
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom;
    }

    /// `SetMatrix`.
    pub fn set_matrix(&mut self, matrix: Transform3) {
        self.matrix = matrix;
    }

    /// `SetFocalDistance`.
    pub fn set_focal_distance(&mut self, distance: f64) {
        self.focal_distance = distance;
    }

    /// `SetDx`.
    pub fn set_dx(&mut self, dx: i32) {
        self.dx = dx;
    }

    /// `SetDy`.
    pub fn set_dy(&mut self, dy: i32) {
        self.dy = dy;
    }

    /// `Transform`: right-multiplies the view matrix.
    pub fn transform(&mut self, transformation: Transform3) {
        self.matrix.multiply(transformation);
    }

    /// `ResetFrame`: forgets the cached frame.
    pub fn reset_frame(&mut self) {
        self.frame = (0, 0, 0, 0);
    }

    /// The window of the view.
    pub fn window(&self) -> &dyn DrawWindow {
        self.window.as_ref()
    }

    /// The window of the view, for modification.
    pub fn window_mut(&mut self) -> &mut dyn DrawWindow {
        self.window.as_mut()
    }

    /// `Draw_Viewer::GetFrame`: the visible frame from the window size and the pan.
    pub fn compute_frame(&self) -> (i32, i32, i32, i32) {
        let (width, height) = (self.window.width(), self.window.height());
        (-self.dx, -self.dy - height, width - self.dx, -self.dy)
    }
}

impl DisplaySurface for DrawView {
    fn zoom(&self) -> f64 {
        self.zoom
    }

    fn matrix(&self) -> Transform3 {
        self.matrix
    }

    fn is_perspective(&self) -> bool {
        self.is_perspective
    }

    fn focal_distance(&self) -> f64 {
        self.focal_distance
    }

    fn dx(&self) -> i32 {
        self.dx
    }

    fn dy(&self) -> i32 {
        self.dy
    }

    /// `GetFrame`: the cached frame, computed by the viewer when the cache is empty
    /// (an empty cache is detected by `x0 == x1`).
    fn frame(&mut self) -> (i32, i32, i32, i32) {
        if self.frame.0 == self.frame.2 {
            let (x0, y0, x1, y1) = self.compute_frame();
            self.frame = (x0, y0, x1, y1);
        }
        self.frame
    }

    fn viewer_frame(&self) -> (i32, i32, i32, i32) {
        self.compute_frame()
    }

    fn view_type(&self) -> String {
        self.view_type.clone()
    }

    fn set_color(&mut self, color: DrawColorKind) {
        self.window.set_color(color.index() as i32);
    }

    fn set_mode(&mut self, mode: i32) {
        self.window.set_mode(mode);
    }

    fn draw_segments(&mut self, segments: &[DrawXSegment]) {
        self.window.draw_segments(segments);
    }

    fn draw_string(&mut self, x: i32, y: i32, text: &str) {
        self.window.draw_string(x, y, text);
    }
}
