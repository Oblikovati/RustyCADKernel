// Rust translation of ModelingData/TKG3d/Geom/Geom_OsculatingSurface.pxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_OsculatingSurface` (a private helper class of Geom_OffsetSurface): for a
//! B-spline or Bezier surface with a degenerate (quasi-punctual) boundary, one
//! reduced osculating B-spline per span along that boundary, used to find the
//! normal of an offset where the basis normal vanishes.
mod build;
mod punctual;
use crate::geom::offset_surface_utils::OsculatingQuery;
use crate::geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurface};
use crate::geom_adaptor::GeomAdaptorSurface;
use build::{Along, build_osculating_surface};
use punctual::is_q_punctual;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib;

/// The osculating surfaces of a basis; cloning shares the surfaces, as the native
/// copy constructor does; `GeomOsculatingSurface::new(&bspline.cast(), 1e-7)?`.
#[derive(Clone, Default)]
pub struct GeomOsculatingSurface {
    basis: Handle<dyn GeomSurface>,
    tol: f64,
    osc1: Vec<Handle<GeomBSplineSurface>>,
    osc2: Vec<(Handle<GeomBSplineSurface>, bool)>,
    along: [bool; 4],
}

impl GeomOsculatingSurface {
    /// The osculating surfaces of `surface` for tolerance `tol`
    /// (`Geom_OsculatingSurface(theBS, theTol)`); `GeomOsculatingSurface::new(&s, 1e-7)?`.
    pub fn new(surface: &Handle<dyn GeomSurface>, tol: f64) -> Result<Self, Failure> {
        let mut result = Self::default();
        result.init(surface, tol)?;
        Ok(result)
    }
    /// Recompute for `surface` (`Init`). The flags are cleared whenever the
    /// computation does not complete, a failure included; the surfaces appended
    /// before an early stop stay, as natively; `osc.init(&s, 1e-7)?`.
    pub fn init(&mut self, surface: &Handle<dyn GeomSurface>, tol: f64) -> Result<(), Failure> {
        self.along = [false; 4];
        self.tol = tol;
        self.basis = Handle::default();
        self.osc1.clear();
        self.osc2.clear();
        let done: Result<bool, Failure> = self.compute(surface, tol);
        if !matches!(done, Ok(true)) {
            self.along = [false; 4];
        }
        done.map(|_| ())
    }
    /// The copied basis surface (`BasisSurface`); `osc.basis_surface()`.
    pub fn basis_surface(&self) -> Handle<dyn GeomSurface> {
        self.basis.clone()
    }
    /// The tolerance (`Tolerance`); `osc.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }
    /// Whether any boundary is degenerate (`HasOscSurf`); `osc.has_osc_surf()`.
    pub fn has_osc_surf(&self) -> bool {
        self.along.iter().any(|flag| *flag)
    }
    /// Whether a V-bound iso is degenerate (`IsAlongU`); `osc.is_along_u()`.
    pub fn is_along_u(&self) -> bool {
        self.along[0] || self.along[1]
    }
    /// Whether a U-bound iso is degenerate (`IsAlongV`); `osc.is_along_v()`.
    pub fn is_along_v(&self) -> bool {
        self.along[2] || self.along[3]
    }

    // The body of Init after the reset; Ok(true) when every reduction was built.
    fn compute(&mut self, surface: &Handle<dyn GeomSurface>, tol: f64) -> Result<bool, Failure> {
        let Some(source) = surface.get() else {
            return Ok(false);
        };
        self.basis = source.copy()?.cast();
        let bezier: Handle<GeomBezierSurface> = surface.cast();
        let is_bspline: bool = surface.cast::<GeomBSplineSurface>().is_some();
        if !is_bspline && bezier.is_null() {
            return Ok(false);
        }
        let bounds = source.bounds();
        let (u1, u2, v1, v2) = bounds;
        let adaptor = GeomAdaptorSurface::with_bounds(surface, bounds)?;
        self.along[0] = is_q_punctual(&adaptor, v1, true, 0.0, tol)?;
        self.along[1] = is_q_punctual(&adaptor, v2, true, 0.0, tol)?;
        self.along[2] = is_q_punctual(&adaptor, u1, false, 0.0, tol)?;
        self.along[3] = is_q_punctual(&adaptor, u2, false, 0.0, tol)?;
        if !self.has_osc_surf() {
            return Ok(false);
        }
        let init: Handle<GeomBSplineSurface> = match bezier.get() {
            Some(bezier) => Handle::new(bezier_as_bspline(bezier)?),
            None => self.basis.cast(),
        };
        let Some(init_surface) = init.get() else {
            return Ok(false);
        };
        if self.is_along_u() && self.is_along_v() {
            return Ok(false);
        }
        if (self.is_along_u() && init_surface.v_degree() <= 1)
            || (self.is_along_v() && init_surface.u_degree() <= 1)
        {
            return Ok(false);
        }
        self.reduce_all(&init, bounds)
    }

    // The reductions of every span along the degenerate bounds.
    fn reduce_all(
        &mut self,
        init: &Handle<GeomBSplineSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<bool, Failure> {
        let (u1, u2, v1, v2) = bounds;
        let surface: &GeomBSplineSurface = init.get().expect("checked by the caller");
        let (nb_u, nb_v) = (surface.nb_u_knots(), surface.nb_v_knots());
        if self.along[0] || self.along[1] {
            for i in 1..nb_u {
                if !self.reduce_pair(init, [(v1, (i, 1)), (v2, (i, nb_v - 1))], true, [0, 1])? {
                    return Ok(false);
                }
            }
        }
        if self.along[2] || self.along[3] {
            for i in 1..nb_v {
                if !self.reduce_pair(init, [(u1, (1, i)), (u2, (nb_u - 1, i))], false, [2, 3])? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    // The first and second bound of one span: myOsculSurf1 gets the first one,
    // myOsculSurf2 the second with its opposite flag.
    fn reduce_pair(
        &mut self,
        init: &Handle<GeomBSplineSurface>,
        bounds: [(f64, (i32, i32)); 2],
        iso_v: bool,
        flags: [usize; 2],
    ) -> Result<bool, Failure> {
        if self.along[flags[0]] {
            let Some((surface, _)) =
                self.reduce_along_iso(init, bounds[0].0, iso_v, bounds[0].1)?
            else {
                return Ok(false);
            };
            self.osc1.push(surface);
        }
        if self.along[flags[1]] {
            let Some((surface, reductions)) =
                self.reduce_along_iso(init, bounds[1].0, iso_v, bounds[1].1)?
            else {
                return Ok(false);
            };
            self.osc2.push((surface, reductions % 2 != 0));
        }
        Ok(true)
    }

    // reduceAlongIso: reduce until the iso is no longer quasi-punctual; the reduced
    // surface and the number of reductions, None when a reduction fails.
    fn reduce_along_iso(
        &self,
        init: &Handle<GeomBSplineSurface>,
        param: f64,
        iso_v: bool,
        knots: (i32, i32),
    ) -> Result<Option<(Handle<GeomBSplineSurface>, usize)>, Failure> {
        let along = Along {
            u: self.is_along_u(),
            v: self.is_along_v(),
        };
        let (mut surface, mut knots, mut reductions) = (init.clone(), knots, 0usize);
        let adaptor = GeomAdaptorSurface::default();
        loop {
            let current: &GeomBSplineSurface = surface.get().expect("reductions are never null");
            let Some(reduced) = build_osculating_surface(param, knots, current, along)? else {
                return Ok(None);
            };
            reductions += 1;
            adaptor.load(&reduced.cast())?;
            let still_punctual: bool = is_q_punctual(&adaptor, param, iso_v, 0.0, self.tol)?;
            knots = (1, 1);
            surface = reduced;
            if !still_punctual {
                return Ok(Some((surface, reductions)));
            }
        }
    }

    // The span (1-based knot index) of a parameter on the B-spline basis, and
    // whether the second bound must be skipped, for one direction of the queries.
    fn span_of(
        &self,
        uv: (f64, f64),
        u_direction: bool,
    ) -> Result<(i32, i32, i32, i32, bool), Failure> {
        let bspline: Handle<GeomBSplineSurface> = self.basis.cast();
        let Some(bspline) = bspline.get() else {
            return Ok((1, 1, 2, 2, false));
        };
        let (nb_uk, nb_vk) = (bspline.nb_u_knots(), bspline.nb_v_knots());
        let (u_knots, v_knots) = (bspline.u_knots(), bspline.v_knots());
        let (mut nu, mut nv) = (1, 1);
        bsplclib::hunt(&u_knots, uv.0, &mut nu)?;
        bsplclib::hunt(&v_knots, uv.1, &mut nv)?;
        let skip_second: bool = if u_direction {
            nu = nu.clamp(1, nb_uk - 1);
            nb_vk == 2 && nv == 1 && v_knots[nb_vk] - uv.1 > uv.1 - v_knots[1]
        } else {
            nv = nv.clamp(1, nb_vk - 1);
            nb_uk == 2 && nu == 1 && u_knots[nb_uk] - uv.0 > uv.0 - u_knots[1]
        };
        Ok((nu, nv, nb_uk, nb_vk, skip_second))
    }

    /// The osculating surface along U at `(u, v)`: the first-bound surface of the
    /// span on the first V span, the second-bound one (with its opposite flag) on the
    /// last V span (`UOsculatingSurface`); `is_opposite` and `surface` are untouched
    /// when the U flags are unset; `osc.u_osculating(u, v, &mut t, &mut l)?`.
    pub fn u_osculating(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<GeomBSplineSurface>,
    ) -> Result<bool, Failure> {
        if !(self.along[0] || self.along[1]) {
            return Ok(false);
        }
        *is_opposite = false;
        let (nu, nv, _, nb_vk, skip_second) = self.span_of((u, v), true)?;
        self.pick(
            nu,
            (nv == 1, nv == nb_vk - 1 && !skip_second),
            [0, 1],
            (is_opposite, surface),
        )
    }

    /// The osculating surface along V at `(u, v)` (`VOsculatingSurface`);
    /// `osc.v_osculating(u, v, &mut t, &mut l)?`.
    pub fn v_osculating(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<GeomBSplineSurface>,
    ) -> Result<bool, Failure> {
        if !(self.along[2] || self.along[3]) {
            return Ok(false);
        }
        *is_opposite = false;
        let (nu, nv, nb_uk, _, skip_second) = self.span_of((u, v), false)?;
        self.pick(
            nv,
            (nu == 1, nu == nb_uk - 1 && !skip_second),
            [2, 3],
            (is_opposite, surface),
        )
    }

    // The surface of span `index` on the first and/or second bound.
    fn pick(
        &self,
        index: i32,
        bounds: (bool, bool),
        flags: [usize; 2],
        outputs: (&mut bool, &mut Handle<GeomBSplineSurface>),
    ) -> Result<bool, Failure> {
        let (is_opposite, surface) = outputs;
        let slot: usize = usize::try_from(index - 1).map_err(|_| out_of_range(index))?;
        let mut along: bool = false;
        if self.along[flags[0]] && bounds.0 {
            *surface = self
                .osc1
                .get(slot)
                .ok_or_else(|| out_of_range(index))?
                .clone();
            along = true;
        }
        if self.along[flags[1]] && bounds.1 {
            let (second, opposite) = self.osc2.get(slot).ok_or_else(|| out_of_range(index))?;
            *is_opposite = *opposite;
            *surface = second.clone();
            along = true;
        }
        Ok(along)
    }
}

fn out_of_range(index: i32) -> Failure {
    crate::geom::errors::out_of_range(&format!(
        "Geom_OsculatingSurface: span {index} has no osculating surface; expected a computed span"
    ))
}

// The Bezier surface as a one-span B-spline on [0, 1] x [0, 1].
fn bezier_as_bspline(bezier: &GeomBezierSurface) -> Result<GeomBSplineSurface, Failure> {
    let knots = |_: ()| -> Result<Array1<'static, f64>, Failure> {
        let mut k = Array1::with_bounds(1, 2).map_err(crate::geom::errors::array_bounds)?;
        k[1] = 0.0;
        k[2] = 1.0;
        Ok(k)
    };
    let mults = |degree: i32| -> Result<Array1<'static, i32>, Failure> {
        let mut m = Array1::with_bounds(1, 2).map_err(crate::geom::errors::array_bounds)?;
        m[1] = degree + 1;
        m[2] = degree + 1;
        Ok(m)
    };
    let (u_deg, v_deg) = (bezier.u_degree(), bezier.v_degree());
    let (u_knots, v_knots) = (knots(())?, knots(())?);
    let (u_mults, v_mults) = (mults(u_deg)?, mults(v_deg)?);
    let poles: Array2<'static, rustycad_tkmath::gp::Point3> = bezier.poles();
    GeomBSplineSurface::new(
        &poles,
        (&u_knots, &u_mults),
        (&v_knots, &v_mults),
        (u_deg, v_deg),
        (bezier.is_u_periodic(), bezier.is_v_periodic()),
    )
}

impl OsculatingQuery for GeomOsculatingSurface {
    fn u_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<dyn GeomSurface>,
    ) -> Result<bool, Failure> {
        let mut found: Handle<GeomBSplineSurface> = surface.cast();
        let along: bool = self.u_osculating(u, v, is_opposite, &mut found)?;
        if along {
            *surface = found.cast();
        }
        Ok(along)
    }
    fn v_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<dyn GeomSurface>,
    ) -> Result<bool, Failure> {
        let mut found: Handle<GeomBSplineSurface> = surface.cast();
        let along: bool = self.v_osculating(u, v, is_opposite, &mut found)?;
        if along {
            *surface = found.cast();
        }
        Ok(along)
    }
}
