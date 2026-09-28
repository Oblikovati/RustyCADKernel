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

//! Positional members that gp conics and quadrics delegate verbatim to their placement frame.
//!
//! Every gp_Circ/gp_Elips/gp_Hypr/gp_Parab (and their 2D and quadric counterparts) forwards
//! Mirror/Rotate/Translate to `pos` with identical code, so one macro per placement kind keeps
//! those members in one place. Length scaling differs per class and stays hand-written.

/// Multiplies a length and negates a negative product, keeping `-0.0` like the native
/// `r *= s; if (r < 0) r = -r;` sequence; `scaled_length(2.0,-3.0)`.
pub(crate) fn scaled_length(length: f64, factor: f64) -> f64 {
    let scaled = length * factor;
    if scaled < 0.0 { -scaled } else { scaled }
}

/// Returns `|factor|` via the native `if (s < 0) s = -s;` sequence; `absolute_factor(-2.0)`.
pub(crate) fn absolute_factor(factor: f64) -> f64 {
    if factor < 0.0 { -factor } else { factor }
}

/// Sums `T(r,i)·T(r,j)` over the given frame-change rows left to right, starting from the
/// first product (no `0.0` accumulator, which would turn a `-0.0` sum into `+0.0`);
/// `column_product(&rows,0,3)`.
pub(crate) fn column_product(rows: &[[f64; 4]], i: usize, j: usize) -> f64 {
    let mut products = rows.iter().map(|row| row[i] * row[j]);
    let first = products.next().unwrap_or_default();
    products.fold(first, |sum, product| sum + product)
}

/// Members of a curve placed by a right-handed `Axis2` stored in `position`.
macro_rules! axis2_curve_placement {
    ($curve:ty) => {
        impl $curve {
            /// Returns the main axis (location and normal); `curve.axis()`.
            pub fn axis(&self) -> $crate::gp::Axis1 {
                self.position.axis()
            }
            /// Returns the centre or apex; `curve.location()`.
            pub const fn location(&self) -> $crate::gp::Point3 {
                self.position.location()
            }
            /// Returns the placement frame; `curve.position()`.
            pub const fn position(&self) -> $crate::gp::Axis2 {
                self.position
            }
            /// Changes the main axis with gp_Ax2::SetAxis; `curve.set_axis(axis)?`.
            pub fn set_axis(
                &mut self,
                axis: $crate::gp::Axis1,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.set_axis(axis)
            }
            /// Replaces the placement frame; `curve.set_position(frame)`.
            pub fn set_position(&mut self, position: $crate::gp::Axis2) {
                self.position = position;
            }
            /// Returns the X axis through the location; `curve.x_axis()`.
            pub fn x_axis(&self) -> $crate::gp::Axis1 {
                $crate::gp::Axis1::new(self.location(), self.position.x_direction())
            }
            /// Returns the Y axis through the location; `curve.y_axis()`.
            pub fn y_axis(&self) -> $crate::gp::Axis1 {
                $crate::gp::Axis1::new(self.location(), self.position.y_direction())
            }
            /// Mirrors the frame through a point; `curve.mirrored_point(center)`.
            pub fn mirrored_point(mut self, center: $crate::gp::Point3) -> Self {
                self.position.mirror_point(center);
                self
            }
            /// Mirrors through a point in place; `curve.mirror_point(center)`.
            pub fn mirror_point(&mut self, center: $crate::gp::Point3) {
                self.position.mirror_point(center);
            }
            /// Mirrors the frame about an axis; `curve.mirrored_axis(axis)?`.
            pub fn mirrored_axis(
                mut self,
                axis: $crate::gp::Axis1,
            ) -> Result<Self, rustycad_tkernel::standard::Failure> {
                self.position.mirror_axis(axis)?;
                Ok(self)
            }
            /// Mirrors about an axis in place; `curve.mirror_axis(axis)?`.
            pub fn mirror_axis(
                &mut self,
                axis: $crate::gp::Axis1,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.mirror_axis(axis)
            }
            /// Mirrors the frame through a plane; `curve.mirrored_plane(plane)?`.
            pub fn mirrored_plane(
                mut self,
                plane: $crate::gp::Axis2,
            ) -> Result<Self, rustycad_tkernel::standard::Failure> {
                self.position.mirror_plane(plane)?;
                Ok(self)
            }
            /// Mirrors through a plane in place; `curve.mirror_plane(plane)?`.
            pub fn mirror_plane(
                &mut self,
                plane: $crate::gp::Axis2,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.mirror_plane(plane)
            }
            /// Rotates the frame; `curve.rotated(axis,angle)?`.
            pub fn rotated(
                mut self,
                axis: $crate::gp::Axis1,
                angle: f64,
            ) -> Result<Self, rustycad_tkernel::standard::Failure> {
                self.position.rotate(axis, angle)?;
                Ok(self)
            }
            /// Rotates in place; `curve.rotate(axis,angle)?`.
            pub fn rotate(
                &mut self,
                axis: $crate::gp::Axis1,
                angle: f64,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.rotate(axis, angle)
            }
            /// Translates the frame origin; `curve.translated(vector)`.
            pub fn translated(mut self, vector: $crate::gp::Vector3) -> Self {
                self.position.translate(vector);
                self
            }
            /// Translates in place; `curve.translate(vector)`.
            pub fn translate(&mut self, vector: $crate::gp::Vector3) {
                self.position.translate(vector);
            }
            /// Translates with gp_Pnt's add-then-subtract order; `curve.translated_between(from,to)`.
            pub fn translated_between(
                mut self,
                from: $crate::gp::Point3,
                to: $crate::gp::Point3,
            ) -> Self {
                self.position.translate_between(from, to);
                self
            }
            /// Translates between points in place; `curve.translate_between(from,to)`.
            pub fn translate_between(&mut self, from: $crate::gp::Point3, to: $crate::gp::Point3) {
                self.position.translate_between(from, to);
            }
        }
    };
}
pub(crate) use axis2_curve_placement;

/// Members of a quadric placed by a possibly indirect `Axis3` stored in `position`.
macro_rules! axis3_surface_placement {
    ($surface:ty) => {
        impl $surface {
            /// Returns the centre or reference location; `surface.location()`.
            pub const fn location(&self) -> $crate::gp::Point3 {
                self.position.location()
            }
            /// Returns the local coordinate system; `surface.position()`.
            pub const fn position(&self) -> $crate::gp::Axis3 {
                self.position
            }
            /// Moves the local origin; `surface.set_location(point)`.
            pub fn set_location(&mut self, location: $crate::gp::Point3) {
                self.position.set_location(location);
            }
            /// Replaces the local coordinate system; `surface.set_position(frame)`.
            pub fn set_position(&mut self, position: $crate::gp::Axis3) {
                self.position = position;
            }
            /// Reverses the U parametrization by reversing Y; `surface.u_reverse()`.
            pub fn u_reverse(&mut self) {
                self.position.y_reverse();
            }
            /// Tests whether the local system is right-handed; `surface.direct()?`.
            pub fn direct(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.position.direct()
            }
            /// Returns the X axis through the location; `surface.x_axis()`.
            pub fn x_axis(&self) -> $crate::gp::Axis1 {
                $crate::gp::Axis1::new(self.location(), self.position.x_direction())
            }
            /// Returns the Y axis through the location; `surface.y_axis()`.
            pub fn y_axis(&self) -> $crate::gp::Axis1 {
                $crate::gp::Axis1::new(self.location(), self.position.y_direction())
            }
            /// Mirrors the frame through a point; `surface.mirrored_point(center)`.
            pub fn mirrored_point(mut self, center: $crate::gp::Point3) -> Self {
                self.position.mirror_point(center);
                self
            }
            /// Mirrors through a point in place; `surface.mirror_point(center)`.
            pub fn mirror_point(&mut self, center: $crate::gp::Point3) {
                self.position.mirror_point(center);
            }
            /// Mirrors the frame about an axis; `surface.mirrored_axis(axis)`.
            pub fn mirrored_axis(mut self, axis: $crate::gp::Axis1) -> Self {
                self.position.mirror_axis(axis);
                self
            }
            /// Mirrors about an axis in place; `surface.mirror_axis(axis)`.
            pub fn mirror_axis(&mut self, axis: $crate::gp::Axis1) {
                self.position.mirror_axis(axis);
            }
            /// Mirrors the frame through a plane; `surface.mirrored_plane(plane)`.
            pub fn mirrored_plane(mut self, plane: $crate::gp::Axis2) -> Self {
                self.position.mirror_plane(plane);
                self
            }
            /// Mirrors through a plane in place; `surface.mirror_plane(plane)`.
            pub fn mirror_plane(&mut self, plane: $crate::gp::Axis2) {
                self.position.mirror_plane(plane);
            }
            /// Rotates the frame; `surface.rotated(axis,angle)?`.
            pub fn rotated(
                mut self,
                axis: $crate::gp::Axis1,
                angle: f64,
            ) -> Result<Self, rustycad_tkernel::standard::Failure> {
                self.position.rotate(axis, angle)?;
                Ok(self)
            }
            /// Rotates in place; `surface.rotate(axis,angle)?`.
            pub fn rotate(
                &mut self,
                axis: $crate::gp::Axis1,
                angle: f64,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.rotate(axis, angle)
            }
            /// Translates the frame origin; `surface.translated(vector)`.
            pub fn translated(mut self, vector: $crate::gp::Vector3) -> Self {
                self.position.translate(vector);
                self
            }
            /// Translates in place; `surface.translate(vector)`.
            pub fn translate(&mut self, vector: $crate::gp::Vector3) {
                self.position.translate(vector);
            }
            /// Translates by gp_Vec(from,to) as gp_Ax3 does; `surface.translated_between(from,to)`.
            pub fn translated_between(
                mut self,
                from: $crate::gp::Point3,
                to: $crate::gp::Point3,
            ) -> Self {
                self.position.translate_between(from, to);
                self
            }
            /// Translates between points in place; `surface.translate_between(from,to)`.
            pub fn translate_between(&mut self, from: $crate::gp::Point3, to: $crate::gp::Point3) {
                self.position.translate_between(from, to);
            }
        }
    };
}
pub(crate) use axis3_surface_placement;

/// Main-axis members shared by the cylinder, cone and torus (the sphere has none).
macro_rules! axis3_surface_axis {
    ($surface:ty) => {
        impl $surface {
            /// Returns the symmetry axis; `surface.axis()`.
            pub fn axis(&self) -> $crate::gp::Axis1 {
                self.position.axis()
            }
            /// Changes the symmetry axis with gp_Ax3::SetAxis; `surface.set_axis(axis)?`.
            pub fn set_axis(
                &mut self,
                axis: $crate::gp::Axis1,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.set_axis(axis)
            }
        }
    };
}
pub(crate) use axis3_surface_axis;
