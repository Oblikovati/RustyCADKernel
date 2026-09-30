// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package GeomPlate: plate surfaces that meet curve and point constraints
//! (`GeomPlate_BuildPlateSurface`), their surface (`GeomPlate_Surface`), constraints
//! (`GeomPlate_CurveConstraint`, `GeomPlate_PointConstraint`), their B-spline
//! approximation (`GeomPlate_MakeApprox` with the `GeomPlate_PlateG0Criterion` and
//! `GeomPlate_PlateG1Criterion` criteria) and the average plane of a point cloud
//! (`GeomPlate_BuildAveragePlane`, `GeomPlate_Aij`).
//!
//! Each class is one file (two for the larger ones); docs/geomalgo-design.md section
//! 19 fixes how the shared constraints, the Geom surface and the approximation
//! callbacks map.
mod aij;
mod average_plane_normals;
mod build_average_plane;
mod build_plate_surface;
mod criterion_patch;
mod curve_constraint;
mod curve_constraint_eval;
mod half_space;
mod make_approx;
mod make_approx_eval;
mod plate_criteria;
mod point_constraint;
mod surface;

pub use aij::GeomPlateAij;
pub use build_average_plane::GeomPlateBuildAveragePlane;
pub use build_plate_surface::GeomPlateBuildPlateSurface;
pub use curve_constraint::GeomPlateCurveConstraint;
pub use make_approx::GeomPlateMakeApprox;
pub use plate_criteria::{GeomPlatePlateG0Criterion, GeomPlatePlateG1Criterion};
pub use point_constraint::GeomPlatePointConstraint;
pub use surface::GeomPlateSurface;

use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::errors;

/// The object of `handle`, Standard_NullObject naming `what` for a null handle (a
/// null dereference natively).
pub(crate) fn live<'h, T: ?Sized + RuntimeTyped>(
    handle: &'h Handle<T>,
    what: &str,
) -> Result<&'h T, Failure> {
    handle.get().ok_or_else(|| {
        errors::null_object(&format!(
            "GeomPlate: {what} is null; expected a constructed object"
        ))
    })
}
