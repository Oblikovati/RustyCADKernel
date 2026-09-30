// Rust translation of ModelingData/TKGeomBase/IntAna2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d: the analytic intersection of 2D lines, circles and conics.
//!
//! [`AnaIntersection`] holds the result of one intersection; each native
//! constructor is a named constructor (`lines`, `circles`, `line_circle`,
//! `line_conic`, ...) and each Perform overload a `perform_*` method, in the
//! module of its native source file (IntAna2d_AnaIntersection_1..8.cxx).
//! [`Conic`] is the implicit equation of the second curve and [`IntPoint`] one
//! result point. The public helpers of IntAna2d_Outils.hxx are
//! [`MyDirectPolynomialRoots`], [`points_confondus`],
//! [`traitement_points_confondus`] and [`coord_ancien_repere`].
mod ana_intersection;
mod circles;
mod conic;
mod int_point;
mod line_circle;
mod line_conic;
mod lines;
mod outils;
mod poly_conics;
mod trig_conics;

pub use ana_intersection::AnaIntersection;
pub use conic::Conic;
pub use int_point::IntPoint;
pub use outils::{
    MyDirectPolynomialRoots, coord_ancien_repere, points_confondus, traitement_points_confondus,
};
