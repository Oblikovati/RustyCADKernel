//! `Draw_Marker3D` and `Draw_Marker2D`: point markers.
use crate::draw::color::{DrawColor, DrawMarkerShape};
use crate::draw::display::DrawDisplay;
use crate::draw::drawable::{Drawable3D, Drawable3DBase, drawable_identity};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use std::cell::Cell;

/// The size of a marker: pixels, or (3D only) a model-space radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkerSize {
    /// `int ISize`: a size in pixels (default 5).
    Pixels(i32),
    /// `double RSize`: a size that follows the zoom.
    Scaled(f64),
}

/// `Draw_Marker3D`.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawColor, DrawMarker3D, DrawMarkerShape, MarkerSize};
/// use rustycad_tkmath::gp::Point3;
/// let marker = DrawMarker3D::new(Point3::new(1.0, 2.0, 3.0), DrawMarkerShape::X, DrawColor::default(), MarkerSize::Pixels(5));
/// assert_eq!(marker.position().z(), 3.0);
/// ```
#[derive(Debug)]
pub struct DrawMarker3D {
    base: Drawable3DBase,
    position: Cell<Point3>,
    color: DrawColor,
    shape: DrawMarkerShape,
    size: MarkerSize,
}

impl DrawMarker3D {
    /// `Draw_Marker3D(P, T, C, ISize)` / `Draw_Marker3D(P, T, C, RSize)`.
    pub fn new(
        position: Point3,
        shape: DrawMarkerShape,
        color: DrawColor,
        size: MarkerSize,
    ) -> Self {
        Self {
            base: Drawable3DBase::new(),
            position: Cell::new(position),
            color,
            shape,
            size,
        }
    }

    /// The position (`ChangePos()` read access).
    pub fn position(&self) -> Point3 {
        self.position.get()
    }

    /// `ChangePos() = P`.
    pub fn set_position(&self, position: Point3) {
        self.position.set(position);
    }
}

impl Drawable3D for DrawMarker3D {
    drawable_identity!("Draw_Marker3D");

    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        display.set_color(self.color);
        match self.size {
            MarkerSize::Scaled(radius) => {
                display.draw_marker_3d_scaled(self.position(), self.shape, radius)
            }
            MarkerSize::Pixels(pixels) => {
                display.draw_marker_3d(self.position(), self.shape, pixels)
            }
        }
        Ok(())
    }

    /// Always false: a marker is always a pick candidate.
    fn pick_reject(&self, _x: f64, _y: f64, _prec: f64) -> bool {
        false
    }
}

/// `Draw_Marker2D`. The `RSize` constructor ignores the size and draws nothing visible
/// (size 0), as upstream.
#[derive(Debug)]
pub struct DrawMarker2D {
    base: Drawable3DBase,
    position: Cell<Point2>,
    color: DrawColor,
    shape: DrawMarkerShape,
    size: i32,
}

impl DrawMarker2D {
    /// `Draw_Marker2D(P, T, C, Size)`; `MarkerSize::Scaled` gives size 0 (upstream).
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawColor, DrawMarker2D, DrawMarkerShape, MarkerSize};
    /// use rustycad_tkmath::gp::Point2;
    /// let marker = DrawMarker2D::new(Point2::new(1.0, 2.0), DrawMarkerShape::Plus, DrawColor::default(), MarkerSize::Pixels(5));
    /// assert_eq!(marker.position().x(), 1.0);
    /// ```
    pub fn new(
        position: Point2,
        shape: DrawMarkerShape,
        color: DrawColor,
        size: MarkerSize,
    ) -> Self {
        let size = match size {
            MarkerSize::Pixels(pixels) => pixels,
            MarkerSize::Scaled(_) => 0,
        };
        Self {
            base: Drawable3DBase::new(),
            position: Cell::new(position),
            color,
            shape,
            size,
        }
    }

    /// The position (`ChangePos()` read access).
    pub fn position(&self) -> Point2 {
        self.position.get()
    }

    /// `ChangePos() = P`.
    pub fn set_position(&self, position: Point2) {
        self.position.set(position);
    }
}

impl Drawable3D for DrawMarker2D {
    drawable_identity!("Draw_Marker2D");

    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        display.set_color(self.color);
        display.draw_marker(self.position(), self.shape, self.size);
        Ok(())
    }

    fn is_3d(&self) -> bool {
        false
    }

    /// Always false.
    fn pick_reject(&self, _x: f64, _y: f64, _prec: f64) -> bool {
        false
    }
}
