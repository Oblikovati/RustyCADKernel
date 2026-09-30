// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_TangentialDeflection.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_TangentialDeflection`: points such that both the angle between consecutive
//! chords and the chordal deviation stay below given bounds.
mod estim_defl;
mod perform_curve;

use super::point_sequence::PointSequence;
use super::t_curve_types::live;
use super::{PoleCurve, TCurveTypes};
use crate::cpnts::PntsCurve;
use rustycad_tkernel::ncollection::ElementCast;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;
use std::f64::consts::FRAC_PI_2;

/// `Us3`: the native literal 0.3333333333333333333333333333 rounds to the same
/// double as the correctly rounded 1 / 3.
const US3: f64 = 1.0 / 3.0;

/// `GCPnts_TangentialDeflection`;
/// `TangentialDeflection::with_curve(curve, 0.1, 0.01, 2, 1.0e-9, 1.0e-7)?`.
///
/// The native defaults of the last three arguments are 2, 1.0e-9 and 1.0e-7; the port
/// takes them explicitly. The curve type `C` of the generic members is
/// `dyn Adaptor3dCurve`, `dyn Adaptor2dCurve2d` or a concrete adaptor.
#[derive(Default)]
pub struct TangentialDeflection {
    my_angular_deflection: f64,
    my_curvature_deflection: f64,
    my_u_tol: f64,
    my_min_nb_pnts: i32,
    my_min_len: f64,
    my_last_u: f64,
    my_firstu: f64,
    my_points: PointSequence,
}

/// The deflection criteria of a distribution: the angular deflection in radians, the
/// linear deflection, the minimum number of points, the parametric tolerance and the
/// minimal length (`theAngularDeflection` .. `theMinLen`).
#[derive(Clone, Copy, Debug)]
pub struct TangentialCriteria {
    /// `theAngularDeflection`.
    pub angular_deflection: f64,
    /// `theCurvatureDeflection`.
    pub curvature_deflection: f64,
    /// `theMinimumOfPoints` (native default 2).
    pub minimum_of_points: i32,
    /// `theUTol` (native default 1.0e-9).
    pub u_tol: f64,
    /// `theMinLen` (native default 1.0e-7).
    pub min_len: f64,
}

impl TangentialCriteria {
    /// The criteria with the native defaults of the last three arguments;
    /// `TangentialCriteria::new(0.1, 0.01)`.
    pub fn new(angular_deflection: f64, curvature_deflection: f64) -> Self {
        Self {
            angular_deflection,
            curvature_deflection,
            minimum_of_points: 2,
            u_tol: 1.0e-9,
            min_len: 1.0e-7,
        }
    }
}

impl TangentialDeflection {
    /// The empty algorithm (`GCPnts_TangentialDeflection()`);
    /// `TangentialDeflection::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The distribution on the whole curve (`GCPnts_TangentialDeflection(C,
    /// AngularDeflection, CurvatureDeflection, MinimumOfPoints, UTol, MinLen)`);
    /// `TangentialDeflection::with_curve(curve, TangentialCriteria::new(0.1, 0.01))?`.
    pub fn with_curve<C: TCurveTypes + ?Sized>(
        curve: &C,
        criteria: TangentialCriteria,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize(curve, criteria)?;
        Ok(result)
    }
    /// The distribution between `first` and `last` (`GCPnts_TangentialDeflection(C,
    /// FirstParameter, LastParameter, AngularDeflection, ...)`);
    /// `TangentialDeflection::with_range(curve, 0.0, 1.0, TangentialCriteria::new(0.1, 0.01))?`.
    pub fn with_range<C: TCurveTypes + ?Sized>(
        curve: &C,
        first: f64,
        last: f64,
        criteria: TangentialCriteria,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.initialize_range(curve, first, last, criteria)?;
        Ok(result)
    }
    /// Recompute on the whole curve (`Initialize(C, AngularDeflection, ...)`);
    /// `algo.initialize(curve, TangentialCriteria::new(0.1, 0.01))?`.
    pub fn initialize<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        criteria: TangentialCriteria,
    ) -> Result<(), Failure> {
        let pnts = curve.pnts_curve();
        let (first, last) = (pnts.first_parameter()?, pnts.last_parameter()?);
        self.initialize_range(curve, first, last, criteria)
    }
    /// Recompute between `first` and `last`, swapped when not increasing
    /// (`Initialize(C, FirstParameter, LastParameter, AngularDeflection, ...)`);
    /// Standard_ConstructionError for a linear deflection below Precision::Confusion()
    /// or an angular one below Precision::Angular(); at least 2 points and a minimal
    /// length of at least Precision::Confusion() are used;
    /// `algo.initialize_range(curve, 0.0, 1.0, TangentialCriteria::new(0.1, 0.01))?`.
    pub fn initialize_range<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        first: f64,
        last: f64,
        criteria: TangentialCriteria,
    ) -> Result<(), Failure> {
        if criteria.curvature_deflection < precision::confusion()
            || criteria.angular_deflection < precision::angular()
        {
            let message = format!(
                "GCPnts_TangentialDeflection::Initialize - Zero Deflection: angular {}, linear {}",
                criteria.angular_deflection, criteria.curvature_deflection
            );
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                &message,
                "",
            ));
        }
        self.my_points.clear();
        (self.my_firstu, self.my_last_u) = if first < last {
            (first, last)
        } else {
            (last, first)
        };
        self.my_u_tol = criteria.u_tol;
        self.my_angular_deflection = criteria.angular_deflection;
        self.my_curvature_deflection = criteria.curvature_deflection;
        self.my_min_nb_pnts = criteria.minimum_of_points.max(2);
        self.my_min_len = std_max(criteria.min_len, precision::confusion());
        self.dispatch(curve)
    }
    // Linear for lines and two-pole curves, circular for circles, curved otherwise.
    fn dispatch<C: TCurveTypes + ?Sized>(&mut self, curve: &C) -> Result<(), Failure> {
        let pnts: PntsCurve = curve.pnts_curve();
        let linear: bool = match pnts.get_type()? {
            CurveType::Line => true,
            CurveType::Circle => return self.perform_circular(pnts),
            CurveType::BSplineCurve => live(&curve.bspline_curve()?)?.pole_count() == 2,
            CurveType::BezierCurve => live(&curve.bezier_curve()?)?.pole_count() == 2,
            _ => false,
        };
        if linear {
            return self.perform_linear(pnts);
        }
        self.perform_curve(curve)
    }
}

impl TangentialDeflection {
    /// `PerformLinear`: the two ends and `MinimumOfPoints - 2` points at constant steps
    /// `(Last - First) / MinimumOfPoints` (so the last step is twice as long).
    fn perform_linear(&mut self, curve: PntsCurve<'_>) -> Result<(), Failure> {
        self.my_points
            .append(self.my_firstu, curve.lifted_d0(self.my_firstu)?);
        if self.my_min_nb_pnts > 2 {
            let du: f64 = (self.my_last_u - self.my_firstu) / f64::from(self.my_min_nb_pnts);
            let mut u: f64 = self.my_firstu + du;
            for _ in 2..self.my_min_nb_pnts {
                self.my_points.append(u, curve.lifted_d0(u)?);
                u += du;
            }
        }
        self.my_points
            .append(self.my_last_u, curve.lifted_d0(self.my_last_u)?);
        Ok(())
    }
    /// `PerformCircular`: constant steps of the arc angular step, their count rounded
    /// up (at most 1e6, at least MinimumOfPoints - 1), then the end point.
    fn perform_circular(&mut self, curve: PntsCurve<'_>) -> Result<(), Failure> {
        // akm 8/01/02 : check the radius before divide by it
        let radius: f64 = curve.circle_radius()?;
        let mut du: f64 = Self::arc_angular_step(
            radius,
            self.my_curvature_deflection,
            self.my_angular_deflection,
            self.my_min_len,
        )?;
        let diff: f64 = self.my_last_u - self.my_firstu;
        // Round up number of points to satisfy curvatureDeflection more precisely
        let mut nb_points: i32 = ElementCast::<i32>::cast(std_min((diff / du).ceil(), 1.0e6));
        nb_points = nb_points.max(self.my_min_nb_pnts - 1);
        du = diff / f64::from(nb_points);
        let mut u: f64 = self.my_firstu;
        for _ in 1..=nb_points {
            self.my_points.append(u, curve.lifted_d0(u)?);
            u += du;
        }
        self.my_points
            .append(self.my_last_u, curve.lifted_d0(self.my_last_u)?);
        Ok(())
    }
    /// `ArcAngularStep`: `2 acos(max(1 - LinearDeflection / Radius, 0))` bounded by the
    /// angular deflection and by the angle of the minimal length (at most a quarter
    /// turn); Standard_ConstructionError for a negative radius;
    /// `TangentialDeflection::arc_angular_step(1.0, 0.01, 0.1, 1e-7)?`.
    pub fn arc_angular_step(
        radius: f64,
        linear_deflection: f64,
        angular_deflection: f64,
        min_length: f64,
    ) -> Result<f64, Failure> {
        if radius < 0.0 {
            let message = format!("Negative radius {radius}, expected >= 0");
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                &message,
                "",
            ));
        }
        let a_precision: f64 = precision::confusion();
        let (mut du, mut min_size_ang) = (0.0, 0.0);
        if radius > a_precision {
            du = std_max(1.0 - (linear_deflection / radius), 0.0);
            // It is not suitable to consider min size greater than 1/4 arc len.
            if min_length > a_precision {
                min_size_ang = std_min(min_length / radius, FRAC_PI_2);
            }
        }
        du = 2.0 * f64::acos(du);
        Ok(std_max(std_min(du, angular_deflection), min_size_ang))
    }
}

impl TangentialDeflection {
    /// Insert a point in parameter order, or replace (when `is_replace`) the point
    /// within Precision::PConfusion() of `param`; returns its index (`AddPoint`);
    /// `algo.add_point(point, 0.5, true)?`.
    pub fn add_point(&mut self, pnt: Point3, param: f64, is_replace: bool) -> Result<i32, Failure> {
        let tol: f64 = precision::p_confusion();
        for i in 1..=self.my_points.length() {
            let dist: f64 = self.my_points.parameter(i)? - param;
            if dist.abs() <= tol {
                if is_replace {
                    self.my_points.set(i, param, pnt)?;
                }
                return Ok(i);
            }
            if dist > tol {
                self.my_points.insert_before(i, param, pnt)?;
                return Ok(i);
            }
        }
        self.my_points.append(param, pnt);
        Ok(self.my_points.length())
    }
    /// The number of points (`NbPoints`); `algo.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.my_points.length()
    }
    /// The parameter of index `i`, from 1 (`Parameter`); Standard_OutOfRange outside
    /// `[1, NbPoints]`; `algo.parameter(1)?`.
    pub fn parameter(&self, i: i32) -> Result<f64, Failure> {
        self.my_points.parameter(i)
    }
    /// The point of index `i`, 2D points in the plane Z = 0 (`Value`);
    /// Standard_OutOfRange outside `[1, NbPoints]`; `algo.value(1)?`.
    pub fn value(&self, i: i32) -> Result<Point3, Failure> {
        self.my_points.point(i)
    }
}
