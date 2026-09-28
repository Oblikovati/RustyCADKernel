// Created on: 1991-03-08
// Created by: Christophe MARION
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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::BoundingBox2;
use crate::gp::{Line2, Point2, Xy};
impl BoundingBox2 {
    /// Tests gap-adjusted closed sides; `bounds.is_out_point(point)`.
    pub fn is_out_point(&self, point: Point2) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        [point.x(), point.y()]
            .into_iter()
            .enumerate()
            .any(|(i, value)| {
                (!self.side_open(i, false) && value < self.minimum[i] - self.gap)
                    || (!self.side_open(i, true) && value > self.maximum[i] + self.gap)
            })
    }
    /// Includes boundaries and gap; `bounds.contains(point)`.
    pub fn contains(&self, point: Point2) -> bool {
        !self.is_out_point(point)
    }
    /// Retains the distinct native fast and open-box arithmetic; `bounds.is_out_box(&other)`.
    pub fn is_out_box(&self, other: &Self) -> bool {
        if self.flags == 0 && other.flags == 0 {
            return self.closed_box_out(other);
        }
        if self.is_void() || other.is_void() {
            return true;
        }
        if self.is_whole() || other.is_whole() {
            return false;
        }
        let limits = other.limits();
        (0..2).any(|i| {
            (!self.side_open(i, false) && limits[i + 2] < self.minimum[i] - self.gap)
                || (!self.side_open(i, true) && limits[i] > self.maximum[i] + self.gap)
        })
    }
    fn closed_box_out(&self, other: &Self) -> bool {
        let delta = other.gap + self.gap;
        (0..2).any(|i| {
            self.minimum[i] - other.maximum[i] > delta || other.minimum[i] - self.maximum[i] > delta
        })
    }
    /// Includes touching boxes; `bounds.intersects(&other)`.
    pub fn intersects(&self, other: &Self) -> bool {
        !self.is_out_box(other)
    }
    /// Uses the native cross-product separating test; `bounds.is_out_line(line)`.
    pub fn is_out_line(&self, line: Line2) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        let (center, height) = self.center_height();
        let (distance, extent) =
            line_products(line.location().xy(), line.direction().xy(), center, height);
        distance > extent
    }
    /// Checks the line then segment extents without normalizing its delta; `bounds.is_out_segment(first,last)`.
    pub fn is_out_segment(&self, first: Point2, last: Point2) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        let (center, height) = self.center_height();
        let delta = last.xy() - first.xy();
        segment_out(first.xy(), delta, center, height)
    }
    fn center_height(&self) -> (Xy, Xy) {
        let [x, y, u, v] = self.limits();
        let center = Xy::new((x + u) / 2.0, (y + v) / 2.0);
        (
            center,
            Xy::new((u - center.x()).abs(), (v - center.y()).abs()),
        )
    }
    /// Returns native per-axis separation, with zero for void boxes; `bounds.distance(&other)`.
    pub fn distance(&self, other: &Self) -> f64 {
        if self.is_void() || other.is_void() {
            return 0.0;
        }
        let left = self.limits();
        let right = other.limits();
        let [x, y] =
            std::array::from_fn(|i| distance_axis(left[i], left[i + 2], right[i], right[i + 2]));
        (x + y).sqrt()
    }
}
fn line_products(origin: Xy, delta: Xy, center: Xy, height: Xy) -> (f64, f64) {
    let distance = delta.crossed(center - origin).abs();
    let extent = (delta.x() * height.y()).abs() + (delta.y() * height.x()).abs();
    (distance, extent)
}
fn segment_out(origin: Xy, delta: Xy, center: Xy, height: Xy) -> bool {
    let (distance, extent) = line_products(origin, delta, center, height);
    if distance <= extent {
        let half = delta * 0.5;
        let difference = origin + half - center;
        return difference.x().abs() > height.x() + half.x().abs()
            || difference.y().abs() > height.y() + half.y().abs();
    }
    true
}
fn distance_axis(a: f64, b: f64, c: f64, d: f64) -> f64 {
    if a > d {
        return (a - d) * (a - d);
    }
    if c > b {
        return (c - b) * (c - b);
    }
    0.0
}
