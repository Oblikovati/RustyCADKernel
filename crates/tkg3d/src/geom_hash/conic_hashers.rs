// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_LineHasher.pxx, _CircleHasher.pxx, _EllipseHasher.pxx, _HyperbolaHasher.pxx, _ParabolaHasher.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the line and the conics: the placement hash combined with the rounded
//! radii (or focal length), and a placement comparison plus radii within tolerance.
//!
//! The native hashers take a handle and dereference it; these take the object.
use super::placement_hashers::{
    GeomHashAxisPlacement, GeomHashDirectionHasher, GeomHashPointHasher,
};
use crate::geom::{GeomCircle, GeomConic, GeomEllipse, GeomHyperbola, GeomLine, GeomParabola};
use rustycad_tkg2d::geom2d_hash::hash_key::{combined_hash, leading_and_rounded, values_within};
use rustycad_tkg2d::hasher_tolerances;
use rustycad_tkmath::gp::Axis2;

hasher_tolerances!(
    /// `GeomHash_LineHasher`: a line by location and direction;
    /// `GeomHashLineHasher::default().hash(&line)`.
    GeomHashLineHasher
);
hasher_tolerances!(
    /// `GeomHash_CircleHasher`: a circle by placement and radius;
    /// `GeomHashCircleHasher::default().hash(&circle)`.
    GeomHashCircleHasher
);
hasher_tolerances!(
    /// `GeomHash_EllipseHasher`: an ellipse by placement and both radii;
    /// `GeomHashEllipseHasher::default().hash(&ellipse)`.
    GeomHashEllipseHasher
);
hasher_tolerances!(
    /// `GeomHash_HyperbolaHasher`: a hyperbola by placement and both radii;
    /// `GeomHashHyperbolaHasher::default().hash(&hyperbola)`.
    GeomHashHyperbolaHasher
);
hasher_tolerances!(
    /// `GeomHash_ParabolaHasher`: a parabola by placement and focal length;
    /// `GeomHashParabolaHasher::default().hash(&parabola)`.
    GeomHashParabolaHasher
);

/// `[axis hash, round(value_i / HashTolerance)...]` combined, the shape of every conic
/// and elementary surface hash; `placement_and_values(axes, frame, &[radius])`.
pub(super) fn placement_and_values(
    axes: GeomHashAxisPlacement,
    frame: Axis2,
    values: &[f64],
) -> usize {
    leading_and_rounded(&[axes.hash(frame)], values, axes.hash_tolerance)
}
/// The placements equal, then each value pair within CompTolerance, in order;
/// `placement_and_values_equal(axes, [a, b], &[(r1, r2)])`.
pub(super) fn placement_and_values_equal(
    axes: GeomHashAxisPlacement,
    frames: [Axis2; 2],
    values: &[(f64, f64)],
) -> bool {
    axes.equal(frames[0], frames[1]) && values_within(axes.comp_tolerance, values)
}

impl GeomHashLineHasher {
    fn parts(&self) -> (GeomHashPointHasher, GeomHashDirectionHasher) {
        (
            GeomHashPointHasher::new(self.comp_tolerance, self.hash_tolerance),
            GeomHashDirectionHasher::new(self.comp_tolerance, self.hash_tolerance),
        )
    }
    /// Location and direction hashes combined (`operator()(handle)`); `hasher.hash(&line)`.
    pub fn hash(&self, line: &GeomLine) -> usize {
        let position = line.position();
        let (points, directions) = self.parts();
        combined_hash(&[
            points.hash(position.location()),
            directions.hash(position.direction()),
        ])
    }
    /// Locations and directions equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomLine, second: &GeomLine) -> bool {
        let (a, b) = (first.position(), second.position());
        let (points, directions) = self.parts();
        points.equal(a.location(), b.location()) && directions.equal(a.direction(), b.direction())
    }
}

impl GeomHashCircleHasher {
    fn axes(&self) -> GeomHashAxisPlacement {
        GeomHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded radius (`operator()(handle)`); `hasher.hash(&circle)`.
    pub fn hash(&self, circle: &GeomCircle) -> usize {
        placement_and_values(self.axes(), circle.position(), &[circle.radius()])
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomCircle, second: &GeomCircle) -> bool {
        let frames = [first.position(), second.position()];
        placement_and_values_equal(self.axes(), frames, &[(first.radius(), second.radius())])
    }
}

impl GeomHashEllipseHasher {
    fn axes(&self) -> GeomHashAxisPlacement {
        GeomHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded radii (`operator()(handle)`); `hasher.hash(&ellipse)`.
    pub fn hash(&self, ellipse: &GeomEllipse) -> usize {
        let radii = [ellipse.major_radius(), ellipse.minor_radius()];
        placement_and_values(self.axes(), ellipse.position(), &radii)
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomEllipse, second: &GeomEllipse) -> bool {
        let frames = [first.position(), second.position()];
        let radii = [
            (first.major_radius(), second.major_radius()),
            (first.minor_radius(), second.minor_radius()),
        ];
        placement_and_values_equal(self.axes(), frames, &radii)
    }
}

impl GeomHashHyperbolaHasher {
    fn axes(&self) -> GeomHashAxisPlacement {
        GeomHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded radii (`operator()(handle)`); `hasher.hash(&hyperbola)`.
    pub fn hash(&self, hyperbola: &GeomHyperbola) -> usize {
        let radii = [hyperbola.major_radius(), hyperbola.minor_radius()];
        placement_and_values(self.axes(), hyperbola.position(), &radii)
    }
    /// Placements and radii equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomHyperbola, second: &GeomHyperbola) -> bool {
        let frames = [first.position(), second.position()];
        let radii = [
            (first.major_radius(), second.major_radius()),
            (first.minor_radius(), second.minor_radius()),
        ];
        placement_and_values_equal(self.axes(), frames, &radii)
    }
}

impl GeomHashParabolaHasher {
    fn axes(&self) -> GeomHashAxisPlacement {
        GeomHashAxisPlacement::new(self.comp_tolerance, self.hash_tolerance)
    }
    /// Placement and rounded focal length (`operator()(handle)`); `hasher.hash(&parabola)`.
    pub fn hash(&self, parabola: &GeomParabola) -> usize {
        placement_and_values(self.axes(), parabola.position(), &[parabola.focal()])
    }
    /// Placements and focal lengths equal (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &GeomParabola, second: &GeomParabola) -> bool {
        let frames = [first.position(), second.position()];
        placement_and_values_equal(self.axes(), frames, &[(first.focal(), second.focal())])
    }
}
