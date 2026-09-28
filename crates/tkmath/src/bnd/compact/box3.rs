// Created on: 2005-09-08
// Created by: Alexander GRIGORIEV
// Copyright (c) 2005-2014 OPEN CASCADE SAS
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

use super::{BoxReal, storage::CompactBounds};
use crate::gp::{Point3, Transform3, TransformForm, Xyz};
/// Native Bnd_B3<double/float> storage; `CompactBox3::<f64>::new(center,half_size)`.
pub type CompactBox3<T = f64> = CompactBounds<T, 3>;
impl<T: BoxReal> CompactBounds<T, 3> {
    /// Converts coordinates to the requested scalar storage; `CompactBox3::<f32>::new(center,half)`.
    pub fn new(center: Xyz, half: Xyz) -> Self {
        Self::from_arrays(convert(center), convert(half))
    }
    /// Replaces the center with native conversion; `bounds.set_center(center)`.
    pub fn set_center(&mut self, center: Xyz) {
        self.center = convert(center);
    }
    /// Retains supplied half sizes without validation; `bounds.set_half_size(half)`.
    pub fn set_half_size(&mut self, half: Xyz) {
        self.half = convert(half);
    }
    /// Subtracts in scalar precision before promoting; `bounds.corner_min()`.
    pub fn corner_min(&self) -> Xyz {
        triple(self.corner_array(false).map(BoxReal::to_double))
    }
    /// Adds in scalar precision before promoting; `bounds.corner_max()`.
    pub fn corner_max(&self) -> Xyz {
        triple(self.corner_array(true).map(BoxReal::to_double))
    }
    /// Retains native scalar accumulation order; `bounds.square_extent()`.
    pub fn square_extent(&self) -> f64 {
        let [x, y, z] = self.half;
        (T::from_double(4.0) * (x * x + y * y + z * z)).to_double()
    }
    /// Expands via coordinate algebra; `bounds.add_point(point)`.
    pub fn add_point(&mut self, point: Point3) {
        self.add_xyz(point.xyz());
    }
    /// Uses the shared native half-shift arithmetic; `bounds.add_xyz(point)`.
    pub fn add_xyz(&mut self, point: Xyz) {
        self.add_array(convert(point));
    }
    /// Converts coordinates before checking half extents; `bounds.is_out_point(point)`.
    pub fn is_out_point(&self, point: Xyz) -> bool {
        self.is_out_array(convert(point))
    }
    /// Transforms the center and absolute matrix extents; `bounds.transformed(transform)`.
    pub fn transformed(&self, transform: Transform3) -> Self {
        if transform.form() == TransformForm::Identity {
            return *self;
        }
        let center = self.center.map(BoxReal::to_double);
        if matches!(
            transform.form(),
            TransformForm::Translation | TransformForm::Scale
        ) {
            return self.translated_compact(center, transform);
        }
        let transformed = transform.apply_coordinates(triple(center));
        let half = self.transformed_half(transform);
        Self::new(transformed, triple(half))
    }
    fn translated_compact(&self, center: [f64; 3], transform: Transform3) -> Self {
        let offset = transform.translation_part();
        let translated = std::array::from_fn(|i| {
            T::from_double(center[i] * transform.scale_factor() + offset.coordinates()[i])
        });
        Self::from_arrays(
            translated,
            self.half
                .map(|h| T::from_double(h.to_double() * transform.scale_factor().abs())),
        )
    }
    fn transformed_half(&self, transform: Transform3) -> [f64; 3] {
        let matrix = transform.matrix();
        let half = self.half.map(BoxReal::to_double);
        std::array::from_fn(|i| {
            let row = matrix.coefficients()[i];
            transform.scale_factor().abs()
                * (row[0].abs() * half[0] + row[1].abs() * half[1] + row[2].abs() * half[2])
        })
    }
}
fn convert<T: BoxReal>(xyz: Xyz) -> [T; 3] {
    xyz.coordinates().map(T::from_double)
}
fn triple(values: [f64; 3]) -> Xyz {
    Xyz::new(values[0], values[1], values[2])
}
mod queries;
