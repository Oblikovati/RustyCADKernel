// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_CurveComputation.pxx (BSplCLib_CurveTraits) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The point/vector pair of a 2D or 3D curve (`gp_Pnt2d`/`gp_Vec2d`, `gp_Pnt`/`gp_Vec`).
use crate::gp::{Point2, Point3, Vector2, Vector3};
use crate::plib::PolePoint;

/// A point with its first three derivative vectors, as D3 returns them;
/// `let (p, d1, d2, d3): CurveD3<Point3> = bsplclib::d3(&span, &poles, None)?`.
pub type CurveD3<P> = (
    P,
    <P as CurvePoint>::Vector,
    <P as CurvePoint>::Vector,
    <P as CurvePoint>::Vector,
);

/// A curve point type and its derivative vector type; implemented for [`Point2`] and
/// [`Point3`]; `<Point3 as CurvePoint>::vector_from(&[1.0, 2.0, 3.0])`.
pub trait CurvePoint: PolePoint {
    /// The derivative vector (`gp_Vec2d` or `gp_Vec`).
    type Vector: Copy + Default + std::fmt::Debug + PartialEq;
    /// Builds a vector from its leading coordinates; `Point2::vector_from(&[1.0, 2.0])`.
    fn vector_from(coordinates: &[f64]) -> Self::Vector;
    /// Reads a zero-based vector coordinate; `Point3::vector_coordinate(&v, 0)`.
    fn vector_coordinate(vector: &Self::Vector, index: usize) -> f64;
    /// Builds a point from its leading coordinates; `Point3::point_from(&[0.0, 0.0, 0.0])`.
    fn point_from(coordinates: &[f64]) -> Self {
        let mut point: Self = Self::default();
        for (k, value) in coordinates.iter().take(Self::DIMENSION).enumerate() {
            point.set_pole_coordinate(k, *value);
        }
        point
    }
}

impl CurvePoint for Point3 {
    type Vector = Vector3;
    fn vector_from(coordinates: &[f64]) -> Vector3 {
        Vector3::new(coordinates[0], coordinates[1], coordinates[2])
    }
    fn vector_coordinate(vector: &Vector3, index: usize) -> f64 {
        [vector.x(), vector.y(), vector.z()][index]
    }
}

impl CurvePoint for Point2 {
    type Vector = Vector2;
    fn vector_from(coordinates: &[f64]) -> Vector2 {
        Vector2::new(coordinates[0], coordinates[1])
    }
    fn vector_coordinate(vector: &Vector2, index: usize) -> f64 {
        [vector.x(), vector.y()][index]
    }
}
