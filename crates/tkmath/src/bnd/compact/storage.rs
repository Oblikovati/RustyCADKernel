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

use super::BoxReal;
/// Shared native center/half-size arithmetic for the public 2D and 3D aliases.
#[derive(Clone, Copy, Debug)]
pub struct CompactBounds<T: BoxReal, const N: usize> {
    pub(super) center: [T; N],
    pub(super) half: [T; N],
}
impl<T: BoxReal, const N: usize> Default for CompactBounds<T, N> {
    fn default() -> Self {
        Self {
            center: [T::from_double(1e30); N],
            half: [T::from_double(-1e30); N],
        }
    }
}
impl<T: BoxReal, const N: usize> CompactBounds<T, N> {
    /// Retains scalar arrays without conversion; `CompactBox2::from_arrays(center,half)`.
    pub const fn from_arrays(center: [T; N], half: [T; N]) -> Self {
        Self { center, half }
    }
    /// Resets to the native ±1e30 sentinel; `bounds.clear()`.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Tests only the X half-size against native -1e-5; `bounds.is_void()`.
    pub fn is_void(&self) -> bool {
        self.half[0].to_double() < -1e-5
    }
    /// Returns stored center coordinates; `bounds.center()`.
    pub const fn center(&self) -> &[T; N] {
        &self.center
    }
    /// Returns stored half extents; `bounds.half_size()`.
    pub const fn half_size(&self) -> &[T; N] {
        &self.half
    }
    /// Increases stored half sizes by the converted absolute amount; `bounds.enlarge(0.5)`.
    pub fn enlarge(&mut self, amount: f64) {
        let value = T::from_double(amount.abs());
        self.half = self.half.map(|h| h + value);
    }
    fn expand_axis(&mut self, axis: usize, coordinate: T) {
        let difference = coordinate - self.center[axis];
        if difference > self.half[axis] {
            let shift = (difference - self.half[axis]) / T::from_double(2.0);
            self.center[axis] = self.center[axis] + shift;
            self.half[axis] = self.half[axis] + shift;
        } else if difference < -self.half[axis] {
            let shift = (difference + self.half[axis]) / T::from_double(2.0);
            self.center[axis] = self.center[axis] + shift;
            self.half[axis] = self.half[axis] - shift;
        }
    }
    /// Tests separation in the stored scalar; `bounds.is_out_box(&other)`.
    pub fn is_out_box(&self, other: &Self) -> bool {
        (0..N).any(|i| (other.center[i] - self.center[i]).absolute() > other.half[i] + self.half[i])
    }
    /// Uses strict containment, so identical boxes are not inside; `bounds.is_in_box(&other)`.
    pub fn is_in_box(&self, other: &Self) -> bool {
        (0..N).all(|i| (other.center[i] - self.center[i]).absolute() < other.half[i] - self.half[i])
    }
    /// Preserves the native single-sided limiting branch per dimension; `bounds.limit(&other)`.
    pub fn limit(&mut self, other: &Self) -> bool {
        if self.is_out_box(other) {
            return false;
        }
        for axis in 0..N {
            self.limit_axis(other, axis);
        }
        true
    }
    fn limit_axis(&mut self, other: &Self, axis: usize) {
        let center_difference = other.center[axis] - self.center[axis];
        let half_difference = other.half[axis] - self.half[axis];
        if (center_difference - half_difference).to_double() > 0.0 {
            let shift = (center_difference - half_difference) / T::from_double(2.0);
            self.center[axis] = self.center[axis] + shift;
            self.half[axis] = self.half[axis] - shift;
        } else if (center_difference + half_difference).to_double() < 0.0 {
            let shift = (center_difference + half_difference) / T::from_double(2.0);
            self.center[axis] = self.center[axis] + shift;
            self.half[axis] = self.half[axis] + shift;
        }
    }
    /// Adds corners in native order, promoting only after scalar subtraction; `bounds.add_box(&other)`.
    pub fn add_box(&mut self, other: &Self) {
        if other.is_void() {
            return;
        }
        self.add_array(other.corner_array(false));
        self.add_array(other.corner_array(true));
    }
    pub(super) fn corner_array(&self, upper: bool) -> [T; N] {
        std::array::from_fn(|i| {
            if upper {
                self.center[i] + self.half[i]
            } else {
                self.center[i] - self.half[i]
            }
        })
    }
    pub(super) fn add_array(&mut self, point: [T; N]) {
        if self.is_void() {
            self.center = point;
            self.half = [T::from_double(0.0); N];
            return;
        }
        for (axis, coordinate) in point.into_iter().enumerate() {
            self.expand_axis(axis, coordinate);
        }
    }
    pub(super) fn is_out_array(&self, point: [T; N]) -> bool {
        point
            .into_iter()
            .enumerate()
            .any(|(i, value)| (value - self.center[i]).absolute() > self.half[i])
    }
}
