// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::BoundingBox3;
use crate::gp::{Direction3, Point3};
impl BoundingBox3 {
    /// Resets the box before adding a point; `bounds.set_point(point)`.
    pub fn set_point(&mut self, point: Point3) {
        self.set_void();
        self.add_point(point);
    }
    /// Resets the box to a located infinite direction; `bounds.set_ray(point,direction)`.
    pub fn set_ray(&mut self, point: Point3, direction: Direction3) {
        self.set_void();
        self.add_ray(point, direction);
    }
    /// Accumulates a point with native comparison ordering; `bounds.add_point(point)`.
    pub fn add_point(&mut self, point: Point3) {
        self.update_bounds(*point.coordinates(), *point.coordinates());
    }
    /// Adds finite position before open flags; `bounds.add_ray(point,direction)`.
    pub fn add_ray(&mut self, point: Point3, direction: Direction3) {
        self.add_point(point);
        self.add_direction(direction);
    }
    /// Opens components exceeding machine epsilon without clearing voidness; `bounds.add_direction(direction)`.
    pub fn add_direction(&mut self, direction: Direction3) {
        for (axis, value) in direction.coordinates().iter().enumerate() {
            if *value < -f64::EPSILON {
                self.open_side(axis, false);
            } else if *value > f64::EPSILON {
                self.open_side(axis, true);
            }
        }
    }
    /// Updates raw corners without sorting supplied bounds or changing gap; `bounds.update_bounds([0.0;3],[1.0;3])`.
    pub fn update_bounds(&mut self, minimum: [f64; 3], maximum: [f64; 3]) {
        if self.is_void() {
            self.minimum = minimum;
            self.maximum = maximum;
            self.flags &= !1;
            return;
        }
        for axis in 0..3 {
            if minimum[axis] < self.minimum[axis] {
                self.minimum[axis] = minimum[axis];
            }
            if self.maximum[axis] < maximum[axis] {
                self.maximum[axis] = maximum[axis];
            }
        }
    }
    /// Unites flags and finite corners with the larger gap; `bounds.add_box(&other)`.
    pub fn add_box(&mut self, other: &Self) {
        if other.is_void() {
            return;
        }
        if self.is_void() {
            *self = other.clone();
            return;
        }
        if self.gap < other.gap {
            self.gap = other.gap;
        }
        if self.is_whole() {
            return;
        }
        if other.is_whole() {
            self.set_whole();
            return;
        }
        for axis in 0..3 {
            self.merge_axis(other, axis);
        }
    }
    fn merge_axis(&mut self, other: &Self, axis: usize) {
        for upper in [false, true] {
            if self.side_open(axis, upper) {
                continue;
            }
            if other.side_open(axis, upper) {
                self.open_side(axis, upper);
                continue;
            }
            self.merge_finite_side(other, axis, upper);
        }
    }
    fn merge_finite_side(&mut self, other: &Self, axis: usize, upper: bool) {
        if upper && self.maximum[axis] < other.maximum[axis] {
            self.maximum[axis] = other.maximum[axis];
        }
        if !upper && self.minimum[axis] > other.minimum[axis] {
            self.minimum[axis] = other.minimum[axis];
        }
    }
}
