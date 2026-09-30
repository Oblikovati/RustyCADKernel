// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_TopolTool.cxx (Analyse, ComputeSamplePoints, NbSamples*, U/VParameters, SamplePoint, SamplePnts, IsUniformSampling, GetConeApexParam) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sample points of the intersection algorithms: counts by surface kind
//! (refined by the pole grid of splines), or explicit parameters.
use super::super::surface::Adaptor3dSurface;
use super::Adaptor3dTopolTool;
use crate::geom::errors::{array_bounds, null_object, program};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Cone3, Point2, Point3, Transform3, Vector3};
use std::f64::consts::PI;

/// The adaptor range, ordered, an infinite side replaced by 2e5 units (a doubly
/// infinite one by [-1e5, 1e5]), as ComputeSamplePoints and SamplePnts read it.
pub(super) fn sampled_box(surface: &dyn Adaptor3dSurface) -> Result<(f64, f64, f64, f64), Failure> {
    let (u1, u2) = ordered(surface.first_u_parameter()?, surface.last_u_parameter()?);
    let (v1, v2) = ordered(surface.first_v_parameter()?, surface.last_v_parameter()?);
    let (u1, u2) = finite(u1, u2);
    let (v1, v2) = finite(v1, v2);
    Ok((u1, u2, v1, v2))
}
fn ordered(first: f64, last: f64) -> (f64, f64) {
    if last < first {
        (last, first)
    } else {
        (first, last)
    }
}
// RealFirst()/RealLast() are -DBL_MAX/DBL_MAX, compared exactly.
fn finite(first: f64, last: f64) -> (f64, f64) {
    match (first == -f64::MAX, last == f64::MAX) {
        (true, true) => (-1.0e5, 1.0e5),
        (true, false) => (last - 2.0e5, last),
        (false, true) => (first, first + 2.0e5),
        (false, false) => (first, last),
    }
}

// The counts of ComputeSamplePoints before refinement, by surface kind.
fn kind_counts(surface: &dyn Adaptor3dSurface, kind: SurfaceType) -> Result<(i32, i32), Failure> {
    Ok(match kind {
        SurfaceType::Plane => (2, 2),
        SurfaceType::BezierSurface => (3 + surface.nb_u_poles()?, 3 + surface.nb_v_poles()?),
        SurfaceType::BSplineSurface => {
            let nbsv: i32 = (surface.nb_v_knots()? * surface.v_degree()?).max(4);
            let nbsu: i32 = (surface.nb_u_knots()? * surface.u_degree()?).max(4);
            (nbsu, nbsv)
        }
        SurfaceType::Cylinder
        | SurfaceType::Cone
        | SurfaceType::Sphere
        | SurfaceType::Torus
        | SurfaceType::SurfaceOfRevolution
        | SurfaceType::SurfaceOfExtrusion => (15, 15),
        _ => (10, 10),
    })
}

/// `ComputeSamplePoints`: the sample counts of the surface (at least 6 each); a
/// spline with more than 8 is analysed by its pole grid, and a B-spline whose box is
/// ten times longer in one direction gets twice the samples there (at most 50);
/// `compute_counts(&*surface)?`.
pub(super) fn compute_counts(surface: &dyn Adaptor3dSurface) -> Result<(i32, i32), Failure> {
    const MAX_NB_SAMPLE: i32 = 50;
    let (uinf, usup, vinf, vsup) = sampled_box(surface)?;
    let kind: SurfaceType = surface.get_type()?;
    let (mut nbsu, mut nbsv) = kind_counts(surface, kind)?;
    nbsu = nbsu.max(6);
    nbsv = nbsv.max(6);
    let poles: Option<Array2<'static, Point3>> = match kind {
        SurfaceType::BSplineSurface => Some(live(surface.bspline()?, "BSpline")?.poles()),
        SurfaceType::BezierSurface => Some(live(surface.bezier()?, "Bezier")?.poles()),
        _ => None,
    };
    if let Some(grid) = poles.filter(|_| nbsu > 8 || nbsv > 8) {
        (nbsu, nbsv) = analyse(&grid)?;
    }
    if kind == SurfaceType::BSplineSurface {
        // Check anisotropy.
        let u_length: f64 = (usup - uinf) / surface.u_resolution(1.)?;
        let v_length: f64 = (vsup - vinf) / surface.v_resolution(1.)?;
        let ratio: f64 = u_length / v_length;
        if ratio >= 10. {
            nbsu = (nbsu * 2).min(MAX_NB_SAMPLE);
        } else if ratio <= 0.1 {
            nbsv = (nbsv * 2).min(MAX_NB_SAMPLE);
        }
    }
    Ok((nbsu, nbsv))
}

fn live<T: rustycad_tkernel::standard::RuntimeTyped + ?Sized>(
    handle: Handle<T>,
    member: &str,
) -> Result<Handle<T>, Failure> {
    if handle.is_null() {
        return Err(null_object(&format!(
            "Adaptor3d_TopolTool: {member}() returned a null surface; expected one"
        )));
    }
    Ok(handle)
}

// The second difference A - 2B + C of three poles.
fn second_difference(a: Point3, b: Point3, c: Point3) -> Vector3 {
    Vector3::new(
        c.x() - b.x() - b.x() + a.x(),
        c.y() - b.y() - b.y() + a.y(),
        c.z() - b.z() - b.z() + a.z(),
    )
}

/// The sign-change counter of `Analyse`; natively the sign `sh` carries over from the
/// V scan to the U scan.
struct SignChanges {
    sign: i32,
}
impl SignChanges {
    // The changes of convexity along one pole line, `pole(k)` for k = 1..count.
    fn along(
        &mut self,
        count: i32,
        pole: impl Fn(i32) -> Result<Point3, Failure>,
    ) -> Result<i32, Failure> {
        let mut previous: Vector3 = second_difference(pole(1)?, pole(2)?, pole(3)?);
        let mut changes: i32 = 0;
        for k in 3..count {
            let next: Vector3 = second_difference(pole(k - 1)?, pole(k)?, pole(k + 1)?);
            let product: f64 = previous.dot(next);
            previous = next;
            if !(product > 1.0e-7 || product < -1.0e-7) {
                continue;
            }
            let wanted: i32 = if product > 0. { 1 } else { -1 };
            if self.sign != wanted {
                self.sign = wanted;
                changes += 1;
            }
        }
        Ok(changes)
    }
}

/// `Analyse`: the sample counts from the pole grid, 5 plus the largest number of
/// convexity changes along the poles of one direction; `analyse(&poles)?`.
pub(super) fn analyse(grid: &Array2<'_, Point3>) -> Result<(i32, i32), Failure> {
    let (nbup, nbvp) = (grid.nb_rows() as i32, grid.nb_columns() as i32);
    let at = |i: i32, j: i32| grid.value(i, j).copied().map_err(array_bounds);
    let mut counter = SignChanges { sign: 1 };
    let mut changes: i32 = 0;
    if nbvp > 2 {
        for i in 2..nbup {
            changes = changes.max(counter.along(nbvp, |j| at(i, j))?);
        }
    }
    let nb_samples_v: i32 = changes + 5;
    changes = 0;
    if nbup > 2 {
        for j in 2..nbvp {
            changes = changes.max(counter.along(nbup, |i| at(i, j))?);
        }
    }
    Ok((changes + 5, nb_samples_v))
}

impl Adaptor3dTopolTool {
    /// Compute the sample counts (`ComputeSamplePoints`); `tool.compute_sample_points()?`.
    pub fn compute_sample_points(&self) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let adapted: &dyn Adaptor3dSurface = &*surface;
        let (nbsu, nbsv) = compute_counts(adapted)?;
        let mut state = self.state.write();
        (state.nb_samples_u, state.nb_samples_v) = (nbsu, nbsv);
        Ok(())
    }
    fn counts(&self) -> Result<(i32, i32), Failure> {
        if self.state.read().nb_samples_u < 0 {
            self.compute_sample_points()?;
        }
        let state = self.state.read();
        Ok((state.nb_samples_u, state.nb_samples_v))
    }
    /// The U sample count, computed on first use (`NbSamplesU`); `tool.nb_samples_u()?`.
    pub fn nb_samples_u(&self) -> Result<i32, Failure> {
        Ok(self.counts()?.0)
    }
    /// The V sample count, computed on first use (`NbSamplesV`); `tool.nb_samples_v()?`.
    pub fn nb_samples_v(&self) -> Result<i32, Failure> {
        Ok(self.counts()?.1)
    }
    /// The number of sample points (`NbSamples`); `tool.nb_samples()?`.
    pub fn nb_samples(&self) -> Result<i32, Failure> {
        let (nbsu, nbsv) = self.counts()?;
        Ok(nbsu.wrapping_mul(nbsv))
    }
    /// A copy of the U sample parameters of SamplePnts (`UParameters`, the array
    /// taking their bounds); Standard_NullObject before SamplePnts; `tool.u_parameters()?`.
    pub fn u_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        let state = self.state.read();
        state.u_pars.clone().ok_or_else(|| {
            null_object("Adaptor3d_TopolTool::UParameters: no parameters; expected SamplePnts()")
        })
    }
    /// A copy of the V sample parameters (`VParameters`); `tool.v_parameters()?`.
    pub fn v_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        let state = self.state.read();
        state.v_pars.clone().ok_or_else(|| {
            null_object("Adaptor3d_TopolTool::VParameters: no parameters; expected SamplePnts()")
        })
    }
    /// The sample point of index `i` and its surface point (`SamplePoint`): from the
    /// SamplePnts parameters (1-based), else a regular grid strictly inside the
    /// rectangle; `let (p2d, p3d) = tool.sample_point(1)?;`.
    pub fn sample_point(&self, i: i32) -> Result<(Point2, Point3), Failure> {
        let (uv, surface) = {
            let state = self.state.read();
            let nbu: i32 = state.nb_samples_u;
            if nbu == 0 {
                return Err(program(
                    "Adaptor3d_TopolTool::SamplePoint: no U samples; expected NbSamplesU() > 0",
                ));
            }
            let uv = match (&state.u_pars, &state.v_pars) {
                (Some(u_pars), Some(v_pars)) => {
                    let (iv, iu) = ((i - 1) / nbu + 1, (i - 1) % nbu + 1);
                    (
                        *u_pars.value(iu).map_err(array_bounds)?,
                        *v_pars.value(iv).map_err(array_bounds)?,
                    )
                }
                _ => grid_parameters(&state, i),
            };
            (uv, state.surface.clone())
        };
        let adapted: &dyn Adaptor3dSurface = surface
            .get()
            .ok_or_else(|| null_object("Adaptor3d_TopolTool: no surface"))?;
        Ok((Point2::new(uv.0, uv.1), adapted.value(uv.0, uv.1)?))
    }
    /// Whether the samples are a regular grid: not for B-splines (`IsUniformSampling`);
    /// `tool.is_uniform_sampling()?`.
    pub fn is_uniform_sampling(&self) -> Result<bool, Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let adapted: &dyn Adaptor3dSurface = &*surface;
        Ok(adapted.get_type()? != SurfaceType::BSplineSurface)
    }
    /// Sample parameters with deflection `deflection` and at least the given counts
    /// (`SamplePnts`): the B-spline analysis, else the counts of ComputeSamplePoints
    /// spread evenly over the (finite) box; `tool.sample_pnts(0.01, 10, 10)?`.
    pub fn sample_pnts(&self, deflection: f64, nu_min: i32, nv_min: i32) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let adapted: &dyn Adaptor3dSurface = &*surface;
        let (uinf, usup, vinf, vsup) = sampled_box(adapted)?;
        if adapted.get_type()? == SurfaceType::BSplineSurface {
            return self.bspl_sample_pnts(deflection, nu_min, nv_min);
        }
        let (nbsu, nbsv) = compute_counts(adapted)?;
        let (u_pars, v_pars) = (spread(uinf, usup, nbsu)?, spread(vinf, vsup, nbsv)?);
        let mut state = self.state.write();
        (state.nb_samples_u, state.nb_samples_v) = (nbsu, nbsv);
        (state.u_pars, state.v_pars) = (Some(u_pars), Some(v_pars));
        Ok(())
    }
}

// The grid of SamplePoint without parameters: step (sup - inf) / (N + 1), indices from 1.
fn grid_parameters(state: &super::TopolState, i: i32) -> (f64, f64) {
    let (nbu, nbv) = (state.nb_samples_u, state.nb_samples_v);
    let du: f64 = (state.usup - state.uinf) / f64::from(nbu + 1);
    let dv: f64 = (state.vsup - state.vinf) / f64::from(nbv + 1);
    let iv: i32 = 1 + i / nbu;
    let iu: i32 = 1 + i - (iv - 1) * nbu;
    (
        state.uinf + f64::from(iu) * du,
        state.vinf + f64::from(iv) * dv,
    )
}

/// `count` parameters from `first` to `last`, the inner ones accumulated by
/// `t += dt`; `spread(0.0, 1.0, 5)?`.
pub(super) fn spread(first: f64, last: f64, count: i32) -> Result<Array1<'static, f64>, Failure> {
    let mut values: Array1<'static, f64> = Array1::with_bounds(1, count).map_err(array_bounds)?;
    let dt: f64 = (last - first) / f64::from(count - 1);
    values.set_value(1, first).map_err(array_bounds)?;
    values.set_value(count, last).map_err(array_bounds)?;
    let mut t: f64 = first + dt;
    for i in 2..count {
        values.set_value(i, t).map_err(array_bounds)?;
        t += dt;
    }
    Ok(values)
}

/// `GetConeApexParam`: the parameters of the apex in the cone frame, U in [0, 2 PI)
/// (0 for an apex on the axis, turned by PI when the apex is on the "wrong" side);
/// `cone_apex_param(&cone)`.
pub(super) fn cone_apex_param(cone: &Cone3) -> (f64, f64) {
    let (radius, semi_angle) = (cone.ref_radius(), cone.semi_angle());
    let mut transform = Transform3::default();
    transform.set_transformation_to(cone.position());
    let local: Point3 = cone.apex().transformed(transform);
    let mut u: f64 = if local.x() == 0.0 && local.y() == 0.0 {
        0.0
    } else if -radius > local.z() * semi_angle.tan() {
        // The point is at the `wrong` side of the apex.
        (-local.y()).atan2(-local.x())
    } else {
        local.y().atan2(local.x())
    };
    if u < -1.0e-16 {
        u += PI + PI;
    } else if u < 0. {
        u = 0.;
    }
    let v: f64 = semi_angle.sin() * (local.x() * u.cos() + local.y() * u.sin() - radius)
        + semi_angle.cos() * local.z();
    (u, v)
}
