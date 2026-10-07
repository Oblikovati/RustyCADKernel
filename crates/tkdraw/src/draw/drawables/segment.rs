//! `Draw_Segment3D` and `Draw_Segment2D`: colored line segments.
use crate::draw::DrawInterpretor;
use crate::draw::color::DrawColor;
use crate::draw::display::DrawDisplay;
use crate::draw::drawable::{Drawable3D, Drawable3DBase, drawable_identity};
use crate::draw::ostream::DrawOStream;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use std::cell::Cell;

/// `Draw_Segment3D`.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawColor, DrawSegment3D};
/// use rustycad_tkmath::gp::Point3;
/// let segment = DrawSegment3D::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), DrawColor::default());
/// assert_eq!(segment.last().x(), 1.0);
/// ```
#[derive(Debug)]
pub struct DrawSegment3D {
    base: Drawable3DBase,
    first: Cell<Point3>,
    last: Cell<Point3>,
    color: DrawColor,
}

impl DrawSegment3D {
    /// `Draw_Segment3D(p1, p2, col)`.
    pub fn new(first: Point3, last: Point3, color: DrawColor) -> Self {
        Self {
            base: Drawable3DBase::new(),
            first: Cell::new(first),
            last: Cell::new(last),
            color,
        }
    }

    /// `First()`.
    pub fn first(&self) -> Point3 {
        self.first.get()
    }

    /// `First(P)`.
    pub fn set_first(&self, point: Point3) {
        self.first.set(point);
    }

    /// `Last()`.
    pub fn last(&self) -> Point3 {
        self.last.get()
    }

    /// `Last(P)`.
    pub fn set_last(&self, point: Point3) {
        self.last.set(point);
    }
}

impl Drawable3D for DrawSegment3D {
    drawable_identity!("Draw_Segment3D");

    fn draw_on(&self, display: &mut DrawDisplay<'_>) {
        display.set_color(self.color);
        display.draw_3d(self.first(), self.last());
    }
}

/// `Draw_Segment2D`.
#[derive(Debug)]
pub struct DrawSegment2D {
    base: Drawable3DBase,
    first: Cell<Point2>,
    last: Cell<Point2>,
    color: DrawColor,
}

impl DrawSegment2D {
    /// `Draw_Segment2D(p1, p2, col)`.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawColor, DrawSegment2D};
    /// use rustycad_tkmath::gp::Point2;
    /// let segment = DrawSegment2D::new(Point2::new(0.0, 0.0), Point2::new(1.0, 2.0), DrawColor::default());
    /// assert_eq!(segment.last().y(), 2.0);
    /// ```
    pub fn new(first: Point2, last: Point2, color: DrawColor) -> Self {
        Self {
            base: Drawable3DBase::new(),
            first: Cell::new(first),
            last: Cell::new(last),
            color,
        }
    }

    /// `First()`.
    pub fn first(&self) -> Point2 {
        self.first.get()
    }

    /// `First(P)`.
    pub fn set_first(&self, point: Point2) {
        self.first.set(point);
    }

    /// `Last()`.
    pub fn last(&self) -> Point2 {
        self.last.get()
    }

    /// `Last(P)`.
    pub fn set_last(&self, point: Point2) {
        self.last.set(point);
    }
}

impl Drawable3D for DrawSegment2D {
    drawable_identity!("Draw_Segment2D");

    fn draw_on(&self, display: &mut DrawDisplay<'_>) {
        display.set_color(self.color);
        display.draw(self.first(), self.last());
    }

    fn is_3d(&self) -> bool {
        false
    }

    /// The end points, each coordinate right-aligned in 17 columns.
    fn dump(&self, stream: &mut DrawOStream) {
        let (first, last) = (self.first(), self.last());
        stream
            .set_width(17)
            .put_real(first.x())
            .put_str(" ")
            .set_width(17)
            .put_real(first.y());
        stream
            .put_str(" - ")
            .set_width(17)
            .put_real(last.x())
            .put_str(" ")
            .set_width(17)
            .put_real(last.y());
        stream.put_str("\n");
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("segment 2d")?;
        Ok(())
    }
}
