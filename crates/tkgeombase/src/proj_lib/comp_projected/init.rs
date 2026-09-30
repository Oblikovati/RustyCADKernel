// Rust translation of ProjLib_CompProjectedCurve::Init (the walk along the curve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Init`: the walk along the curve. A part starts at an initial point (an
//! extremum curve-surface or the nearest normal projection) and is prolonged by
//! steps adapted to the curvature of the projection, halved where the projection
//! fails; the exact bound is searched where it leaves the surface.
use super::bounds::{dich_exact_bound, exact_bound};
use super::derivatives::{Pair, d1, d2_curv_on_surf};
use super::parts::Parts;
use super::{CompProjectedCurve, CompState, FUNC_TOL};
use crate::extrema::ExtCS;
use crate::proj_lib::{PrjResolve, PrjResolveBox};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

// `GlobalMinStep`: small enough to solve from an initial point, large enough to
// avoid too close solutions.
const GLOBAL_MIN_STEP: f64 = 1.0e-4;

/// The state of the walk of `Init`.
pub(super) struct Walk<'a> {
    pub(super) pair: Pair<'a>,
    pub(super) state: &'a CompState,
    pub(super) ext: ExtCS<'a>,
    /// `Nend`: the extrema used as start candidates (0 unless MaxDist filtered them).
    pub(super) nend: i32,
    pub(super) first_u: f64,
    pub(super) last_u: f64,
    pub(super) min_step: f64,
    pub(super) max_step: f64,
    pub(super) search_step: f64,
    pub(super) step: f64,
    pub(super) t: f64,
    pub(super) low: Point2,
    pub(super) upp: Point2,
    pub(super) prev_deb: f64,
    pub(super) same_deb: bool,
    pub(super) from_last_u: bool,
    pub(super) triple: Point3,
    pub(super) prev_triple: Point3,
    /// `Tol`: the parameter tolerance of the last exact bound search.
    pub(super) tol: f64,
    pub(super) splits: Vec<f64>,
    pub(super) split_idx: usize,
    pub(super) is_splits_computed: bool,
}

/// How the walk ended.
pub(super) enum WalkEnd {
    /// The walk covered the curve: the post-processing follows.
    Finished,
    /// A MaxDist check stopped it: the parts are cleared, no post-processing.
    Cleared,
}

impl<'a> Walk<'a> {
    /// The PrjResolve box of the free (u, v) search.
    pub(super) fn uv_box(&self) -> PrjResolveBox {
        PrjResolveBox {
            tol: Point2::new(self.state.tol_u, self.state.tol_v),
            inf: self.low,
            sup: self.upp,
        }
    }
    /// A fresh `ProjLib_PrjResolve(C, S, 1)`.
    pub(super) fn resolver(&self) -> Result<PrjResolve<'a>, Failure> {
        PrjResolve::new(self.pair.curve, self.pair.surface, 1)
    }
    /// `WalkStep`: a tenth of |C'| / |C''| of the projected curve, within
    /// `[MinStep, MaxStep]` (MaxStep when the second derivative is null).
    pub(super) fn walk_step(&self) -> Result<f64, Failure> {
        let (dd1, dd2): (Vector3, Vector3) =
            d2_curv_on_surf(self.triple.x(), self.triple.y(), self.triple.z(), self.pair)?;
        let (magn_d1, magn_d2) = (dd1.magnitude(), dd2.magnitude());
        if magn_d2 < precision::confusion() {
            return Ok(self.max_step);
        }
        Ok(self
            .max_step
            .min(self.min_step.max(0.1 * magn_d1 / magn_d2)))
    }
    /// The exact bound between the current triple and `not_sol`, by ExactBound or
    /// by dichotomy; sets `tol`.
    pub(super) fn exact_boundary(&mut self, not_sol: f64) -> Result<(), Failure> {
        let tol_uv: (f64, f64) = (self.state.tol_u, self.state.tol_v);
        let mut tol: f64 = self.state.tol_u.min(self.state.tol_v);
        let d: Vector2 = d1(self.triple.x(), self.triple.y(), self.triple.z(), self.pair)?;
        tol /= d.x().abs().max(d.y().abs());
        self.tol = tol;
        if !exact_bound(&mut self.triple, not_sol, tol, tol_uv, self.pair)? {
            dich_exact_bound(&mut self.triple, not_sol, tol, tol_uv, self.pair)?;
        }
        Ok(())
    }
}

impl CompProjectedCurve {
    /// `Init()`: computes the continuous parts of the projection, then the maximal
    /// distances, the single point and the isoparametric flags. Parts are appended
    /// to the existing ones (a second Init adds to the first, as natively);
    /// `projection.init()?`.
    pub fn init(&self) -> Result<(), Failure> {
        self.state.write().tab_int = None;
        let state: CompState = self.snapshot();
        let pair: Pair<'_> = state.pair()?;
        let mut nb_curves: i32 = state.nb_curves;
        let end: Option<WalkEnd> = run_walk(&state, pair, &mut nb_curves)?;
        self.state.write().nb_curves = nb_curves;
        let Some(WalkEnd::Finished) = end else {
            return Ok(());
        };
        self.post_process()
    }
}

// The extrema curve-surface, the MaxDist early exit, then the walk; None for the
// early exit (nothing is changed).
fn run_walk(
    state: &CompState,
    pair: Pair<'_>,
    nb_curves: &mut i32,
) -> Result<Option<WalkEnd>, Failure> {
    let tol_ext: f64 = precision::p_confusion();
    let ext = ExtCS::with_curve(pair.curve, pair.surface, tol_ext, tol_ext)?;
    let mut nend: i32 = 0;
    if ext.is_done()
        && ext.nb_ext()? != 0
        && state.max_dist > 0.
        && pair.surface.get_type()? != SurfaceType::SurfaceOfExtrusion
    {
        // The minimum solution; the extrema of an extrusion can be wrong.
        let mut min_val2: f64 = ext.square_distance(1)?;
        nend = ext.nb_ext()?;
        for i in 2..=nend {
            min_val2 = min_val2.min_native(ext.square_distance(i)?);
        }
        if min_val2 > state.max_dist * state.max_dist {
            return Ok(None); // No near solution.
        }
    }
    let mut walk: Walk<'_> = Walk::new(state, pair, ext, nend)?;
    Ok(Some(walk.run(nb_curves)?))
}

trait NativeMin {
    fn min_native(self, other: f64) -> f64;
}
impl NativeMin for f64 {
    // `if (x < min) min = x`.
    fn min_native(self, other: f64) -> f64 {
        if other < self { other } else { self }
    }
}

impl<'a> Walk<'a> {
    fn new(
        state: &'a CompState,
        pair: Pair<'a>,
        ext: ExtCS<'a>,
        nend: i32,
    ) -> Result<Self, Failure> {
        let (first_u, last_u) = (pair.curve.first_parameter()?, pair.curve.last_parameter()?);
        let (min_step, max_step) = (0.01 * (last_u - first_u), 0.1 * (last_u - first_u));
        let s = pair.surface;
        Ok(Self {
            pair,
            state,
            ext,
            nend,
            first_u,
            last_u,
            min_step,
            max_step,
            search_step: 10. * min_step,
            step: 10. * min_step,
            t: first_u,
            low: Point2::new(s.first_u_parameter()?, s.first_v_parameter()?),
            upp: Point2::new(s.last_u_parameter()?, s.last_v_parameter()?),
            prev_deb: 0.,
            same_deb: false,
            from_last_u: false,
            triple: Point3::default(),
            prev_triple: Point3::default(),
            tol: 0.,
            splits: Vec::new(),
            split_idx: 0,
            is_splits_computed: false,
        })
    }

    // The basic loop: search the start of a part, record it, prolong it.
    fn run(&mut self, nb_curves: &mut i32) -> Result<WalkEnd, Failure> {
        let parts = self.state.parts()?;
        while self.t <= self.last_u {
            // Search for the beginning of a new continuous part, avoiding an infinite
            // computation in some difficult cases.
            if self.t > self.first_u && (self.t - self.prev_deb).abs() <= precision::p_confusion() {
                self.same_deb = true;
            }
            match self.search_start()? {
                super::start::StartSearch::Found => {}
                super::start::StartSearch::NotFound => break,
                super::start::StartSearch::Cleared => {
                    parts.clear_parts();
                    *nb_curves = 0;
                    return Ok(WalkEnd::Cleared);
                }
            }
            // A new continuous part.
            parts.push_part();
            *nb_curves += 1;
            parts.push_point(*nb_curves, self.triple)?;
            self.prev_triple = self.triple;
            if self.triple.x() == self.last_u {
                break;
            }
            self.prolong(*nb_curves)?;
        }
        Ok(WalkEnd::Finished)
    }

    // Appends the triple to part `index` when it is past its last one.
    fn append_triple(&self, index: i32) -> Result<(), Failure> {
        let parts = self.state.parts()?;
        if (self.triple.x() - parts.last_point(index)?.x()) > 1.0e-10 {
            parts.push_point(index, self.triple)?;
        }
        Ok(())
    }

    // Prolongs the current part while the projection exists.
    fn prolong(&mut self, index: i32) -> Result<(), Failure> {
        self.step = self.walk_step()?;
        self.t = self.triple.x() + self.step;
        if self.t > self.last_u {
            self.t = self.last_u;
        }
        let mut prev_step: f64 = self.step;
        let mut new_part: bool = true;
        while self.t <= self.last_u && new_part {
            let ratio: f64 = self.step / prev_step;
            let mut u0: f64 = self.triple.y() + ratio * (self.triple.y() - self.prev_triple.y());
            let mut v0: f64 = self.triple.z() + ratio * (self.triple.z() - self.prev_triple.z());
            // Keep (U0, V0) in the surface bounds.
            u0 = u0.max(self.low.x()).min(self.upp.x());
            v0 = v0.max(self.low.y()).min(self.upp.y());
            let mut resolve: PrjResolve<'_> = self.resolver()?;
            resolve.perform(self.t, u0, v0, self.uv_box(), FUNC_TOL, true)?;
            if !resolve.is_done() {
                new_part = self.shrink_step(index)?;
                continue;
            }
            prev_step = self.step;
            let solution: Point2 = resolve.solution()?;
            if self.advance(index, solution)? {
                break;
            }
        }
        Ok(())
    }

    // The projection failed: at the minimal step, the exact bound ends the part
    // (returns false); otherwise the step is halved (returns true).
    fn shrink_step(&mut self, index: i32) -> Result<bool, Failure> {
        if self.step <= GLOBAL_MIN_STEP {
            self.exact_boundary(self.t)?;
            self.append_triple(index)?;
            if (self.last_u - self.triple.x()) < self.tol {
                self.t = self.last_u + 1.;
                return Ok(true);
            }
            self.step = self.search_step;
            self.t = self.triple.x() + self.step;
            if self.t > (self.last_u - self.min_step / 2.) {
                self.step = self.step + self.last_u - self.t;
                self.t = self.last_u;
            }
            return Ok(false);
        }
        let save_step: f64 = self.step;
        self.step /= 2.;
        self.t = self.triple.x() + self.step;
        if self.t > (self.last_u - self.min_step / 4.) {
            self.step = self.step + self.last_u - self.t;
            if (self.step - save_step).abs() <= precision::p_confusion() {
                self.step = GLOBAL_MIN_STEP; // To avoid looping.
            }
            self.t = self.last_u;
        }
        Ok(true)
    }
}
