//! `Draw_Axis3D` and `Draw_Axis2D`: labelled coordinate axes of a fixed screen size.
use crate::draw::color::DrawColor;
use crate::draw::display::DrawDisplay;
use crate::draw::drawable::{Drawable3D, Drawable3DBase, drawable_identity};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis3, Axis22d, Point2, Point3, StandardDirection3, Vector2, Vector3};

/// The default axis length in pixels.
pub const DEFAULT_AXIS_SIZE: i32 = 5;

/// `Draw_Axis3D`: the X, Y and Z axes of a frame, `size` pixels long.
#[derive(Debug)]
pub struct DrawAxis3D {
    base: Drawable3DBase,
    axes: Axis3,
    color: DrawColor,
    size: i32,
}

impl DrawAxis3D {
    /// `Draw_Axis3D(col, Size)`: the axes of `gp::XOY()`.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawAxis3D, DrawColor};
    /// let axes = DrawAxis3D::standard(DrawColor::default(), 5);
    /// assert_eq!(axes.axes().location().x(), 0.0);
    /// ```
    pub fn standard(color: DrawColor, size: i32) -> Self {
        Self::at_point(Point3::new(0.0, 0.0, 0.0), color, size)
    }

    /// `Draw_Axis3D(p, col, Size)`: the standard directions at `p` (`gp_Ax3(p, DZ, DX)`).
    pub fn at_point(point: Point3, color: DrawColor, size: i32) -> Self {
        let axes = Axis3::with_standard_frame(point, StandardDirection3::Z, StandardDirection3::X);
        Self::new(axes, color, size)
    }

    /// `Draw_Axis3D(A, col, Size)`.
    pub fn new(axes: Axis3, color: DrawColor, size: i32) -> Self {
        Self {
            base: Drawable3DBase::new(),
            axes,
            color,
            size,
        }
    }

    /// The frame drawn.
    pub fn axes(&self) -> Axis3 {
        self.axes
    }
}

impl Drawable3D for DrawAxis3D {
    drawable_identity!("Draw_Axis3D");

    /// Draws and labels each axis, without growing the drawing bounds.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        display.set_draw_bounds(false);
        display.set_color(self.color);
        let length = self.size as f64 / display.zoom();
        let origin = self.axes.location();
        let directions = [
            (self.axes.x_direction(), "X"),
            (self.axes.y_direction(), "Y"),
            (self.axes.direction(), "Z"),
        ];
        for (direction, label) in directions {
            let tip = origin.translated(Vector3::from(direction) * length);
            display.draw_3d(origin, tip);
            display.draw_string_3d(tip, label);
        }
        display.set_draw_bounds(true);
        Ok(())
    }
}

/// `Draw_Axis2D`: the X and Y axes of a 2D frame, `size` pixels long.
#[derive(Debug)]
pub struct DrawAxis2D {
    base: Drawable3DBase,
    axes: Axis22d,
    color: DrawColor,
    size: i32,
}

impl DrawAxis2D {
    /// `Draw_Axis2D(col, Size)`: the axes at the origin.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawAxis2D, DrawColor};
    /// let axes = DrawAxis2D::standard(DrawColor::default(), 5);
    /// assert_eq!(axes.axes().location().y(), 0.0);
    /// ```
    pub fn standard(color: DrawColor, size: i32) -> Self {
        Self::at_point(Point2::new(0.0, 0.0), color, size)
    }

    /// `Draw_Axis2D(p, col, Size)`: `gp_Ax22d(p, gp_Dir2d(X))`, direct sense.
    pub fn at_point(point: Point2, color: DrawColor, size: i32) -> Self {
        // The default frame is (origin, X, Y), what `gp_Ax22d(P, X)` builds at P.
        let mut axes = Axis22d::default();
        axes.set_location(point);
        Self::new(axes, color, size)
    }

    /// `Draw_Axis2D(A, col, Size)`.
    pub fn new(axes: Axis22d, color: DrawColor, size: i32) -> Self {
        Self {
            base: Drawable3DBase::new(),
            axes,
            color,
            size,
        }
    }

    /// The frame drawn.
    pub fn axes(&self) -> Axis22d {
        self.axes
    }
}

impl Drawable3D for DrawAxis2D {
    drawable_identity!("Draw_Axis2D");

    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        display.set_draw_bounds(false);
        display.set_color(self.color);
        let length = self.size as f64 / display.zoom();
        let origin = self.axes.location();
        for (direction, label) in [
            (self.axes.x_direction(), "X"),
            (self.axes.y_direction(), "Y"),
        ] {
            let tip = origin.translated(Vector2::from(direction) * length);
            display.draw(origin, tip);
            display.draw_string(tip, label);
        }
        display.set_draw_bounds(true);
        Ok(())
    }

    fn is_3d(&self) -> bool {
        false
    }
}
