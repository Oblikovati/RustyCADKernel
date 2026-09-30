// Rust translation of ModelingData/TKG3d/GeomGridEval (GeomGridEval.hxx and the package) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GeomGridEval package: batch evaluation of 3D curves and surfaces on parameter
//! grids.
//!
//! A curve evaluator takes an array of parameters (any bounds) and returns an array
//! indexed from 1; a surface evaluator takes a U and a V array and returns a grid of
//! bounds `1..NbU x 1..NbV`. Empty parameters, a null geometry or an invalid derivative
//! order give an empty result. `GeomGridEvalCurve` and `GeomGridEvalSurface` pick the
//! specialized evaluator from the geometry class or the adaptor. Evaluators that can
//! raise natively (through the gp getters, BSplCLib/BSplSLib, the offset utilities or
//! an adaptor) return `Result`. The TKG2d parameter loops (`grid_loops`) are shared.
use crate::geom::{
    GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};

/// Point and first derivative of a curve grid (`GeomGridEval::CurveD1`); the same
/// fields as `Geom_Curve::ResD1`, so the port shares the struct.
pub type GeomGridEvalCurveD1 = GeomCurveD1;
/// Point and two derivatives of a curve grid (`GeomGridEval::CurveD2`).
pub type GeomGridEvalCurveD2 = GeomCurveD2;
/// Point and three derivatives of a curve grid (`GeomGridEval::CurveD3`).
pub type GeomGridEvalCurveD3 = GeomCurveD3;
/// Point and first partial derivatives of a surface grid (`GeomGridEval::SurfD1`).
pub type GeomGridEvalSurfD1 = GeomSurfaceD1;
/// Point and partial derivatives up to order 2 (`GeomGridEval::SurfD2`).
pub type GeomGridEvalSurfD2 = GeomSurfaceD2;
/// Point and partial derivatives up to order 3 (`GeomGridEval::SurfD3`).
pub type GeomGridEvalSurfD3 = GeomSurfaceD3;

mod helpers;
pub use helpers::{
    evaluate_grid_d1_helper, evaluate_grid_d2_helper, evaluate_grid_d3_helper,
    evaluate_grid_dn_helper, evaluate_grid_helper,
};
mod surface_grid;

mod conic;
mod frame;
mod line;
pub use line::GeomGridEvalLine;
mod circle;
pub use circle::GeomGridEvalCircle;
mod ellipse;
pub use ellipse::GeomGridEvalEllipse;
mod hyperbola;
pub use hyperbola::GeomGridEvalHyperbola;
mod parabola;
pub use parabola::GeomGridEvalParabola;
mod bezier_curve;
pub use bezier_curve::GeomGridEvalBezierCurve;
mod bspline_curve;
mod spans;
pub use bspline_curve::GeomGridEvalBSplineCurve;
mod offset_curve;
pub use offset_curve::GeomGridEvalOffsetCurve;
mod other_curve;
pub use other_curve::GeomGridEvalOtherCurve;
mod curve;
pub use curve::{GeomGridEvalCurve, GeomGridEvalCurveEvaluator};

mod plane;
pub use plane::GeomGridEvalPlane;
mod cylinder;
mod quadric;
pub use cylinder::GeomGridEvalCylinder;
mod sphere;
pub use sphere::GeomGridEvalSphere;
mod cone;
pub use cone::GeomGridEvalCone;
mod torus;
pub use torus::GeomGridEvalTorus;
mod bezier_surface;
pub use bezier_surface::GeomGridEvalBezierSurface;
mod bspline_surface;
mod bspline_surface_loops;
pub use bspline_surface::GeomGridEvalBSplineSurface;
mod offset_surface;
pub use offset_surface::GeomGridEvalOffsetSurface;
mod other_surface;
pub use other_surface::{GeomGridEvalOtherSurface, GeomGridEvalSurfaceSource};
mod surface_of_revolution;
pub use surface_of_revolution::GeomGridEvalSurfaceOfRevolution;
mod surface_of_extrusion;
pub use surface_of_extrusion::GeomGridEvalSurfaceOfExtrusion;
mod surface;
mod surface_adaptor;
mod surface_transform;
pub use surface::{GeomGridEvalSurface, GeomGridEvalSurfaceEvaluator};
