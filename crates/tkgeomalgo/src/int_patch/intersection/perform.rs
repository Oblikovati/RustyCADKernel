// Rust translation of the IntPatch_Intersection constructors and the Perform overloads of two
// surfaces (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Perform` overloads of two surfaces: the classification of the pair, the
//! dispatch to the intersectors and the purge of the walking lines.
use super::IntPatchIntersection;
use super::classify::{PairKind, Variant, classify};
use crate::int_patch::{IntPatchWLine, IntPatchWLineTool};
use crate::int_start::TheTopolTool;
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// The surfaces and domains of an intersection.
#[derive(Clone, Copy)]
pub(super) struct Pair<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) d1: &'a (dyn TheTopolTool + 'static),
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) d2: &'a (dyn TheTopolTool + 'static),
}

/// The options of the `Perform` overloads of two surfaces (`isGeomInt`,
/// `theIsReqToKeepRLine`, `theIsReqToPostWLProc`; true, false, true natively by
/// default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntPatchIntersectionOptions {
    /// `isGeomInt`: the implicit-implicit intersector is used for two quadrics.
    pub is_geom_int: bool,
    /// `theIsReqToKeepRLine`: the restriction lines of two quadrics are kept.
    pub is_req_to_keep_rline: bool,
    /// `theIsReqToPostWLProc`: the walking lines are purged.
    pub is_req_to_post_wl_proc: bool,
}

impl Default for IntPatchIntersectionOptions {
    fn default() -> Self {
        Self {
            is_geom_int: true,
            is_req_to_keep_rline: false,
            is_req_to_post_wl_proc: true,
        }
    }
}

impl IntPatchIntersection {
    /// `IntPatch_Intersection(S1, D1, S2, D2, TolArc, TolTang)`: the clamped tolerances
    /// are overwritten by `Perform` with the given ones, as natively.
    ///
    /// ```ignore
    /// let inter = IntPatchIntersection::with_surfaces(&s1, &*d1, &s2, &*d2, 1e-7, 1e-7)?;
    /// ```
    pub fn with_surfaces(
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        tol_arc: f64,
        tol_tang: f64,
    ) -> Result<Self, Failure> {
        let mut inter = Self::new();
        inter.my_tol_arc = super::clamp_tolerance(tol_arc);
        inter.my_tol_tang = super::clamp_tolerance(tol_tang);
        inter.perform(
            s1,
            d1,
            s2,
            d2,
            (tol_arc, tol_tang),
            IntPatchIntersectionOptions::default(),
        )?;
        Ok(inter)
    }

    /// The fields reset at the start of `Perform` (the deflection and the step get their
    /// defaults when they are not above PConfusion).
    fn reset(&mut self, (tol_arc, tol_tang): (f64, f64)) {
        self.my_tol_arc = tol_arc;
        self.my_tol_tang = tol_tang;
        if self.my_fleche <= precision::p_confusion() {
            self.my_fleche = 0.01;
        }
        if self.my_uv_max_step <= precision::p_confusion() {
            self.my_uv_max_step = 0.01;
        }
        self.done = false;
        self.spnt.clear();
        self.slin.clear();
        self.empt = true;
        self.tgte = false;
        self.oppo = false;
    }

    /// `Perform(S1, D1, S2, D2, TolArc, TolTang, isGeomInt, theIsReqToKeepRLine,
    /// theIsReqToPostWLProc)`: the intersection lines of the two surfaces, from the
    /// implicit-implicit, implicit-parametric or parametric-parametric intersector by the
    /// kinds of the surfaces; the walking lines are purged unless told otherwise.
    ///
    /// ```ignore
    /// inter.perform(&s1, &*d1, &s2, &*d2, (1e-7, 1e-7), IntPatchIntersectionOptions::default())?;
    /// ```
    pub fn perform(
        &mut self,
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        (tol_arc, tol_tang): (f64, f64),
        options: IntPatchIntersectionOptions,
    ) -> Result<(), Failure> {
        self.reset((tol_arc, tol_tang));
        let pair = Pair { s1, d1, s2, d2 };
        let kind: PairKind = classify((s1, d1), (s2, d2), tol_tang, Variant::Plain)?;
        let (ts1, ts2) = kind.implicit();
        let tols = (tol_arc, tol_tang);
        // Geom - Geom.
        if ts1 == ts2 && ts1 {
            let mut list: Vec<IntSurfPntOn2S> = Vec::new();
            if options.is_geom_int {
                self.geom_geom_perform(
                    pair,
                    tols,
                    &mut list,
                    kind.typs,
                    options.is_req_to_keep_rline,
                )?;
            } else {
                self.param_param_perform(pair, tols, &mut list, kind.typs)?;
            }
        }
        // Geom - Param.
        if ts1 != ts2 {
            self.geom_param_perform(pair, !ts1, kind.typs)?;
        }
        // Param - Param.
        if ts1 == ts2 && !ts1 {
            let mut list: Vec<IntSurfPntOn2S> = Vec::new();
            self.param_param_perform(pair, tols, &mut list, kind.typs)?;
        }
        if options.is_req_to_post_wl_proc {
            self.purge_wlines(pair)?;
        }
        Ok(())
    }

    /// `Perform(S1, D1, S2, D2, TolArc, TolTang, LOfPnts, isGeomInt, theIsReqToKeepRLine,
    /// theIsReqToPostWLProc)`: the same with start points for the parametric
    /// intersection (a pair treated as bi-parametric always goes to the
    /// parametric-parametric intersector).
    ///
    /// ```ignore
    /// inter.perform_with_points(&s1, &*d1, &s2, &*d2, (1e-7, 1e-7), &mut points, options)?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn perform_with_points(
        &mut self,
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        (tol_arc, tol_tang): (f64, f64),
        list_of_pnts: &mut Vec<IntSurfPntOn2S>,
        options: IntPatchIntersectionOptions,
    ) -> Result<(), Failure> {
        self.reset((tol_arc, tol_tang));
        let pair = Pair { s1, d1, s2, d2 };
        let kind: PairKind = classify((s1, d1), (s2, d2), tol_tang, Variant::WithPoints)?;
        let (ts1, ts2) = kind.implicit();
        let tols = (tol_arc, tol_tang);
        if !options.is_geom_int {
            self.param_param_perform(pair, tols, list_of_pnts, kind.typs)?;
        } else if ts1 != ts2 {
            self.geom_param_perform(pair, !ts1, kind.typs)?;
        } else if !ts1 {
            self.param_param_perform(pair, tols, list_of_pnts, kind.typs)?;
        } else {
            self.geom_geom_perform(
                pair,
                tols,
                list_of_pnts,
                kind.typs,
                options.is_req_to_keep_rline,
            )?;
        }
        if options.is_req_to_post_wl_proc {
            self.purge_wlines(pair)?;
        }
        Ok(())
    }

    /// Each walking line that allows it is replaced by its purged line (when there is
    /// one).
    pub(super) fn purge_wlines(&mut self, pair: Pair<'_>) -> Result<(), Failure> {
        for i in 0..self.slin.len() {
            let wl: Handle<IntPatchWLine> = self.slin[i].cast::<IntPatchWLine>();
            if wl.is_null() || !wl.is_purging_allowed() {
                continue;
            }
            let rw =
                IntPatchWLineTool::compute_purged_wline(&wl, pair.s1, pair.s2, pair.d1, pair.d2)?;
            if rw.is_null() {
                continue;
            }
            self.slin[i] = rw.cast();
        }
        Ok(())
    }
}
