//! `Aspect_Grid`: the abstract snapping grid of a viewer.
//!
//! The abstract class is the trait [`AspectGrid`] over the embedded [`AspectGridBase`]
//! (visualization-design.md section 1, geom-design.md section 3). The pure virtuals
//! (`Compute`, `Display`, `Erase`, `IsDisplayed`, `Init`, `UpdateDisplay`) are required
//! trait methods; the non-virtual members are the extension trait [`AspectGridMembers`].
use super::AspectGridDrawMode;
use crate::json_dump::{class_begin, field_color, field_integer, field_real};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{SharedState, Transient};
use std::io;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// The protected fields of `Aspect_Grid`, which subclasses read and write.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectGridFields {
    /// `myRotationAngle`.
    pub rotation_angle: f64,
    /// `myXOrigin`.
    pub x_origin: f64,
    /// `myYOrigin`.
    pub y_origin: f64,
    /// `myColor`.
    pub color: Color,
    /// `myTenthColor`.
    pub tenth_color: Color,
    /// `myIsActive`.
    pub is_active: bool,
    /// `myDrawMode`.
    pub draw_mode: AspectGridDrawMode,
}

/// The state every grid embeds; `AspectGridBase::default()` is the native default grid.
///
/// A concrete grid also embeds its own `StandardTransient` (the reference count), as every
/// transient of this port does.
#[derive(Debug)]
pub struct AspectGridBase {
    fields: SharedState<AspectGridFields>,
}
impl Default for AspectGridBase {
    /// Origin 0, angle 0, gray 50 and gray 70 colors, as the defaulted protected constructor.
    fn default() -> Self {
        Self::with_default_colors(0.0, 0.0, 0.0)
    }
}
impl AspectGridBase {
    /// The protected constructor with its default colors (gray 50, tenth lines gray 70), as
    /// the rectangular and circular grids call it; `AspectGridBase::with_default_colors(0.0, 0.0, 0.0)`.
    pub fn with_default_colors(x_origin: f64, y_origin: f64, angle: f64) -> Self {
        let color = Color::named(NamedColor::GRAY50);
        let tenth = Color::named(NamedColor::GRAY70);
        Self::new(x_origin, y_origin, angle, color, tenth)
    }
    /// A new inactive grid drawn with lines; `AspectGridBase::new(0.0, 0.0, 0.0, gray, gray)`.
    pub fn new(x_origin: f64, y_origin: f64, angle: f64, color: Color, tenth_color: Color) -> Self {
        let fields = AspectGridFields {
            rotation_angle: angle,
            x_origin,
            y_origin,
            color,
            tenth_color,
            is_active: false,
            draw_mode: AspectGridDrawMode::Lines,
        };
        Self {
            fields: SharedState::new(fields),
        }
    }
    /// Reads the protected fields; `base.fields().x_origin`.
    pub fn fields(&self) -> RwLockReadGuard<'_, AspectGridFields> {
        self.fields.read()
    }
    /// Writes the protected fields; `base.fields_mut().is_active = true`.
    pub fn fields_mut(&self) -> RwLockWriteGuard<'_, AspectGridFields> {
        self.fields.write()
    }
    /// `Aspect_Grid::DumpJson`: class name, angle, origins, colors, state and draw mode.
    ///
    /// Example: `grid.grid_base().dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let fields = *self.fields();
        // OCCT_DUMP_TRANSIENT_CLASS_BEGIN names the static class of the member.
        class_begin(stream, "Aspect_Grid")?;
        field_real(stream, "myRotationAngle", fields.rotation_angle)?;
        field_real(stream, "myXOrigin", fields.x_origin)?;
        field_real(stream, "myYOrigin", fields.y_origin)?;
        field_color(stream, depth, "&myColor", fields.color)?;
        field_color(stream, depth, "&myTenthColor", fields.tenth_color)?;
        field_integer(stream, "myIsActive", i32::from(fields.is_active))?;
        field_integer(stream, "myDrawMode", fields.draw_mode as i32)
    }
}

/// The abstract grid; a viewer grid (TKV3d) implements the drawing members.
pub trait AspectGrid: Transient {
    /// The embedded base; `grid.grid_base().fields().x_origin`.
    fn grid_base(&self) -> &AspectGridBase;
    /// The grid point closest to `(x, y)`; `grid.compute(0.4, 1.6)`.
    fn compute(&self, x: f64, y: f64) -> (f64, f64);
    /// Displays the grid on screen; `grid.display()`.
    fn display(&self);
    /// Erases the grid from screen; `grid.erase()`.
    fn erase(&self);
    /// True when the grid is displayed; `grid.is_displayed()`.
    fn is_displayed(&self) -> bool;
    /// Recomputes the cached grid parameters; `grid.init()`.
    fn init(&self);
    /// Updates the displayed grid after a parameter change (protected); `grid.update_display()`.
    fn update_display(&self);
    /// Changes the colors (virtual); `grid.set_colors(gray, white)`.
    fn set_colors(&self, color: Color, tenth_color: Color) {
        {
            let mut fields = self.grid_base().fields_mut();
            fields.color = color;
            fields.tenth_color = tenth_color;
        }
        self.update_display();
    }
    /// `DumpJson` (virtual); the base writes the `Aspect_Grid` fields.
    ///
    /// Example: `grid.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.grid_base().dump_json(stream, depth)
    }
}

/// The non-virtual members of `Aspect_Grid`, for every grid.
pub trait AspectGridMembers: AspectGrid {
    /// Moves the origin along X, then reinitialises and redisplays; `grid.set_x_origin(1.0)`.
    fn set_x_origin(&self, origin: f64) {
        self.grid_base().fields_mut().x_origin = origin;
        init_and_update(self);
    }
    /// Moves the origin along Y; `grid.set_y_origin(1.0)`.
    fn set_y_origin(&self, origin: f64) {
        self.grid_base().fields_mut().y_origin = origin;
        init_and_update(self);
    }
    /// Sets the rotation angle; `grid.set_rotation_angle(0.5)`.
    fn set_rotation_angle(&self, angle: f64) {
        self.grid_base().fields_mut().rotation_angle = angle;
        init_and_update(self);
    }
    /// Rotates by a relative angle; `grid.rotate(0.1)`.
    fn rotate(&self, angle: f64) {
        self.grid_base().fields_mut().rotation_angle += angle;
        init_and_update(self);
    }
    /// Translates by a relative distance; `grid.translate(1.0, 2.0)`.
    fn translate(&self, dx: f64, dy: f64) {
        {
            let mut fields = self.grid_base().fields_mut();
            fields.x_origin += dx;
            fields.y_origin += dy;
        }
        init_and_update(self);
    }
    /// The grid point closest to `(x, y)` when active, else `(x, y)`; `grid.hit(0.4, 1.6)`.
    fn hit(&self, x: f64, y: f64) -> (f64, f64) {
        if self.is_active() {
            return self.compute(x, y);
        }
        (x, y)
    }
    /// Activates snapping; `grid.activate()`.
    fn activate(&self) {
        self.grid_base().fields_mut().is_active = true;
    }
    /// Deactivates snapping; `grid.deactivate()`.
    fn deactivate(&self) {
        self.grid_base().fields_mut().is_active = false;
    }
    /// The X origin; `grid.x_origin()`.
    fn x_origin(&self) -> f64 {
        self.grid_base().fields().x_origin
    }
    /// The Y origin; `grid.y_origin()`.
    fn y_origin(&self) -> f64 {
        self.grid_base().fields().y_origin
    }
    /// The rotation angle; `grid.rotation_angle()`.
    fn rotation_angle(&self) -> f64 {
        self.grid_base().fields().rotation_angle
    }
    /// True when the grid is active; `grid.is_active()`.
    fn is_active(&self) -> bool {
        self.grid_base().fields().is_active
    }
    /// The two colors (the C++ output references); `let (color, tenth) = grid.colors();`.
    fn colors(&self) -> (Color, Color) {
        let fields = self.grid_base().fields();
        (fields.color, fields.tenth_color)
    }
    /// Changes the draw mode and redisplays; `grid.set_draw_mode(AspectGridDrawMode::Points)`.
    fn set_draw_mode(&self, mode: AspectGridDrawMode) {
        self.grid_base().fields_mut().draw_mode = mode;
        self.update_display();
    }
    /// The draw mode; `grid.draw_mode()`.
    fn draw_mode(&self) -> AspectGridDrawMode {
        self.grid_base().fields().draw_mode
    }
}
impl<T: AspectGrid + ?Sized> AspectGridMembers for T {}

/// `Init()` then `UpdateDisplay()`, as every native origin, angle and step setter ends.
pub(crate) fn init_and_update<G: AspectGrid + ?Sized>(grid: &G) {
    grid.init();
    grid.update_display();
}

rustycad_tkernel::impl_transient_interface!(
    dyn AspectGrid,
    "Aspect_Grid",
    rustycad_tkernel::standard::StandardTransient
);
