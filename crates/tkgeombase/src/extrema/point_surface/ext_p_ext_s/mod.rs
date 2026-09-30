// Rust translation of Extrema_ExtPExtS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtPExtS`: the extrema of the distance between a point and a surface of
//! linear extrusion: from the extrema to the conic basis curve in its plane when the
//! extrusion direction is not parallel to that plane, otherwise by Extrema_GenExtPS
//! on a 32 x 32 grid.
mod analytic;

use super::func_ps_norm::FuncPSNorm;
use super::gen_ext_ps::{GenExtPS, Sampling};
use crate::extrema::ExtPElC;
use crate::extrema::failures::{extremum_index, not_done, null_object};
use crate::extrema::pon::POnSurf;
use crate::extrema::surface_ref::SurfaceRef;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurfaceOfLinearExtrusion;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Axis2, Direction3, Plane3, Point3, Vector3};
use std::f64::consts::PI;

/// `NbExtMax`: the dimension of the analytical result arrays.
const NB_EXT_MAX: i32 = 4;

/// `Extrema_ExtPExtS`; `ExtPExtS::with_surface(point, &extrusion, 1e-9, 1e-9)?.nb_ext()?`.
pub struct ExtPExtS {
    uinf: f64,
    usup: f64,
    tolu: f64,
    vinf: f64,
    vsup: f64,
    tolv: f64,
    f: FuncPSNorm<'static>,
    c: Handle<dyn Adaptor3dCurve>,
    s: Handle<GeomAdaptorSurfaceOfLinearExtrusion>,
    direction: Vector3,
    position: Axis2,
    ext_ps: GenExtPS<'static>,
    is_analytically_computable: bool,
    done: bool,
    nb_ext: i32,
    sq_dist: [f64; 4],
    points: [POnSurf; 4],
}

impl Default for ExtPExtS {
    fn default() -> Self {
        Self {
            uinf: 0.0,
            usup: 0.0,
            tolu: 0.0,
            vinf: 0.0,
            vsup: 0.0,
            tolv: 0.0,
            f: FuncPSNorm::new(),
            c: Handle::default(),
            s: Handle::default(),
            direction: Vector3::default(),
            position: Axis2::default(),
            ext_ps: GenExtPS::new(),
            is_analytically_computable: false,
            done: false,
            nb_ext: 0,
            sq_dist: [f64::MAX; 4],
            points: [POnSurf::default(); 4],
        }
    }
}

// `ProjectPnt`: the projection of `point` on the plane `plane` along `dir`.
fn project_pnt(plane: &Axis2, dir: Direction3, point: Point3) -> Point3 {
    let po: Vector3 = Vector3::between(point, plane.location());
    let mut alpha: f64 = po.dot(Vector3::from(plane.direction()));
    alpha /= dir.dot(plane.direction());
    let mut projected: Point3 = Point3::default();
    projected.set_xyz(point.xyz() + alpha * dir.xyz());
    projected
}

// `IsOriginalPnt`: no kept point is within Precision::Confusion() of `point`.
fn is_original_pnt(point: &Point3, points: &[POnSurf]) -> bool {
    !points
        .iter()
        .any(|kept| kept.value().is_equal(point, precision::confusion()))
}

// `GetPosition`: the frame of the conic, a frame normal to a line, or the default.
fn get_position(curve: &dyn Adaptor3dCurve) -> Result<Axis2, Failure> {
    Ok(match curve.get_type()? {
        CurveType::Line => {
            let line = curve.line()?;
            let pln: Plane3 = Plane3::new(line.location(), line.direction())?;
            Axis2::with_x_direction(
                pln.location(),
                pln.position().direction(),
                pln.position().x_direction(),
            )?
        }
        CurveType::Circle => curve.circle()?.position(),
        CurveType::Ellipse => curve.ellipse()?.position(),
        CurveType::Hyperbola => curve.hyperbola()?.position(),
        CurveType::Parabola => curve.parabola()?.position(),
        _ => Axis2::default(),
    })
}

// `PerformExtPElC`: the point-conic extrema over the whole conic.
fn perform_ext_p_el_c(
    extrema: &mut ExtPElC,
    point: Point3,
    curve: &dyn Adaptor3dCurve,
    tol: f64,
) -> Result<(), Failure> {
    let inf: f64 = precision::infinite();
    match curve.get_type()? {
        CurveType::Hyperbola => {
            extrema.perform_hyperbola(point, &curve.hyperbola()?, tol, -inf, inf)
        }
        CurveType::Line => {
            extrema.perform_line(point, &curve.line()?, tol, -inf, inf);
            Ok(())
        }
        CurveType::Circle => extrema.perform_circle(point, &curve.circle()?, tol, 0.0, 2.0 * PI),
        CurveType::Ellipse => extrema.perform_ellipse(point, &curve.ellipse()?, tol, 0.0, 2.0 * PI),
        CurveType::Parabola => extrema.perform_parabola(point, &curve.parabola()?, tol, -inf, inf),
        _ => Ok(()),
    }
}

// `IsCaseAnalyticallyComputable`: a conic whose plane is not parallel to the
// extrusion direction.
fn is_case_analytically_computable(
    curve_type: CurveType,
    curve_pos: &Axis2,
    surface_direction: Direction3,
) -> bool {
    match curve_type {
        CurveType::Line
        | CurveType::Circle
        | CurveType::Ellipse
        | CurveType::Hyperbola
        | CurveType::Parabola => {
            curve_pos.direction().dot(surface_direction).abs() > gp::resolution()
        }
        _ => false,
    }
}

// `GetValue`: the point of the conic at `u`, the origin for another curve.
fn get_value(u: f64, curve: &dyn Adaptor3dCurve) -> Result<Point3, Failure> {
    Ok(match curve.get_type()? {
        CurveType::Line => elclib::value(u, &curve.line()?),
        CurveType::Circle => elclib::value(u, &curve.circle()?),
        CurveType::Ellipse => elclib::value(u, &curve.ellipse()?),
        CurveType::Hyperbola => elclib::value(u, &curve.hyperbola()?),
        CurveType::Parabola => elclib::value(u, &curve.parabola()?),
        _ => Point3::default(),
    })
}

impl ExtPExtS {
    /// `Extrema_ExtPExtS()`: not done; `ExtPExtS::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtPExtS(P, S, Umin, Usup, Vmin, Vsup, TolU, TolV)`;
    /// `ExtPExtS::with_range(point, &extrusion, (0., 1.), (-1., 1.), (1e-9, 1e-9))?`.
    pub fn with_range(
        point: Point3,
        surface: &Handle<GeomAdaptorSurfaceOfLinearExtrusion>,
        u_range: (f64, f64),
        v_range: (f64, f64),
        tolerances: (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(surface, u_range, v_range, tolerances)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Extrema_ExtPExtS(P, S, TolU, TolV)`: over the surface bounds;
    /// `ExtPExtS::with_surface(point, &extrusion, 1e-9, 1e-9)?`.
    pub fn with_surface(
        point: Point3,
        surface: &Handle<GeomAdaptorSurfaceOfLinearExtrusion>,
        tol_u: f64,
        tol_v: f64,
    ) -> Result<Self, Failure> {
        let adaptor = Self::adaptor(surface)?;
        let u_range = (adaptor.first_u_parameter()?, adaptor.last_u_parameter()?);
        let v_range = (adaptor.first_v_parameter()?, adaptor.last_v_parameter()?);
        Self::with_range(point, surface, u_range, v_range, (tol_u, tol_v))
    }
    // The adaptor of a handle, Standard_NullObject for a null handle.
    fn adaptor(
        surface: &Handle<GeomAdaptorSurfaceOfLinearExtrusion>,
    ) -> Result<&GeomAdaptorSurfaceOfLinearExtrusion, Failure> {
        surface
            .get()
            .ok_or_else(|| null_object("Extrema_ExtPExtS::Initialize()"))
    }
    /// `Initialize(S, Uinf, Usup, Vinf, Vsup, TolU, TolV)`: the analytical case, or
    /// Extrema_GenExtPS on a 32 x 32 grid;
    /// `extrema.initialize(&extrusion, (0., 1.), (-1., 1.), (1e-9, 1e-9))?`.
    pub fn initialize(
        &mut self,
        surface: &Handle<GeomAdaptorSurfaceOfLinearExtrusion>,
        (uinf, usup): (f64, f64),
        (vinf, vsup): (f64, f64),
        (tol_u, tol_v): (f64, f64),
    ) -> Result<(), Failure> {
        (self.uinf, self.usup, self.tolu) = (uinf, usup, tol_u);
        (self.vinf, self.vsup, self.tolv) = (vinf, vsup, tol_v);
        self.is_analytically_computable = false;
        self.done = false;
        self.nb_ext = 0;
        let adaptor = Self::adaptor(surface)?;
        let curve: Handle<dyn Adaptor3dCurve> = adaptor.basis_curve()?;
        let shared = SurfaceRef::Shared(surface.cast::<dyn Adaptor3dSurface>());
        self.f.initialize_ref(shared.clone());
        self.c = curve;
        self.s = surface.clone();
        let basis_handle: Handle<dyn Adaptor3dCurve> = self.c.clone();
        let basis: &dyn Adaptor3dCurve = basis_handle
            .get()
            .ok_or_else(|| null_object("Extrema_ExtPExtS::Initialize()"))?;
        self.position = get_position(basis)?;
        let direction: Direction3 = adaptor.direction()?;
        self.direction = Vector3::from(direction);
        self.is_analytically_computable = is_case_analytically_computable(
            basis.get_type()?,
            &self.position,
            self.direction_dir()?,
        );
        if !self.is_analytically_computable {
            let sampling = Sampling {
                nb_u: 32,
                nb_v: 32,
                tol_u,
                tol_v,
            };
            self.ext_ps
                .initialize_range_ref(shared, sampling, (uinf, usup), (vinf, vsup))?;
        }
        Ok(())
    }
    // The basis curve, Standard_NullObject when there is none.
    fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.c
            .get()
            .ok_or_else(|| null_object("Extrema_ExtPExtS::Perform()"))
    }
    // `myDirection` as a gp_Dir (the native implicit conversion normalizes it).
    fn direction_dir(&self) -> Result<Direction3, Failure> {
        Direction3::from_vector(self.direction)
    }
    /// `Perform(P)`: in the grid case, NbExt of the grid search (StdFail_NotDone when
    /// it found nothing); `extrema.perform(point)?`.
    pub fn perform(&mut self, point: Point3) -> Result<(), Failure> {
        self.done = false;
        self.nb_ext = 0;
        if !self.is_analytically_computable {
            self.ext_ps.perform(point)?;
            self.done = self.ext_ps.is_done();
            self.nb_ext = self.ext_ps.nb_ext()?;
            return Ok(());
        }
        self.perform_analytic(point)
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtPExtS::NbExt()"));
        }
        if self.is_analytically_computable {
            return Ok(self.nb_ext);
        }
        self.ext_ps.nb_ext()
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let index: usize =
            extremum_index("Extrema_ExtPExtS::SquareDistance()", n, || self.nb_ext())?;
        if self.is_analytically_computable {
            return Ok(self.sq_dist[index]);
        }
        self.ext_ps.square_distance(n)
    }
    /// `Point(N)`; `extrema.point(1)?`.
    pub fn point(&self, n: i32) -> Result<POnSurf, Failure> {
        let index: usize = extremum_index("Extrema_ExtPExtS::Point()", n, || self.nb_ext())?;
        if self.is_analytically_computable {
            return Ok(self.points[index]);
        }
        self.ext_ps.point(n)
    }
}
