use super::NodeArray;
use crate::gp::{Point2, Point3};
use rustycad_tkernel::standard::Failure;
impl NodeArray<'_, 3> {
    /// Reads a three-dimensional node; `nodes.value(0)?`.
    pub fn value(&self, index: usize) -> Result<Point3, Failure> {
        let [x, y, z] = self.coordinates(index)?;
        Ok(Point3::new(x, y, z))
    }
    /// Stores a three-dimensional node; `nodes.set_value(0,Point3::default())?`.
    pub fn set_value(&mut self, index: usize, point: Point3) -> Result<(), Failure> {
        self.set_coordinates(index, [point.x(), point.y(), point.z()])
    }
}
impl NodeArray<'_, 2> {
    /// Reads a parametric node; `uv_nodes.value(0)?`.
    pub fn value(&self, index: usize) -> Result<Point2, Failure> {
        let [x, y] = self.coordinates(index)?;
        Ok(Point2::new(x, y))
    }
    /// Stores a parametric node; `uv_nodes.set_value(0,Point2::default())?`.
    pub fn set_value(&mut self, index: usize, point: Point2) -> Result<(), Failure> {
        self.set_coordinates(index, [point.x(), point.y()])
    }
}
