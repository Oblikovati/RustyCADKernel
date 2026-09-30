// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_CurveConstraint.cxx
// (the parameters, evaluations, criteria and 2d curves) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluations of `GeomPlate_CurveConstraint`.
use super::curve_constraint::{GeomPlateCurveConstraint, not_on_surface};
use super::live;
use crate::law::LawFunction;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkg3d::geom::{GeomSurfaceD1, GeomSurfaceD2};
use rustycad_tkgeombase::approx::Curve2d as ApproxCurve2d;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};
use std::sync::RwLockWriteGuard;

/// The curve a parameter member reads, in the native order of the tests.
enum ParameterSource {
    Projected(Handle<dyn Adaptor2dCurve2d>),
    Frontiere(Handle<Adaptor3dCurveOnSurface>),
    Curve3d(Handle<dyn Adaptor3dCurve>),
}

impl GeomPlateCurveConstraint {
    fn parameter_source(&self) -> ParameterSource {
        let state = self.state.read();
        if state.hcurve_2d.is_some() {
            return ParameterSource::Projected(state.hcurve_2d.clone());
        }
        if state.curve_3d.is_null() {
            return ParameterSource::Frontiere(state.frontiere.clone());
        }
        ParameterSource::Curve3d(state.curve_3d.clone())
    }

    /// The first parameter: of the projected curve when set, else of the boundary
    /// (`FirstParameter`); `c.first_parameter()?`.
    pub fn first_parameter(&self) -> Result<f64, Failure> {
        match self.parameter_source() {
            ParameterSource::Projected(c) => live(&c, "the projected curve")?.first_parameter(),
            ParameterSource::Frontiere(c) => live(&c, "the boundary")?.first_parameter(),
            ParameterSource::Curve3d(c) => live(&c, "the 3d curve")?.first_parameter(),
        }
    }

    /// The last parameter, as `first_parameter` (`LastParameter`); `c.last_parameter()?`.
    pub fn last_parameter(&self) -> Result<f64, Failure> {
        match self.parameter_source() {
            ParameterSource::Projected(c) => live(&c, "the projected curve")?.last_parameter(),
            ParameterSource::Frontiere(c) => live(&c, "the boundary")?.last_parameter(),
            ParameterSource::Curve3d(c) => live(&c, "the 3d curve")?.last_parameter(),
        }
    }

    /// The length of the boundary (`Length`, `GCPnts_AbscissaPoint::Length`);
    /// `c.length()?`.
    pub fn length(&self) -> Result<f64, Failure> {
        let (frontiere, curve_3d) = self.boundaries();
        if curve_3d.is_null() {
            let curve: &dyn Adaptor3dCurve = live(&frontiere, "the boundary")?;
            return AbscissaPoint::length(curve);
        }
        AbscissaPoint::length(live(&curve_3d, "the 3d curve")?)
    }

    fn boundaries(&self) -> (Handle<Adaptor3dCurveOnSurface>, Handle<dyn Adaptor3dCurve>) {
        let state = self.state.read();
        (state.frontiere.clone(), state.curve_3d.clone())
    }

    /// The point of the curve on surface at `u`: its 2d point evaluated on the surface.
    fn frontiere_uv(&self, u: f64) -> Result<(Handle<Adaptor3dCurveOnSurface>, Point2), Failure> {
        let frontiere = self.state.read().frontiere.clone();
        let curve = live(&frontiere, "the boundary")?.get_curve();
        let p2d: Point2 = live(&curve, "the 2d curve of the boundary")?.value(u)?;
        Ok((frontiere, p2d))
    }

    /// The point at `u` (`D0`); `c.d0(0.5)?`.
    pub fn d0(&self, u: f64) -> Result<Point3, Failure> {
        let curve_3d = self.state.read().curve_3d.clone();
        if let Some(curve) = curve_3d.get() {
            return curve.d0(u);
        }
        let (frontiere, p2d) = self.frontiere_uv(u)?;
        let surface = live(&frontiere, "the boundary")?.get_surface();
        live(&surface, "the surface of the boundary")?.d0(p2d.x(), p2d.y())
    }

    /// The point and the first derivatives of the surface at the point of parameter
    /// `u` (`D1`); Standard_Failure for a constraint without surface. `c.d1(0.5)?.d1u`.
    pub fn d1(&self, u: f64) -> Result<GeomSurfaceD1, Failure> {
        if self.state.read().curve_3d.is_some() {
            return Err(not_on_surface());
        }
        let (frontiere, p2d) = self.frontiere_uv(u)?;
        let surface = live(&frontiere, "the boundary")?.get_surface();
        live(&surface, "the surface of the boundary")?.d1(p2d.x(), p2d.y())
    }

    /// The point and the surface derivatives up to order 2 (`D2`); Standard_Failure
    /// for a constraint without surface. `c.d2(0.5)?.d2uv`.
    pub fn d2(&self, u: f64) -> Result<GeomSurfaceD2, Failure> {
        if self.state.read().curve_3d.is_some() {
            return Err(not_on_surface());
        }
        let (frontiere, p2d) = self.frontiere_uv(u)?;
        let surface = live(&frontiere, "the boundary")?.get_surface();
        live(&surface, "the surface of the boundary")?.d2(p2d.x(), p2d.y())
    }

    /// The distance criterion at `u`: the tolerance, or the law once set
    /// (`G0Criterion`); `c.g0_criterion(0.5)?`.
    pub fn g0_criterion(&self, u: f64) -> Result<f64, Failure> {
        let (constant, tol, law) = {
            let state = self.state.read();
            (state.const_g0, state.tol_dist, state.g0_crit.clone())
        };
        criterion(constant, tol, &law, u)
    }

    /// The angular criterion at `u` (`G1Criterion`); Standard_Failure for a
    /// constraint without surface. `c.g1_criterion(0.5)?`.
    pub fn g1_criterion(&self, u: f64) -> Result<f64, Failure> {
        let (curve_3d, constant, tol, law) = {
            let state = self.state.read();
            let law = state.g1_crit.clone();
            (state.curve_3d.is_some(), state.const_g1, state.tol_ang, law)
        };
        if curve_3d {
            return Err(not_on_surface());
        }
        criterion(constant, tol, &law, u)
    }

    /// The curvature criterion at `u` (`G2Criterion`); Standard_Failure for a
    /// constraint without surface. `c.g2_criterion(0.5)?`.
    pub fn g2_criterion(&self, u: f64) -> Result<f64, Failure> {
        let (curve_3d, constant, tol, law) = {
            let state = self.state.read();
            let law = state.g2_crit.clone();
            (
                state.curve_3d.is_some(),
                state.const_g2,
                state.tol_curv,
                law,
            )
        };
        if curve_3d {
            return Err(not_on_surface());
        }
        criterion(constant, tol, &law, u)
    }

    /// The 2d curve on the initial surface: the stored one, or, when only the
    /// projected curve is set, its C1 approximation of degree 10 at most on
    /// `20 + NbIntervals(C3)` segments within the parametric tolerances, computed at
    /// each call (`Curve2dOnSurf`); `c.curve2d_on_surf()?`.
    pub fn curve2d_on_surf(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let (curve_2d, hcurve_2d, tol_u, tol_v) = {
            let state = self.state.read();
            let (c2, h2) = (state.curve_2d.clone(), state.hcurve_2d.clone());
            (c2, h2, state.tol_u, state.tol_v)
        };
        let Some(projected) = hcurve_2d.get().filter(|_| curve_2d.is_null()) else {
            return Ok(curve_2d);
        };
        let max_seg: i32 = 20 + projected.nb_intervals(Shape::C3)?;
        let (first, last) = (projected.first_parameter()?, projected.last_parameter()?);
        let appr = ApproxCurve2d::new(
            &hcurve_2d,
            first,
            last,
            tol_u,
            tol_v,
            Shape::C1,
            10,
            max_seg,
        )?;
        Ok(appr.curve().cast())
    }

    /// The local properties of the surface at the point of parameter `u` of the
    /// curve on surface (`LPropSurf`), returned for the caller to update;
    /// Standard_Failure for a constraint without surface. `c.lprop_surf(0.5)?.value()`.
    pub fn lprop_surf(&self, u: f64) -> Result<RwLockWriteGuard<'_, SLProps>, Failure> {
        if self.state.read().frontiere.is_null() {
            return Err(not_on_surface());
        }
        let (_, p2d) = self.frontiere_uv(u)?;
        let mut lprop = self.lprop.write();
        lprop.set_parameters(p2d.x(), p2d.y())?;
        Ok(lprop)
    }
}

/// `myConstGi ? myTol : myGiCrit->Value(U)`; Standard_NullObject for a null law.
fn criterion(
    constant: bool,
    tol: f64,
    law: &Handle<dyn LawFunction>,
    u: f64,
) -> Result<f64, Failure> {
    if constant {
        return Ok(tol);
    }
    live(law, "the criterion law")?.value(u)
}
