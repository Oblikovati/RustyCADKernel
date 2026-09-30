//! `ApproxInt_ImpPrmSvSurfaces.gxx`: the SvSurfaces of an implicit (quadric) and a
//! parametric surface, which refine a point of the line with the IntImp `ZerImpFunc`
//! function (docs/geomalgo-design.md section 20). The tangents are in `compute`, the
//! singular cases in `singular`, the start point in `initial`.
mod compute;
mod initial;
mod singular;

pub use initial::PatchBounds;

use super::surface_tool::ApproxIntSurfaceTool;
use super::sv_cache::{SvCache, sv_stored_members};
use super::sv_surfaces::{ApproxIntSvComputed, ApproxIntSvSurfaces, ApproxIntSvSurfacesBase};
use crate::int_imp::IntImpZerImpFunc;
use crate::int_patch::failures::null_object;
use crate::int_surf::{IntSurfPntOn2S, IntSurfQuadric, IntSurfQuadricTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use rustycad_tkmath::math::{FunctionSetRoot, Vector};
use std::f64::consts::PI;

/// The solver tolerance on the parameters (`Tolerance(1) = Tolerance(2) = 1.0e-8`).
const SOLVER_TOL: f64 = 1.0e-8;

/// `ApproxInt_ImpPrmSvSurfaces` over the surface tool `T`
/// (GeomInt_TheImpPrmSvSurfacesOfWLApprox over Adaptor3d_HSurfaceTool): the point of the
/// intersection line of a quadric and a parametric surface near given parameters, and
/// its tangents, with the two last results kept. The implicit surface is
/// `IntSurf_Quadric` in every native instance.
///
/// ```ignore
/// let mut sv = GeomIntTheImpPrmSvSurfacesOfWLApprox::quadric_first(&quadric, &s2);
/// let point = sv.seek_point([0.5, 0.5, 0.5, 0.5])?;
/// ```
pub struct ApproxIntImpPrmSvSurfaces<T: ApproxIntSurfaceTool> {
    base: ApproxIntSvSurfacesBase,
    cache: SvCache,
    my_implicit_first: bool,
    my_zer_imp_func: IntImpZerImpFunc<T, IntSurfQuadricTool>,
}

impl<T: ApproxIntSurfaceTool> ApproxIntImpPrmSvSurfaces<T> {
    fn with_order(p_surf: &T::Surface, i_surf: &IntSurfQuadric, implicit_first: bool) -> Self {
        Self {
            base: ApproxIntSvSurfacesBase { use_solver: true },
            cache: SvCache::default(),
            my_implicit_first: implicit_first,
            my_zer_imp_func: IntImpZerImpFunc::with_surfaces(p_surf, i_surf),
        }
    }

    /// `ApproxInt_ImpPrmSvSurfaces(ISurf, PSurf)`: the quadric is the first surface; the
    /// solver is used; `ApproxIntImpPrmSvSurfaces::<T>::quadric_first(&quadric, &s2)`.
    pub fn quadric_first(i_surf: &IntSurfQuadric, p_surf: &T::Surface) -> Self {
        Self::with_order(p_surf, i_surf, true)
    }

    /// `ApproxInt_ImpPrmSvSurfaces(PSurf, ISurf)`: the quadric is the second surface;
    /// `ApproxIntImpPrmSvSurfaces::<T>::quadric_second(&s1, &quadric)`.
    pub fn quadric_second(p_surf: &T::Surface, i_surf: &IntSurfQuadric) -> Self {
        Self::with_order(p_surf, i_surf, false)
    }

    /// The parametric surface of the function (always set by the constructors).
    fn p_surface(&self) -> Result<&T::Surface, Failure> {
        self.my_zer_imp_func
            .p_surface()
            .ok_or_else(|| null_object("ApproxInt_ImpPrmSvSurfaces::PSurface", "surface"))
    }

    /// The quadric of the function (always set by the constructors).
    fn i_surface(&self) -> Result<IntSurfQuadric, Failure> {
        self.my_zer_imp_func
            .i_surface()
            .cloned()
            .ok_or_else(|| null_object("ApproxInt_ImpPrmSvSurfaces::ISurface", "quadric"))
    }

    /// The bounds of the parametric surface.
    fn bounds(&self) -> Result<PatchBounds, Failure> {
        let ps = self.p_surface()?;
        Ok(PatchBounds {
            binfu: T::first_u_parameter(ps)?,
            bsupu: T::last_u_parameter(ps)?,
            binfv: T::first_v_parameter(ps)?,
            bsupv: T::last_v_parameter(ps)?,
        })
    }

    /// The solver on the function from `x` within `b` (tolerances 1e-8); true when done,
    /// with `x` set to its root.
    fn solve(&mut self, x: &mut Vector<'_>, b: PatchBounds) -> Result<bool, Failure> {
        let tolerance = Vector::filled(1, 2, SOLVER_TOL)?;
        let mut born_inf = Vector::filled(1, 2, b.binfu)?;
        born_inf[2] = b.binfv;
        let mut born_sup = Vector::filled(1, 2, b.bsupu)?;
        born_sup[2] = b.bsupv;
        let func = &mut self.my_zer_imp_func;
        let mut rsnld = FunctionSetRoot::without_tolerance(func, 100)?;
        rsnld.set_tolerance(&tolerance)?;
        let solved = rsnld.perform_bounded(func, x, &born_inf, &born_sup, false);
        func.take_failure()?;
        solved?;
        if !rsnld.is_done() {
            return Ok(false);
        }
        rsnld.copy_root(x)?;
        Ok(true)
    }

    /// The parameters on the quadric of `p`, brought within Pi of `reference` by turns
    /// of 2 Pi (the `SeekPoint` adjustment of U).
    fn quadric_parameters(q: &IntSurfQuadric, p: Point3, reference: f64) -> (f64, f64) {
        // An undefined quadric leaves the outputs uninitialized natively; every
        // instance sets its quadric.
        let (mut u, v) = q.parameters(p).unwrap_or((0.0, 0.0));
        if q.type_quadric() != SurfaceType::Plane {
            let sign: f64 = if u > reference { -1.0 } else { 1.0 };
            while (reference - u).abs() > PI {
                u += sign * (PI + PI);
            }
        }
        (u, v)
    }
}

impl<T: ApproxIntSurfaceTool> ApproxIntSvSurfaces for ApproxIntImpPrmSvSurfaces<T> {
    fn sv_base(&self) -> &ApproxIntSvSurfacesBase {
        &self.base
    }
    fn sv_base_mut(&mut self) -> &mut ApproxIntSvSurfacesBase {
        &mut self.base
    }

    /// `Compute`: the stored result at these parameters, else the point from the
    /// solver (or from the start point without it) and the tangents.
    fn compute(
        &mut self,
        uv: &mut [f64; 4],
        out: &mut ApproxIntSvComputed,
    ) -> Result<bool, Failure> {
        self.compute_point(uv, out)
    }

    sv_stored_members!();

    /// `SeekPoint`: the solver from the parameters (whatever `UseSolver`), the point on
    /// the parametric surface and its parameters on the quadric near the given ones.
    /// The stored point changes without its parameters, as natively.
    fn seek_point(&mut self, uv: [f64; 4]) -> Result<Option<IntSurfPntOn2S>, Failure> {
        let q: IntSurfQuadric = self.i_surface()?;
        let b: PatchBounds = self.bounds()?;
        let mut x = Vector::filled(1, 2, 0.0)?;
        let Some((tu, tv)) = self.fill_initial_vector_of_solution(uv, b, &mut x)? else {
            return Ok(None);
        };
        if !self.solve(&mut x, b)? {
            return Ok(None);
        }
        self.cache.computed = true;
        let pnt: Point3 = T::value(self.p_surface()?, x[1], x[2])?;
        self.cache.cur.pnt = pnt;
        let prm: (f64, f64) = (x[1] - tu, x[2] - tv);
        let mut point = IntSurfPntOn2S::new();
        if self.my_implicit_first {
            let (u1, v1) = Self::quadric_parameters(&q, pnt, uv[0]);
            point.set_value_all(pnt, u1, v1, prm.0, prm.1);
        } else {
            let (u2, v2) = Self::quadric_parameters(&q, pnt, uv[2]);
            point.set_value_all(pnt, prm.0, prm.1, u2, v2);
        }
        Ok(Some(point))
    }
}
