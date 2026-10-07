//! `Draw_Text3D` and `Draw_Text2D`: labels at a point.
use crate::draw::color::DrawColor;
use crate::draw::display::DrawDisplay;
use crate::draw::drawable::{Drawable3D, Drawable3DBase, drawable_identity};
use crate::draw::interpretor::c_text;
use rustycad_tkmath::gp::{Point2, Point3};
use std::cell::Cell;

/// `Draw_Text3D`: a text at a 3D point, shifted by `(move_x, move_y)` pixels.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawColor, DrawText3D};
/// use rustycad_tkmath::gp::Point3;
/// let text = DrawText3D::new(Point3::new(0.0, 0.0, 0.0), "P1", DrawColor::default(), 0.0, 0.0);
/// assert_eq!(text.text(), "P1");
/// ```
#[derive(Debug)]
pub struct DrawText3D {
    base: Drawable3DBase,
    point: Cell<Point3>,
    color: DrawColor,
    text: String,
    move_x: f64,
    move_y: f64,
}

impl DrawText3D {
    /// `Draw_Text3D(p, T, col[, moveX, moveY])` (the short form shifts by 0).
    pub fn new(point: Point3, text: &str, color: DrawColor, move_x: f64, move_y: f64) -> Self {
        let text = c_text(text).to_string();
        Self {
            base: Drawable3DBase::new(),
            point: Cell::new(point),
            color,
            text,
            move_x,
            move_y,
        }
    }

    /// The text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// `SetPnt`.
    pub fn set_point(&self, point: Point3) {
        self.point.set(point);
    }
}

impl Drawable3D for DrawText3D {
    drawable_identity!("Draw_Text3D");

    fn draw_on(&self, display: &mut DrawDisplay<'_>) {
        display.set_color(self.color);
        display.draw_string_moved_3d(self.point.get(), &self.text, self.move_x, self.move_y);
    }
}

/// `Draw_Text2D`: a text at a 2D point, shifted by whole pixels.
#[derive(Debug)]
pub struct DrawText2D {
    base: Drawable3DBase,
    point: Cell<Point2>,
    color: DrawColor,
    text: String,
    move_x: i32,
    move_y: i32,
}

impl DrawText2D {
    /// `Draw_Text2D(p, T, col[, moveX, moveY])`.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawColor, DrawText2D};
    /// use rustycad_tkmath::gp::Point2;
    /// let text = DrawText2D::new(Point2::new(0.0, 0.0), "uv", DrawColor::default(), 2, 3);
    /// assert_eq!(text.text(), "uv");
    /// ```
    pub fn new(point: Point2, text: &str, color: DrawColor, move_x: i32, move_y: i32) -> Self {
        let text = c_text(text).to_string();
        Self {
            base: Drawable3DBase::new(),
            point: Cell::new(point),
            color,
            text,
            move_x,
            move_y,
        }
    }

    /// The text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// `SetPnt2d`.
    pub fn set_point(&self, point: Point2) {
        self.point.set(point);
    }
}

impl Drawable3D for DrawText2D {
    drawable_identity!("Draw_Text2D");

    fn draw_on(&self, display: &mut DrawDisplay<'_>) {
        display.set_color(self.color);
        display.draw_string_moved(
            self.point.get(),
            &self.text,
            self.move_x as f64,
            self.move_y as f64,
        );
    }

    fn is_3d(&self) -> bool {
        false
    }
}
