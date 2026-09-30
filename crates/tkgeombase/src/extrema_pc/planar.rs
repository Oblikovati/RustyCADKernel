// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Planar.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_Planar`: solving a planar 3D curve with the 2D evaluator in its plane.
//! The projections keep the curve parameterization, so the 2D parameters are the 3D
//! ones and only the points and distances are re-evaluated in 3D.
use super::{Domain1D, ExtremaResult, ExtremumResult, SearchMode, Status};
use crate::extrema_pc_core::CurveEvaluator;
use crate::extrema_pc2d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis3, Direction3, Line3, Plane3, Point2, Point3};

mod projection;
pub use projection::{project_bezier, project_bspline, project_offset};

/// `ToSearchMode2d`: the 2D search mode of a 3D one. Both packages share one Rust
/// enum, so it is the identity; `to_search_mode_2d(SearchMode::Min) == SearchMode::Min`.
pub fn to_search_mode_2d(mode: SearchMode) -> SearchMode {
    mode
}

/// `LinePlane`: a deterministic plane containing the line, whose X direction is the line
/// direction (so the line parameter is kept); the normal is `D x Z`, or `D x Y` when
/// `|D.Z| >= 0.9`; `line_plane(&line)?`.
pub fn line_plane(line: &Line3) -> Result<Plane3, Failure> {
    let direction: Direction3 = line.direction();
    let reference: Direction3 = if direction.z().abs() < 0.9 {
        Direction3::Z
    } else {
        Direction3::Y
    };
    let normal: Direction3 = Direction3::from_xyz(direction.xyz().crossed(reference.xyz()))?;
    Ok(Plane3::from_position(Axis3::new(
        line.location(),
        normal,
        direction,
    )?))
}

/// `ConvertPlanarResult`: the 3D result of a 2D one, re-evaluating each extremum on the
/// 3D evaluator at its parameter. Returns false (leaving `result` untouched) for a 2D
/// NotDone or NumericalError, which the caller answers with the 3D algorithm;
/// `convert_planar_result(&result2d, &point, &plane, &evaluator, &mut result)?`.
pub fn convert_planar_result<E: CurveEvaluator<Point3> + ?Sized>(
    result2d: &extrema_pc2d::ExtremaResult,
    query: &Point3,
    plane: &Plane3,
    evaluator: &E,
    result: &mut ExtremaResult,
) -> Result<bool, E::Error> {
    if result2d.status == Status::NotDone || result2d.status == Status::NumericalError {
        return Ok(false);
    }
    result.clear();
    result.status = result2d.status;
    if result2d.is_infinite() {
        let height: f64 = plane.distance(*query);
        result.infinite_square_distance = result2d.infinite_square_distance + height * height;
        return Ok(true);
    }
    for extremum in &result2d.extrema {
        let curve_point: Point3 = evaluator.evaluate(extremum.parameter)?;
        result.extrema.push(ExtremumResult::new(
            extremum.parameter,
            curve_point,
            query.square_distance(&curve_point),
            extremum.is_minimum,
            extremum.is_maximum,
        ));
    }
    Ok(true)
}

/// A 2D analytical evaluator `PerformPlanar` can run (its `Evaluator2d` parameter).
pub trait PlanarEvaluator2d: Sized {
    /// The 2D geometry of the constructors (`Geometry2d`).
    type Geometry;
    /// The constructor without a domain.
    fn whole(geometry: Self::Geometry) -> Self;
    /// The constructor with a domain.
    fn bounded(geometry: Self::Geometry, domain: Domain1D) -> Self;
    /// `Perform` or `PerformWithEndpoints`.
    fn run(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
        with_endpoints: bool,
    ) -> &extrema_pc2d::ExtremaResult;
}

/// The query of `PerformPlanar`: the projected and original points, the plane, the
/// tolerance, the mode and whether the endpoints are included.
pub struct PlanarQuery<'p> {
    /// The query projected on the plane.
    pub point2d: Point2,
    /// The original query.
    pub point3d: Point3,
    /// The projection plane.
    pub plane: &'p Plane3,
    /// The geometric tolerance.
    pub tol: f64,
    /// The search mode.
    pub mode: SearchMode,
    /// Whether the endpoints are included.
    pub with_endpoints: bool,
}

/// `PerformPlanar`: runs the 2D evaluator on the projected geometry and converts its
/// result; false when the 3D algorithm must be used instead;
/// `perform_planar::<extrema_pc2d::Circle, _>(circle2d, domain, &query, &evaluator, &mut result)?`.
pub fn perform_planar<E2: PlanarEvaluator2d, E3: CurveEvaluator<Point3> + ?Sized>(
    geometry: E2::Geometry,
    domain: Option<Domain1D>,
    query: &PlanarQuery<'_>,
    evaluator: &E3,
    result: &mut ExtremaResult,
) -> Result<bool, E3::Error> {
    let mode: SearchMode = to_search_mode_2d(query.mode);
    let mut evaluator2d: E2 = match domain {
        Some(domain) => E2::bounded(geometry, domain),
        None => E2::whole(geometry),
    };
    let result2d = evaluator2d.run(query.point2d, query.tol, mode, query.with_endpoints);
    convert_planar_result(result2d, &query.point3d, query.plane, evaluator, result)
}

// The five analytical 2D evaluators as `Evaluator2d`.
macro_rules! planar_evaluator {
    ($evaluator:ty, $geometry:ty) => {
        impl PlanarEvaluator2d for $evaluator {
            type Geometry = $geometry;
            fn whole(geometry: $geometry) -> Self {
                <$evaluator>::new(geometry)
            }
            fn bounded(geometry: $geometry, domain: Domain1D) -> Self {
                <$evaluator>::with_domain(geometry, domain)
            }
            fn run(
                &mut self,
                point: Point2,
                tol: f64,
                mode: SearchMode,
                with_endpoints: bool,
            ) -> &extrema_pc2d::ExtremaResult {
                if with_endpoints {
                    self.perform_with_endpoints(point, tol, mode)
                } else {
                    self.perform(point, tol, mode)
                }
            }
        }
    };
}

planar_evaluator!(extrema_pc2d::Line, rustycad_tkmath::gp::Line2);
planar_evaluator!(extrema_pc2d::Circle, rustycad_tkmath::gp::Circle2);
planar_evaluator!(extrema_pc2d::Ellipse, rustycad_tkmath::gp::Ellipse2);
planar_evaluator!(extrema_pc2d::Hyperbola, rustycad_tkmath::gp::Hyperbola2);
planar_evaluator!(extrema_pc2d::Parabola, rustycad_tkmath::gp::Parabola2);
