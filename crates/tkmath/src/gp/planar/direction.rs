// Copyright (c) 1995-1999 Matra Datavision
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

use super::{
    Axis2d, Failure, Point2, Transform2, TransformForm, Vector2, Xy, construction, oriented_angle,
};
/// A normalized planar direction; native mirrored/parsed states may depart from unit length; `Direction2::new(1.0,0.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Direction2 {
    coordinates: Xy,
}
impl Default for Direction2 {
    fn default() -> Self {
        Self::X
    }
}
impl Direction2 {
    /// Negates stored components without normalization; `direction.reversed()`.
    pub fn reversed(self) -> Self {
        Self {
            coordinates: self.coordinates.reversed(),
        }
    }

    /// Positive X axis; `Direction2::X`.
    pub const X: Self = Self {
        coordinates: Xy::new(1.0, 0.0),
    };
    /// Positive Y axis; `Direction2::Y`.
    pub const Y: Self = Self {
        coordinates: Xy::new(0.0, 1.0),
    };
    /// Negative Y axis; `Direction2::NEG_Y`.
    pub const NEG_Y: Self = Self {
        coordinates: Xy::new(0.0, -1.0),
    };
    /// Normalizes using the native squared-norm fast path; `Direction2::new(3.0,4.0)?`.
    pub fn new(x: f64, y: f64) -> Result<Self, Failure> {
        let xy = Xy::new(x, y);
        let squared = xy.square_modulus();
        if squared == 1.0 {
            return Ok(Self { coordinates: xy });
        }
        if squared <= 0.0 {
            return Err(construction("gp_Dir2d() - input vector has zero norm"));
        }
        Ok(Self {
            coordinates: xy / squared.sqrt(),
        })
    }
    /// Constructs from planar algebra; `Direction2::from_xy(xy)?`.
    pub fn from_xy(xy: Xy) -> Result<Self, Failure> {
        Self::new(xy.x(), xy.y())
    }
    /// Constructs from a vector; `Direction2::from_vector(vector)?`.
    pub fn from_vector(vector: Vector2) -> Result<Self, Failure> {
        Self::from_xy(vector.xy())
    }
    /// Returns coordinate algebra; `direction.xy()`.
    pub const fn xy(&self) -> Xy {
        self.coordinates
    }
    /// Reads X; `direction.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates.x()
    }
    /// Reads Y; `direction.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates.y()
    }
    /// Computes the signed native angle, including -PI for opposite X axes; `direction.angle(other)`.
    #[allow(clippy::approx_constant)]
    pub fn angle(self, other: Self) -> f64 {
        oriented_angle(self.dot(other), self.crossed(other), 0.70710678118655)
    }
    /// Compares angular separation; `direction.is_equal(other,1e-12)`.
    pub fn is_equal(self, other: Self, tolerance: f64) -> bool {
        self.angle(other).abs() <= tolerance
    }
    /// Tests both signs of a right angle; `direction.is_normal(other,1e-12)`.
    pub fn is_normal(self, other: Self, tolerance: f64) -> bool {
        (std::f64::consts::FRAC_PI_2 - self.angle(other).abs()).abs() <= tolerance
    }
    /// Tests opposite directions; `direction.is_opposite(other,1e-12)`.
    pub fn is_opposite(self, other: Self, tolerance: f64) -> bool {
        std::f64::consts::PI - self.angle(other).abs() <= tolerance
    }
    /// Accepts parallel or opposite directions; `direction.is_parallel(other,1e-12)`.
    pub fn is_parallel(self, other: Self, tolerance: f64) -> bool {
        let angle = self.angle(other).abs();
        angle <= tolerance || std::f64::consts::PI - angle <= tolerance
    }
    /// Computes scalar product; `direction.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        self.coordinates.dot(other.coordinates)
    }
    /// Computes signed planar cross product; `direction.crossed(other)`.
    pub fn crossed(self, other: Self) -> f64 {
        self.coordinates.crossed(other.coordinates)
    }
    /// Applies rotation without an extra normalization; `direction.rotated(angle)`.
    pub fn rotated(self, angle: f64) -> Self {
        Self {
            coordinates: Transform2::rotation(Point2::default(), angle)
                .apply_vector(self.coordinates),
        }
    }
    /// Mirrors about an axis using native direct component expressions; `direction.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis2d) -> Self {
        let normal = axis.direction();
        let (a, b, x, y) = (normal.x(), normal.y(), self.x(), self.y());
        let mixed = 2.0 * a * b;
        Self {
            coordinates: Xy::new(
                ((2.0 * a * a) - 1.0) * x + mixed * y,
                mixed * x + ((2.0 * b * b) - 1.0) * y,
            ),
        }
    }
    /// Transforms and renormalizes, preserving the native scale sign; `direction.transformed(transform)`.
    pub fn transformed(self, transform: Transform2) -> Self {
        if matches!(
            transform.form(),
            TransformForm::Identity | TransformForm::Translation
        ) {
            return self;
        }
        let xy = transform.apply_unscaled(self.coordinates);
        let mut xy = xy / xy.modulus();
        if transform.scale_factor() < 0.0 {
            xy = xy.reversed();
        }
        Self { coordinates: xy }
    }
}
