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

use super::super::relation::TransformView;
use super::{BoxReal, CompactBox2};
use crate::gp::{Axis2d, Transform2, TransformForm, Xy};
impl<T: BoxReal> CompactBox2<T> {
    /// `IsOut(theBox, theTrsf)`: tests `other` mapped by `transform`; simple forms compare in
    /// the storage scalar, general forms test both boxes' separating axes in double;
    /// `bounds.is_out_transformed_box(&other, transform)`.
    pub fn is_out_transformed_box(&self, other: &Self, transform: Transform2) -> bool {
        self.out_of_transformed(other, &transform_view(other, transform))
    }
    /// `IsIn(theBox, theTrsf)`: strict inclusion of `self` in the transformed `other`;
    /// `bounds.is_in_transformed_box(&other, transform)`.
    pub fn is_in_transformed_box(&self, other: &Self, transform: Transform2) -> bool {
        self.in_transformed(other, &transform_view(other, transform))
    }
}
fn transform_view<T: BoxReal>(other: &CompactBox2<T>, transform: Transform2) -> TransformView<2> {
    let center = Xy::new(other.center[0].to_double(), other.center[1].to_double());
    let moved = transform.apply_coordinates(center);
    let offset = transform.translation_part();
    TransformView {
        simple: matches!(
            transform.form(),
            TransformForm::Translation
                | TransformForm::Identity
                | TransformForm::PointMirror
                | TransformForm::Scale
        ),
        scale: transform.scale_factor(),
        translation: [offset.x(), offset.y()],
        matrix: *transform.matrix().coefficients(),
        moved_center: [moved.x(), moved.y()],
    }
}
impl<T: BoxReal> CompactBox2<T> {
    /// Tests a disk or hollow circle, retaining strict hollow tangency; `bounds.is_out_circle(center,radius,false)`.
    pub fn is_out_circle(&self, center: Xy, radius: f64, hollow: bool) -> bool {
        self.ball_out([center.x(), center.y()], radius, hollow)
    }
    /// Uses the located direction's cross product with center displacement; `bounds.is_out_line(axis)`.
    pub fn is_out_line(&self, axis: Axis2d) -> bool {
        if self.is_void() {
            return true;
        }
        let (distance, extent) = self.line_products(axis.location().xy(), axis.direction().xy());
        distance > extent
    }
    /// Uses strict line intersection before segment overlap; `bounds.is_out_segment(first,last)`.
    pub fn is_out_segment(&self, first: Xy, last: Xy) -> bool {
        if self.is_void() {
            return true;
        }
        let delta = last - first;
        let (distance, extent) = self.line_products(first, delta);
        if distance < extent {
            return self.segment_bounds_out(first, delta);
        }
        true
    }
    fn line_products(&self, origin: Xy, delta: Xy) -> (f64, f64) {
        let center = Xy::new(self.center[0].to_double(), self.center[1].to_double());
        let distance = delta.crossed(center - origin).abs();
        let extent = (delta.x() * self.half[1].to_double()).abs()
            + (delta.y() * self.half[0].to_double()).abs();
        (distance, extent)
    }
    fn segment_bounds_out(&self, origin: Xy, delta: Xy) -> bool {
        let half = delta * 0.5;
        let center = Xy::new(self.center[0].to_double(), self.center[1].to_double());
        let displacement = origin + half - center;
        displacement.x().abs() > self.half[0].to_double() + half.x().abs()
            || displacement.y().abs() > self.half[1].to_double() + half.y().abs()
    }
    /// Transforms center and absolute matrix extents with native promotion; `bounds.transformed(transform)`.
    pub fn transformed(&self, transform: Transform2) -> Self {
        if transform.form() == TransformForm::Identity {
            return *self;
        }
        let center = self.center.map(BoxReal::to_double);
        if transform.form() == TransformForm::Translation {
            return self.translated_compact(transform, center);
        }
        let transformed = transform.apply_coordinates(Xy::new(center[0], center[1]));
        let half = self.transformed_half(transform);
        Self::new(transformed, Xy::new(half[0], half[1]))
    }
    fn translated_compact(&self, transform: Transform2, center: [f64; 2]) -> Self {
        let offset = transform.translation_part();
        let scale = transform.scale_factor();
        Self::from_arrays(
            [
                T::from_double(center[0] * scale + offset.x()),
                T::from_double(center[1] * scale + offset.y()),
            ],
            self.half
                .map(|h| T::from_double(h.to_double() * scale.abs())),
        )
    }
    fn transformed_half(&self, transform: Transform2) -> [f64; 2] {
        let matrix = *transform.matrix().coefficients();
        let half = self.half.map(BoxReal::to_double);
        std::array::from_fn(|i| {
            transform.scale_factor().abs()
                * (matrix[i][0].abs() * half[0] + matrix[i][1].abs() * half[1])
        })
    }
}
