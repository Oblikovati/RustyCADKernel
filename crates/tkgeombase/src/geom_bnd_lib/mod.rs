// Rust translation of ModelingData/TKGeomBase/GeomBndLib at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomBndLib: bounding boxes of Geom and Geom2d curves and surfaces.
//!
//! One evaluator per geometry kind (`GeomBndLib_Line` ... `GeomBndLib_OtherSurface`)
//! computes `Box` (a fast, conservative box) and `BoxOptimal` (a tight box, sampled and
//! refined by numerical optimization for non-analytic geometry). The dispatchers
//! [`Curve`], [`Curve2d`] and [`Surface`] detect the kind of a Geom handle or adaptor and
//! delegate. Each native overload is a named Rust method: `Box(Tol)` is
//! `bounding_box`, `Box(U1, U2, Tol)` `bounding_box_range`, `BoxOptimal` the `_optimal`
//! variants, `Add`/`AddOptimal` `add`/`add_range`/`add_optimal`/`add_optimal_range`.
//! Every method returns `Result`: the native code raises Standard_Failure for invalid
//! infinite ranges and lets geometry exceptions propagate.
//!
//! The private `.pxx` helpers are modules too: [`sampling`] (whose functions the
//! upstream tests call), [`infinite`], [`optimization`] and [`spline`].
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, exceptions};

mod circle;
mod curve;
mod curve2d;
mod cylinder;
mod ellipse;
mod elliptic_arc;
mod extrusion;
mod hyperbola;
pub mod infinite;
mod line;
mod offset_curve;
mod offset_curve2d;
mod offset_surface;
pub mod optimization;
mod other_curve;
mod other_surface;
mod parabola;
pub mod patch;
mod plane;
pub mod ranges;
mod revolution;
pub mod sampling;
pub mod space;
mod sphere;
pub mod spline;
mod spline_curve;
mod spline_curve2d;
mod spline_surface;
mod surface;
mod surface_coord;
pub mod surface_sampling;
pub mod swept;

pub use circle::{Circle, Circle2d};
pub use curve::Curve;
pub use curve2d::Curve2d;
pub use cylinder::{Cone, Cylinder};
pub use ellipse::{Ellipse, Ellipse2d};
pub use elliptic_arc::{ArcTangent, EllipticArc};
pub use extrusion::SurfaceOfExtrusion;
pub use hyperbola::{Hyperbola, Hyperbola2d, HyperbolaArc};
pub use line::{Line, Line2d};
pub use offset_curve::OffsetCurve;
pub use offset_curve2d::OffsetCurve2d;
pub use offset_surface::OffsetSurface;
pub use other_curve::{OtherCurve, OtherCurve2d};
pub use other_surface::OtherSurface;
pub use parabola::{Parabola, Parabola2d, add_parabola_range};
pub use plane::Plane;
pub use revolution::SurfaceOfRevolution;
pub use sphere::{Sphere, Torus, sphere_extents, sphere_patch};
pub use spline_curve::{BSplineCurve, BezierCurve};
pub use spline_curve2d::{BSplineCurve2d, BezierCurve2d};
pub use spline_surface::{BSplineSurface, BezierSurface};
pub use surface::Surface;
pub use surface_coord::SurfCoordFunc;

/// The object of a handle the native code dereferences without a check; a null handle
/// (a native crash) is a Standard_NullObject naming `owner`; `live(&handle, "Owner")?`.
pub(crate) fn live<'h, T: ?Sized + RuntimeTyped>(
    handle: &'h Handle<T>,
    owner: &str,
) -> Result<&'h T, Failure> {
    handle.get().ok_or_else(|| {
        exceptions::null_object(&format!(
            "{owner}: null geometry handle, expected a live object"
        ))
    })
}
