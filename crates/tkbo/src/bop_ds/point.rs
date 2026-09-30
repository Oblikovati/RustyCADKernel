// BOPDS_Point.hxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! An intersection point with parameters on two faces and a vertex index.
use rustycad_tkmath::gp::{Point2, Point3};
/// `BOPDS_Point`; `Point::default().index() == -1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    point: Point3,
    uv1: Point2,
    uv2: Point2,
    index: i32,
}
impl Default for Point {
    fn default() -> Self {
        Self {
            point: Point3::new(99.0, 99.0, 99.0),
            uv1: Point2::new(99.0, 99.0),
            uv2: Point2::new(99.0, 99.0),
            index: -1,
        }
    }
}
impl Point {
    /// Replaces the spatial point; `point.set_pnt(Point3::new(1.0, 2.0, 3.0))`.
    pub fn set_pnt(&mut self, point: Point3) {
        self.point = point;
    }
    /// Borrows the spatial point; `point.pnt()`.
    pub fn pnt(&self) -> &Point3 {
        &self.point
    }
    /// Replaces the first face parameters; `point.set_pnt2d1(Point2::new(1.0, 2.0))`.
    pub fn set_pnt2d1(&mut self, point: Point2) {
        self.uv1 = point;
    }
    /// Borrows the first face parameters; `point.pnt2d1()`.
    pub fn pnt2d1(&self) -> &Point2 {
        &self.uv1
    }
    /// Replaces the second face parameters; `point.set_pnt2d2(Point2::new(3.0, 4.0))`.
    pub fn set_pnt2d2(&mut self, point: Point2) {
        self.uv2 = point;
    }
    /// Borrows the second face parameters; `point.pnt2d2()`.
    pub fn pnt2d2(&self) -> &Point2 {
        &self.uv2
    }
    /// Replaces the vertex index; `point.set_index(3)`.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }
    /// Reads the vertex index; `point.index()`.
    pub fn index(&self) -> i32 {
        self.index
    }
}
