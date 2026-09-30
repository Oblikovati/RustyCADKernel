// Rust translation of ModelingData/TKGeomBase/ProjLib (package overview)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! ProjLib: the projection of curves onto surfaces.
//!
//! The static `ProjLib::Project` overloads and the elementary projectors
//! ([`Plane`], [`Cylinder`], [`Cone`], [`Sphere`], [`Torus`], all derived from
//! [`Projector`] and driven through the [`Project`] trait) give the exact 2D image
//! of lines and conics on the elementary surfaces. [`PrjFunc`] and [`PrjResolve`]
//! solve the orthogonal projection of one curve point.
mod comp_projected;
mod compute_approx;
mod cone;
mod cylinder;
mod plane;
mod polar;
mod prj_func;
mod prj_resolve;
mod project_on_plane;
mod projected_curve;
mod projector;
mod sphere;
mod statics;
mod torus;

pub use comp_projected::{CompProjectedCurve, HCompProjectedCurve, SequenceOfSequenceOfPnt};
pub use compute_approx::ComputeApprox;
pub use cone::Cone;
pub use cylinder::Cylinder;
pub use plane::Plane;
pub use polar::ComputeApproxOnPolarSurface;
pub use prj_func::PrjFunc;
pub use prj_resolve::{PrjResolve, PrjResolveBox};
pub use project_on_plane::ProjectOnPlane;
pub use projected_curve::ProjectedCurve;
pub use projector::{Project, Projector};
pub use sphere::Sphere;
pub use statics::*;
pub use torus::Torus;

pub(crate) use project_on_plane::approx::new_array;
