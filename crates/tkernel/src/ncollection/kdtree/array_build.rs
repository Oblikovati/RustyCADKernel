// Copyright (c) 2026 OPEN CASCADE SAS
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

//! The `NCollection_Array1` build overloads and the native move operations.
use super::{KDTree, KdPoint};
use crate::ncollection::Array1;
use crate::standard::Failure;

impl<P: KdPoint, const D: usize> KDTree<P, D, false> {
    /// Builds from an array of any lower bound; original indices stay one-based in array
    /// order. Example: `tree.build_array(&points)`.
    pub fn build_array(&mut self, points: &Array1<'_, P>) {
        self.build(points.as_slice());
    }
}
impl<P: KdPoint, const D: usize> KDTree<P, D, true> {
    /// Builds from point and radius arrays; extra radii are ignored. A radius array shorter
    /// than the point array makes the native checked `Value` throw Standard_OutOfRange
    /// part-way through, leaving an unusable tree; here the error is returned and the tree
    /// is left empty. Example: `tree.build_radii_array(&points, &radii)?`.
    pub fn build_radii_array(
        &mut self,
        points: &Array1<'_, P>,
        radii: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        if radii.len() < points.len() {
            self.clear();
            let index: i64 = i64::from(radii.lower()) + radii.len() as i64;
            return Err(Failure::out_of_range(&format!(
                "NCollection_Array1::Value: radius index {index}; expected [{}, {}]",
                radii.lower(),
                radii.upper()
            )));
        }
        self.build_radii(points.as_slice(), radii.as_slice());
        Ok(())
    }
}
impl<P, const D: usize, const R: bool> KDTree<P, D, R> {
    /// Takes the source's cloud and leaves it empty, as the native move constructor.
    /// Example: `let moved = KDTree::move_from(&mut tree);`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Replaces this tree with the source's cloud and empties the source, as native move
    /// assignment; `tree.move_assign(&mut other)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = std::mem::take(source);
    }
}
