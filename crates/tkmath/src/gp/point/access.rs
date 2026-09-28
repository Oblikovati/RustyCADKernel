// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

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
