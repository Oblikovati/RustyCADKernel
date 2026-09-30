// Rust translation of BRepTopAdaptor_TopolTool.cxx (samples) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sample points of `BRepTopAdaptor_TopolTool`: counts by surface kind (at
//! least 10 each, the pole grid of a spline analysed above 10), on a regular grid
//! strictly inside the (finite) surface range, and the inherited SamplePnts parameters.
use super::errors::{c_int_cast, null_object};
use super::topol_tool::BRepTopAdaptorTopolTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision::{is_negative_infinite, is_positive_infinite};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};

impl BRepTopAdaptorTopolTool {
    // The surface, read without keeping the lock; natively a null one is dereferenced.
    fn live_surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.state.read().surface.clone();
        if surface.is_null() {
            return Err(null_object(
                "BRepTopAdaptor_TopolTool: no surface; expected Initialize(S) first",
            ));
        }
        Ok(surface)
    }

    /// `ComputeSamplePoints()`: the sample counts (a plane 2, a quadric 8 per unit of
    /// U and 7 per unit of V within [5, 30] x [5, 15], a revolution or extrusion
    /// 25 x 15, a Bezier 3 + poles, a B-spline knots x degree, at least 10 each; a
    /// spline above 10 is analysed by its pole grid), the grid origin and steps;
    /// `tool.compute_sample_points()?`.
    pub fn compute_sample_points(&self) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let adapted: &dyn Adaptor3dSurface = &*surface;
        let (uinf, usup, vinf, vsup) = Adaptor3dTopolTool::sampled_box(adapted)?;
        let kind: SurfaceType = adapted.get_type()?;
        let (mut nbsu, mut nbsv) = kind_counts(adapted, kind, (usup - uinf, vsup - vinf))?;
        //-- If the number of points is too great, analyze
        nbsu = nbsu.max(10);
        nbsv = nbsv.max(10);
        if (nbsu > 10 || nbsv > 10)
            && let Some((u, v)) = analysed_counts(adapted, kind)?
        {
            (nbsu, nbsv) = (u, v);
        }
        nbsu = nbsu.max(10);
        nbsv = nbsv.max(10);
        let mut state = self.state.write();
        (state.nb_samples_u, state.nb_samples_v) = (nbsu, nbsv);
        (state.u0, state.v0) = (uinf, vinf);
        state.du = (usup - uinf) / f64::from(nbsu + 1);
        state.dv = (vsup - vinf) / f64::from(nbsv + 1);
        Ok(())
    }

    // The counts, computed on first use (`myNbSamplesU < 0`).
    fn counts(&self) -> Result<(i32, i32), Failure> {
        if self.state.read().nb_samples_u < 0 {
            self.compute_sample_points()?;
        }
        let state = self.state.read();
        Ok((state.nb_samples_u, state.nb_samples_v))
    }

    /// `NbSamplesU()`; `tool.nb_samples_u()?`.
    pub fn nb_samples_u(&self) -> Result<i32, Failure> {
        Ok(self.counts()?.0)
    }

    /// `NbSamplesV()`; `tool.nb_samples_v()?`.
    pub fn nb_samples_v(&self) -> Result<i32, Failure> {
        Ok(self.counts()?.1)
    }

    /// `NbSamples()`: `NbSamplesU * NbSamplesV`; `tool.nb_samples()?`.
    pub fn nb_samples(&self) -> Result<i32, Failure> {
        let (nbsu, nbsv) = self.counts()?;
        Ok(nbsu.wrapping_mul(nbsv))
    }

    /// `SamplePoint(Index, P2d, P3d)`: the grid point `(U0 + iu dU, V0 + iv dV)` with
    /// `iv = 1 + Index / NbSamplesU` and `iu = 1 + Index - (iv - 1) NbSamplesU`, and its
    /// surface point; it reads the counts as they are (Standard_DivideByZero natively
    /// for a zero count is a Standard_ProgramError here);
    /// `let (p2d, p3d) = tool.sample_point(1)?;`.
    pub fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        let (u, v) = {
            let state = self.state.read();
            let nbu: i32 = state.nb_samples_u;
            if nbu == 0 {
                return Err(Failure::with_message(
                    "Standard_ProgramError",
                    "BRepTopAdaptor_TopolTool::SamplePoint: NbSamplesU is 0; expected a sample count",
                ));
            }
            let iv: i32 = 1 + index / nbu;
            let iu: i32 = 1 + index - (iv - 1) * nbu;
            (
                state.u0 + f64::from(iu) * state.du,
                state.v0 + f64::from(iv) * state.dv,
            )
        };
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        Ok((Point2::new(u, v), surface.value(u, v)?))
    }

    /// `SamplePnts(theDefl, theNUmin, theNVmin)`: the inherited base member on the
    /// base's protected samples: the B-spline analysis of `BSplSamplePnts`, else the
    /// counts of the overriding [`Self::compute_sample_points`] (which also sets the
    /// sample grid) spread over the (finite) box (docs/topalgo-design.md section 8);
    /// `tool.sample_pnts(0.01, 10, 10)?`.
    pub fn sample_pnts(&self, deflection: f64, nu_min: i32, nv_min: i32) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let (u_pars, v_pars) = Adaptor3dTopolTool::sample_pnts_parameters(
            &*surface,
            (deflection, nu_min, nv_min),
            || {
                self.compute_sample_points()?;
                let state = self.state.read();
                Ok((state.nb_samples_u, state.nb_samples_v))
            },
        )?;
        self.store_sample_parameters(u_pars, v_pars);
        Ok(())
    }

    /// `BSplSamplePnts(theDefl, theNUmin, theNVmin)`: the inherited base member on the
    /// base's protected samples (the B-spline analysis of the surface);
    /// `tool.bspl_sample_pnts(0.01, 10, 10)?`.
    pub fn bspl_sample_pnts(
        &self,
        deflection: f64,
        nu_min: i32,
        nv_min: i32,
    ) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let (u_pars, v_pars) =
            Adaptor3dTopolTool::bspl_sample_parameters(&*surface, deflection, nu_min, nv_min)?;
        self.store_sample_parameters(u_pars, v_pars);
        Ok(())
    }

    // The protected myNbSamplesU/V and myUPars/myVPars that SamplePnts and BSplSamplePnts
    // set: the counts are the numbers of parameters.
    fn store_sample_parameters(&self, u_pars: Array1<'static, f64>, v_pars: Array1<'static, f64>) {
        let mut state = self.state.write();
        (state.nb_samples_u, state.nb_samples_v) = (u_pars.length(), v_pars.length());
        (state.u_pars, state.v_pars) = (Some(u_pars), Some(v_pars));
    }

    /// `UParameters(theArray)`: the inherited base member, a copy of the U parameters of
    /// [`Self::sample_pnts`]; Standard_NullObject before it (a null dereference
    /// natively); `tool.u_parameters()?`.
    pub fn u_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        self.state.read().u_pars.clone().ok_or_else(|| {
            null_object(
                "BRepTopAdaptor_TopolTool::UParameters: no parameters; expected SamplePnts()",
            )
        })
    }

    /// `VParameters(theArray)`: the V parameters, as [`Self::u_parameters`];
    /// `tool.v_parameters()?`.
    pub fn v_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        self.state.read().v_pars.clone().ok_or_else(|| {
            null_object(
                "BRepTopAdaptor_TopolTool::VParameters: no parameters; expected SamplePnts()",
            )
        })
    }

    /// `IsUniformSampling()`: the inherited base member, false for a B-spline surface;
    /// `tool.is_uniform_sampling()?`.
    pub fn is_uniform_sampling(&self) -> Result<bool, Failure> {
        Adaptor3dTopolTool::uniform_sampling_of(&*self.live_surface()?)
    }

    /// `DomainIsInfinite()`: whether a bound of the surface range is infinite;
    /// `tool.domain_is_infinite()?`.
    pub fn domain_is_infinite(&self) -> Result<bool, Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        Ok(is_negative_infinite(surface.first_u_parameter()?)
            || is_positive_infinite(surface.last_u_parameter()?)
            || is_negative_infinite(surface.first_v_parameter()?)
            || is_positive_infinite(surface.last_v_parameter()?))
    }
}

// The counts of ComputeSamplePoints before the minimum of 10, by surface kind.
fn kind_counts(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    (u_length, v_length): (f64, f64),
) -> Result<(i32, i32), Failure> {
    Ok(match kind {
        SurfaceType::Plane => (2, 2),
        SurfaceType::BezierSurface => (3 + surface.nb_u_poles()?, 3 + surface.nb_v_poles()?),
        SurfaceType::BSplineSurface => {
            let nbsv: i32 = surface.nb_v_knots()? * surface.v_degree()?;
            let nbsu: i32 = surface.nb_u_knots()? * surface.u_degree()?;
            (nbsu.max(4), nbsv.max(4))
        }
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus => {
            //-- Set 15 for 2pi
            //-- Not enough ->25 for 2pi
            let nbsu: i32 = c_int_cast(8.0 * u_length);
            let nbsv: i32 = c_int_cast(7.0 * v_length);
            // modif HRT buc60462: at most 30 in U.
            (nbsu.clamp(5, 30), nbsv.clamp(5, 15))
        }
        SurfaceType::SurfaceOfRevolution | SurfaceType::SurfaceOfExtrusion => (25, 15),
        _ => (10, 10),
    })
}

// The counts of the pole grid of a B-spline or Bezier surface (`Analyse`), None for
// the other kinds.
fn analysed_counts(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
) -> Result<Option<(i32, i32)>, Failure> {
    let poles = match kind {
        SurfaceType::BSplineSurface => surface.bspline()?.get().map(|bspl| bspl.poles()),
        SurfaceType::BezierSurface => surface.bezier()?.get().map(|bez| bez.poles()),
        _ => return Ok(None),
    };
    let Some(poles) = poles else {
        return Err(null_object(
            "BRepTopAdaptor_TopolTool::ComputeSamplePoints: no spline behind the surface; expected one",
        ));
    };
    Adaptor3dTopolTool::analyse_pole_grid(&poles).map(Some)
}
