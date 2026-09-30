//! gp_Pnt::ChangeCoord: a writable gp_XYZ view of a point.
//!
//! The point stores a plain coordinate triple (shared crate-wide), so the view edits a gp_XYZ
//! copy and writes it back when dropped; within one statement this behaves exactly like the
//! native `gp_XYZ&`, e.g. `point.change_coord().add_assign(...)`.
use super::super::{Point3, Xyz};
use std::ops::{Deref, DerefMut};
/// Mutable coordinate algebra of a [`Point3`], committed on drop; `*point.change_coord() += xyz`.
#[derive(Debug)]
pub struct PointCoordinatesMut<'point> {
    point: &'point mut Point3,
    coordinates: Xyz,
}
impl Point3 {
    /// Borrows the coordinates as writable gp_XYZ algebra (gp_Pnt::ChangeCoord);
    /// `point.change_coord().set_x(1.0)`.
    pub fn change_coord(&mut self) -> PointCoordinatesMut<'_> {
        let coordinates = self.xyz();
        PointCoordinatesMut {
            point: self,
            coordinates,
        }
    }
}
impl Deref for PointCoordinatesMut<'_> {
    type Target = Xyz;
    fn deref(&self) -> &Xyz {
        &self.coordinates
    }
}
impl DerefMut for PointCoordinatesMut<'_> {
    fn deref_mut(&mut self) -> &mut Xyz {
        &mut self.coordinates
    }
}
impl Drop for PointCoordinatesMut<'_> {
    fn drop(&mut self) {
        self.point.coordinates = *self.coordinates.coordinates();
    }
}
