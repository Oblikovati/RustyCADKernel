// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG2d/Geom2dHash/Geom2dHash_LineHasher.pxx, _CircleHasher.pxx, _EllipseHasher.pxx, _HyperbolaHasher.pxx, _ParabolaHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the line and the conics: the placement hash combined with the rounded
//! radii (or focal length), and a placement comparison plus radii within tolerance.
//!
//! The native hashers take a handle and dereference it; these take the object.
use super::hash_key::{combined_hash, rounded_hash};
use super::placement_hashers::{
    Geom2dHashAxisPlacement, Geom2dHashDirectionHasher, Geom2dHashPointHasher,
};
use crate::geom2d::Geom2dParabola;
use crate::geom2d::{Geom2dCircle, Geom2dConic, Geom2dEllipse, Geom2dHyperbola, Geom2dLine};
use rustycad_tkmath::gp::Axis22d;

hasher_tolerances!(
    /// `Geom2dHash_LineHasher`: a line by location and direction;
    /// `Geom2dHashLineHasher::default().hash(&line)`.
    Geom2dHashLineHasher
);
hasher_tolerances!(
    /// `Geom2dHash_CircleHasher`: a circle by placement and radius;
    /// `Geom2dHashCircleHasher::default().hash(&circle)`.
    Geom2dHashCircleHasher
);
hasher_tolerances!(
    /// `Geom2dHash_EllipseHasher`: an ellipse by placement and both radii;
    /// `Geom2dHashEllipseHasher::default().hash(&ellipse)`.
    Geom2dHashEllipseHasher
);
hasher_tolerances!(
    /// `Geom2dHash_HyperbolaHasher`: a hyperbola by placement and both radii;
    /// `Geom2dHashHyperbolaHasher::default().hash(&hyperbola)`.
    Geom2dHashHyperbolaHasher
);
hasher_tolerances!(
    /// `Geom2dHash_ParabolaHasher`: a parabola by placement and focal length;
    /// `Geom2dHashParabolaHasher::default().hash(&parabola)`.
    Geom2dHashParabolaHasher
);

// `[axis hash, round(value_i / HashTolerance)...]` combined, the shape of every conic hash.
fn placement_and_values(axes: Geom2dHashAxisPlacement, frame: Axis22d, values: &[f64]) -> usize {
    let factor: f64 = 1.0 / axes.hash_tolerance;
    let mut hashes: Vec<usize> = vec![axes.hash(frame)];
    hashes.extend(values.iter().map(|&value| rounded_hash(value, factor)));
    combined_hash(&hashes)
}
// The placements equal, then each value pair within CompTolerance, in order.
fn placement_and_values_equal(
    axes: Geom2dHashAxisPlacement,
    frames: [Axis22d; 2],
    values: &[(f64, f64)],
) -> bool {
    axes.equal(frames[0], frames[1])
        && values
            .iter()
            .all(|&(first, second)| (first - second).abs() <= axes.comp_tolerance)
}

impl Geom2dHashLineHasher {
    /// Location and direction hashes combined (`operator()(handle)`); `hasher.hash(&line)`.
    pub fn hash(&self, line: &Geom2dLine) -> usize {
        let position = line.position();
        let points = Geom2dHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let directions = Geom2dHashDirectionHasher::new(self.comp_tolerance, self.hash_tolerance);
        combined_hash(&[
            points.hash(position.location()),
            directions.hash(position.direction()),
        ])
    }
    /// Locations and directions equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dLine, second: &Geom2dLine) -> bool {
        let (a, b) = (first.position(), second.position());
        let points = Geom2dHashPointHasher::new(self.comp_tolerance, self.hash_tolerance);
        let directions = Geom2dHashDirectionHasher::new(self.comp_tolerance, self.hash_tolerance);
        points.equal(a.location(), b.location()) && directions.equal(a.direction(), b.direction())
    }
}

impl Geom2dHashCircleHasher {
    fn axes(&self) -> Geom2dHashAxisPlacement {
        Geom2dHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded radius (`operator()(handle)`); `hasher.hash(&circle)`.
    pub fn hash(&self, circle: &Geom2dCircle) -> usize {
        placement_and_values(self.axes(), circle.position(), &[circle.radius()])
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dCircle, second: &Geom2dCircle) -> bool {
        let frames = [first.position(), second.position()];
        placement_and_values_equal(self.axes(), frames, &[(first.radius(), second.radius())])
    }
}

impl Geom2dHashEllipseHasher {
    fn axes(&self) -> Geom2dHashAxisPlacement {
        Geom2dHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded radii (`operator()(handle)`); `hasher.hash(&ellipse)`.
    pub fn hash(&self, ellipse: &Geom2dEllipse) -> usize {
        let radii = [ellipse.major_radius(), ellipse.minor_radius()];
        placement_and_values(self.axes(), ellipse.position(), &radii)
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dEllipse, second: &Geom2dEllipse) -> bool {
        let frames = [first.position(), second.position()];
        let radii = [
            (first.major_radius(), second.major_radius()),
            (first.minor_radius(), second.minor_radius()),
        ];
        placement_and_values_equal(self.axes(), frames, &radii)
    }
}

impl Geom2dHashHyperbolaHasher {
    fn axes(&self) -> Geom2dHashAxisPlacement {
        Geom2dHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded radii (`operator()(handle)`); `hasher.hash(&hyperbola)`.
    pub fn hash(&self, hyperbola: &Geom2dHyperbola) -> usize {
        let radii = [hyperbola.major_radius(), hyperbola.minor_radius()];
        placement_and_values(self.axes(), hyperbola.position(), &radii)
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dHyperbola, second: &Geom2dHyperbola) -> bool {
        let frames = [first.position(), second.position()];
        let radii = [
            (first.major_radius(), second.major_radius()),
            (first.minor_radius(), second.minor_radius()),
        ];
        placement_and_values_equal(self.axes(), frames, &radii)
    }
}

impl Geom2dHashParabolaHasher {
    fn axes(&self) -> Geom2dHashAxisPlacement {
        Geom2dHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded focal length (`operator()(handle)`); `hasher.hash(&parabola)`.
    pub fn hash(&self, parabola: &Geom2dParabola) -> usize {
        placement_and_values(self.axes(), parabola.position(), &[parabola.focal()])
    }
    /// Placements and focal lengths equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Geom2dParabola, second: &Geom2dParabola) -> bool {
        let frames = [first.position(), second.position()];
        placement_and_values_equal(self.axes(), frames, &[(first.focal(), second.focal())])
    }
}
