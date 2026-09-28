// Copyright (c) 2016 OPEN CASCADE SAS
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

// Rust translation of FoundationClasses/TKMath/gp/gp_TrsfNLerp.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_TrsfNLerp (`NCollection_Lerp<gp_Trsf>`): interpolates the rotation (gp_QuaternionNLerp),
//! the translation and the scale factor of two transformations independently.
use super::super::{Failure, QuaternionNLerp, Transform3, Vector3, Xyz};
use rustycad_tkernel::ncollection::Lerp;
use rustycad_tkernel::precision;
/// Part-wise transformation interpolator; `TransformNLerp::new(a, b).interpolate(0.5)?`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TransformNLerp {
    location: Lerp<Xyz>,
    scale: Lerp<f64>,
    rotation: QuaternionNLerp,
    start: Transform3,
    end: Transform3,
}
impl TransformNLerp {
    /// Initializes from both end transformations, as the main constructor;
    /// `TransformNLerp::new(a, b)`.
    pub fn new(start: Transform3, end: Transform3) -> Self {
        let mut lerp = Self::default();
        lerp.init(start, end);
        lerp
    }
    /// Stores both ends and splits them into translation, unit rotation and scale, as `Init`;
    /// `lerp.init(a, b)`.
    pub fn init(&mut self, start: Transform3, end: Transform3) {
        self.start = start;
        self.end = end;
        self.location
            .init(start.translation_part(), end.translation_part());
        self.rotation
            .init(start.rotation_quaternion(), end.rotation_quaternion());
        self.scale.init(start.scale_factor(), end.scale_factor());
    }
    /// Returns an exact end within Precision::Confusion of t = 0 or 1; otherwise rebuilds
    /// rotation, then translation, then scale. The scale step keeps gp_Trsf::SetScaleFactor's
    /// raise for an interpolated scale at or below gp::Resolution; `lerp.interpolate(0.5)?`.
    pub fn interpolate(&self, parameter: f64) -> Result<Transform3, Failure> {
        if (parameter - 0.0).abs() < precision::confusion() {
            return Ok(self.start);
        }
        if (parameter - 1.0).abs() < precision::confusion() {
            return Ok(self.end);
        }
        let location = self.location.interpolate(parameter);
        let rotation = self.rotation.interpolate(parameter);
        let scale = self.scale.interpolate(parameter);
        let mut result = Transform3::default();
        result.set_rotation_quaternion(rotation);
        result.set_translation_part(Vector3::from(location));
        result.set_scale_factor(scale)?;
        Ok(result)
    }
}
