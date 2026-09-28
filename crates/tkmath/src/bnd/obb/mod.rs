// Created by: Eugeny MALTCHIKOV
// Copyright (c) 2017 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_OBB.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bnd_OBB`: oriented bounding boxes built by the DiTO algorithm.
use super::BoundingBox3;
use crate::gp::{Axis3, Direction3, Point3, Xyz};
use rustycad_tkernel::standard::Failure;

mod accumulate;
mod dito;
mod json;
mod queries;

/// The three half sizes of an oriented box, as `Bnd_OBB::HalfSizes`.
///
/// Example: `let ObbHalfSizes { x, y, z } = obb.half_sizes();`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObbHalfSizes {
    /// Half size along the X axis.
    pub x: f64,
    /// Half size along the Y axis.
    pub y: f64,
    /// Half size along the Z axis.
    pub z: f64,
}

/// Center, three axes and three half sizes; void while any half size is negative.
///
/// The axes are stored as coordinate triples, not directions, exactly like the native
/// `gp_XYZ myAxes[3]`, so `set_void` leaves null axes. Example:
/// `OrientedBoundingBox::new(Point3::new(0.0,0.0,0.0), [Direction3::X, Direction3::Y, Direction3::Z], 1.0, 2.0, 3.0)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrientedBoundingBox {
    center: Xyz,
    axes: [Xyz; 3],
    half_sizes: [f64; 3],
    is_aa_box: bool,
}

impl Default for OrientedBoundingBox {
    fn default() -> Self {
        Self {
            center: Xyz::default(),
            axes: [Xyz::default(); 3],
            half_sizes: [-1.0; 3],
            is_aa_box: false,
        }
    }
}

impl OrientedBoundingBox {
    /// Stores every defining parameter; the native negative-size assertions are
    /// `Standard_ASSERT_VOID`, compiled out of release builds, so negative sizes are kept
    /// (and make the box void). Example: see the type documentation.
    pub fn new(
        center: Point3,
        axes: [Direction3; 3],
        half_x: f64,
        half_y: f64,
        half_z: f64,
    ) -> Self {
        Self {
            center: center.xyz(),
            axes: axes.map(Xyz::from),
            half_sizes: [half_x, half_y, half_z],
            is_aa_box: false,
        }
    }
    /// Converts an axis-aligned box, using its gap-adjusted limits; a void box gives the
    /// void OBB. Example: `OrientedBoundingBox::from_box(&bounds)`.
    pub fn from_box(bounds: &BoundingBox3) -> Self {
        let Ok([x1, y1, z1, x2, y2, z2]) = bounds.get() else {
            return Self::default();
        };
        Self {
            center: Xyz::new(0.5 * (x2 + x1), 0.5 * (y2 + y1), 0.5 * (z2 + z1)),
            axes: [Direction3::X, Direction3::Y, Direction3::Z].map(Xyz::from),
            half_sizes: [0.5 * (x2 - x1), 0.5 * (y2 - y1), 0.5 * (z2 - z1)],
            is_aa_box: true,
        }
    }
    /// Sets the center; `obb.set_center(Point3::new(1.0, 2.0, 3.0))`.
    pub fn set_center(&mut self, center: Point3) {
        self.center = center.xyz();
    }
    /// Sets the X direction and half size (negative sizes kept, see `new`);
    /// `obb.set_x_component(Direction3::X, 2.0)`.
    pub fn set_x_component(&mut self, direction: Direction3, half_size: f64) {
        self.set_component(0, direction, half_size);
    }
    /// Sets the Y direction and half size; `obb.set_y_component(Direction3::Y, 2.0)`.
    pub fn set_y_component(&mut self, direction: Direction3, half_size: f64) {
        self.set_component(1, direction, half_size);
    }
    /// Sets the Z direction and half size; `obb.set_z_component(Direction3::Z, 2.0)`.
    pub fn set_z_component(&mut self, direction: Direction3, half_size: f64) {
        self.set_component(2, direction, half_size);
    }
    fn set_component(&mut self, axis: usize, direction: Direction3, half_size: f64) {
        self.axes[axis] = Xyz::from(direction);
        self.half_sizes[axis] = half_size;
    }
    /// Local frame `gp_Ax3(Center, ZDirection, XDirection)`; the axis triples are converted to
    /// directions first, so null axes of a void box raise `Standard_ConstructionError`.
    /// Example: `obb.position()?.location()`.
    pub fn position(&self) -> Result<Axis3, Failure> {
        let normal = Direction3::from_xyz(self.axes[2])?;
        let x_direction = Direction3::from_xyz(self.axes[0])?;
        Axis3::new(Point3::from(self.center), normal, x_direction)
    }
    /// Returns the center; `obb.center()`.
    pub const fn center(&self) -> Xyz {
        self.center
    }
    /// Returns the X axis; `obb.x_direction()`.
    pub const fn x_direction(&self) -> Xyz {
        self.axes[0]
    }
    /// Returns the Y axis; `obb.y_direction()`.
    pub const fn y_direction(&self) -> Xyz {
        self.axes[1]
    }
    /// Returns the Z axis; `obb.z_direction()`.
    pub const fn z_direction(&self) -> Xyz {
        self.axes[2]
    }
    /// Returns the X half size; `obb.x_h_size()`.
    pub const fn x_h_size(&self) -> f64 {
        self.half_sizes[0]
    }
    /// Returns the Y half size; `obb.y_h_size()`.
    pub const fn y_h_size(&self) -> f64 {
        self.half_sizes[1]
    }
    /// Returns the Z half size; `obb.z_h_size()`.
    pub const fn z_h_size(&self) -> f64 {
        self.half_sizes[2]
    }
    /// Returns the three half sizes (`GetHalfSizes`); `obb.half_sizes().y`.
    pub const fn half_sizes(&self) -> ObbHalfSizes {
        let [x, y, z] = self.half_sizes;
        ObbHalfSizes { x, y, z }
    }
    /// True when any half size is negative (NaN sizes do not make it void);
    /// `OrientedBoundingBox::default().is_void()`.
    pub fn is_void(&self) -> bool {
        self.half_sizes.iter().any(|size| *size < 0.0)
    }
    /// Clears the box to half sizes -1 and a null center and null axes; `obb.set_void()`.
    pub fn set_void(&mut self) {
        *self = Self::default();
    }
    /// Sets the axis-aligned flag; `obb.set_aa_box(true)`.
    pub fn set_aa_box(&mut self, is_aa_box: bool) {
        self.is_aa_box = is_aa_box;
    }
    /// Returns the axis-aligned flag; `obb.is_aa_box()`.
    pub const fn is_aa_box(&self) -> bool {
        self.is_aa_box
    }
    /// Adds the absolute gap to every half size, even on a void box; `obb.enlarge(0.1)`.
    pub fn enlarge(&mut self, gap: f64) {
        let gap = gap.abs();
        self.half_sizes = self.half_sizes.map(|size| size + gap);
    }
    /// The eight corners in the native `GetVertex` order (index bit 0 = +X, bit 1 = +Y,
    /// bit 2 = +Z), or `None` for a void box; `obb.vertices()`.
    pub fn vertices(&self) -> Option<[Point3; 8]> {
        if self.is_void() {
            return None;
        }
        Some(std::array::from_fn(|index| {
            Point3::from(self.vertex(index))
        }))
    }
    /// `myCenter ∓ h0*a0 ∓ h1*a1 ∓ h2*a2`, accumulated left to right like the native sum.
    fn vertex(&self, index: usize) -> Xyz {
        (0..3).fold(self.center, |sum, axis| {
            let step = self.half_sizes[axis] * self.axes[axis];
            if index & (1 << axis) == 0 {
                sum - step
            } else {
                sum + step
            }
        })
    }
    /// Squared diagonal `4*(hx²+hy²+hz²)`; `obb.square_extent()`.
    pub fn square_extent(&self) -> f64 {
        let [x, y, z] = self.half_sizes;
        4.0 * (x * x + y * y + z * z)
    }
    /// `ProcessOnePoint`: a zero-size axis-aligned box at the point.
    fn process_one_point(&mut self, point: Xyz) {
        self.is_aa_box = true;
        self.half_sizes = [0.0; 3];
        self.axes = [Direction3::X, Direction3::Y, Direction3::Z].map(Xyz::from);
        self.center = point;
    }
}
