// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Point.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Point`: a drawable 3D or 2D point.
use super::params::current;
use crate::draw::{
    DrawColor, DrawDisplay, DrawIStream, DrawInterpretor, DrawMarkerShape, DrawOStream,
    DrawSession, Drawable3D, Drawable3DBase, DrawableHandle,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use std::cell::Cell;
use std::rc::Rc;

/// The `Size` default of `Draw_Display::DrawMarker`, in pixels.
const MARKER_SIZE: i32 = 5;

/// `DrawTrSurf_Point`: a marker at a point; a 2D point is stored as `(x, y, 0)`.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawColor, DrawColorKind, DrawMarkerShape, Drawable3D};
/// use rustycad_tkdraw::draw_tr_surf::DrawTrSurfPoint;
/// use rustycad_tkmath::gp::Point2;
/// let red = DrawColor::new(DrawColorKind::Rouge);
/// let point = DrawTrSurfPoint::new_2d(Point2::new(1.0, 2.0), DrawMarkerShape::Plus, red);
/// assert!(!point.is_3d());
/// assert_eq!(point.point_2d(), Point2::new(1.0, 2.0));
/// ```
pub struct DrawTrSurfPoint {
    base: Drawable3DBase,
    point: Cell<Point3>,
    is_3d: Cell<bool>,
    shape: Cell<DrawMarkerShape>,
    color: Cell<DrawColor>,
}

impl DrawTrSurfPoint {
    /// `DrawTrSurf_Point(gp_Pnt, Shape, Col)`.
    pub fn new(point: Point3, shape: DrawMarkerShape, color: DrawColor) -> Self {
        Self {
            base: Drawable3DBase::new(),
            point: Cell::new(point),
            is_3d: Cell::new(true),
            shape: Cell::new(shape),
            color: Cell::new(color),
        }
    }

    /// `DrawTrSurf_Point(gp_Pnt2d, Shape, Col)`.
    pub fn new_2d(point: Point2, shape: DrawMarkerShape, color: DrawColor) -> Self {
        let drawable = Self::new(Point3::new(point.x(), point.y(), 0.), shape, color);
        drawable.is_3d.set(false);
        drawable
    }

    /// `Point()`.
    pub fn point(&self) -> Point3 {
        self.point.get()
    }

    /// `Point(gp_Pnt)`: the point becomes 3D.
    pub fn set_point(&self, point: Point3) {
        self.point.set(point);
        self.is_3d.set(true);
    }

    /// `Point2d()`.
    pub fn point_2d(&self) -> Point2 {
        let point = self.point.get();
        Point2::new(point.x(), point.y())
    }

    /// `Point2d(gp_Pnt2d)`: the point becomes 2D.
    pub fn set_point_2d(&self, point: Point2) {
        self.point.set(Point3::new(point.x(), point.y(), 0.));
        self.is_3d.set(false);
    }

    /// `Color()`.
    pub fn color(&self) -> DrawColor {
        self.color.get()
    }

    /// `Color(col)`.
    pub fn set_color(&self, color: DrawColor) {
        self.color.set(color);
    }

    /// `Shape()`.
    pub fn shape(&self) -> DrawMarkerShape {
        self.shape.get()
    }

    /// `Shape(shape)`.
    pub fn set_shape(&self, shape: DrawMarkerShape) {
        self.shape.set(shape);
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_Point", Self::restore);
    }

    /// `Restore`: `1 x y z` or `0 x y`, with the marker and color of
    /// `DrawTrSurf::Parameters()`; a missing number reads as 0 (`operator>>` since
    /// C++11).
    pub fn restore(session: &DrawSession, stream: &mut DrawIStream) -> Option<DrawableHandle> {
        let params = current(session);
        let is_3d = stream.read_through(|io| {
            let mut value = 0;
            io.extract_int(&mut value);
            value
        });
        let mut next = || stream.read_real().unwrap_or(0.0);
        let point: DrawableHandle = if is_3d != 0 {
            let (x, y, z) = (next(), next(), next());
            Rc::new(Self::new(
                Point3::new(x, y, z),
                params.pnt_marker,
                params.pnt_color,
            ))
        } else {
            let (x, y) = (next(), next());
            Rc::new(Self::new_2d(
                Point2::new(x, y),
                params.pnt_marker,
                params.pnt_color,
            ))
        };
        Some(point)
    }
}

impl Drawable3D for DrawTrSurfPoint {
    fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_Point"
    }

    /// `DrawOn`: the marker, 5 pixels.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        display.set_color(self.color());
        if self.is_3d.get() {
            display.draw_marker_3d(self.point(), self.shape(), MARKER_SIZE);
        } else {
            display.draw_marker(self.point_2d(), self.shape(), MARKER_SIZE);
        }
        Ok(())
    }

    fn is_3d(&self) -> bool {
        self.is_3d.get()
    }

    /// `Copy`: a new point of the same dimension, marker and color.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let (shape, color) = (self.shape(), self.color());
        if self.is_3d.get() {
            return Ok(Rc::new(Self::new(self.point(), shape, color)));
        }
        Ok(Rc::new(Self::new_2d(self.point_2d(), shape, color)))
    }

    /// `Dump`: `Point : x, y, z` or `Point 2d : x, y` in scientific notation with 15
    /// digits. Upstream quirk, kept: the stream stays scientific with precision 15.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let point = self.point();
        stream.set_scientific(true).set_precision(15);
        if self.is_3d.get() {
            stream.put_str("Point : ").put_real(point.x()).put_str(", ");
            stream.put_real(point.y()).put_str(", ").put_real(point.z());
        } else {
            stream
                .put_str("Point 2d : ")
                .put_real(point.x())
                .put_str(", ");
            stream.put_real(point.y());
        }
        stream.put_str("\n");
        Ok(())
    }

    /// `Save`: `1 x y z` or `0 x y` in scientific notation with 15 digits (the stream
    /// stays so, as upstream).
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let point = self.point();
        stream.set_scientific(true).set_precision(15);
        if self.is_3d.get() {
            stream.put_str("1 ").put_real(point.x()).put_str(" ");
            stream.put_real(point.y()).put_str(" ").put_real(point.z());
        } else {
            stream
                .put_str("0 ")
                .put_real(point.x())
                .put_str(" ")
                .put_real(point.y());
        }
        stream.put_str("\n");
        Ok(())
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("point")?;
        Ok(())
    }
}
