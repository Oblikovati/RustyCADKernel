// Rust translation of IntPatch_Intersection::ParamParamPerfom
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ParamParamPerfom`: the parametric-parametric intersection, after the trimming of the
//! infinite surfaces, or the lines of two infinite surfaces with parallel straight
//! isolines.
use super::IntPatchIntersection;
use super::infinite::{get_min_max_xyz_pnt, trim_both_surf, trim_inf_surf};
use super::perform::Pair;
use super::pl_inter::{PlSide, pl_intersection};
use crate::int_patch::{
    IntPatchGLine, IntPatchIType, IntPatchLine, IntPatchLineTransitions,
    IntPatchPrmPrmIntersection, IntPatchPrmPrmTolerances,
};
use crate::int_surf::IntSurfPntOn2S;
use crate::intf::std_max;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Line3};

impl IntPatchIntersection {
    /// The marching tolerances of the parametric intersector (the tangency tolerance
    /// first, as natively).
    pub(super) fn prm_tolerances(
        &self,
        (tol_arc, tol_tang): (f64, f64),
    ) -> IntPatchPrmPrmTolerances {
        IntPatchPrmPrmTolerances {
            tol_tangency: tol_tang,
            epsilon: tol_arc,
            deflection: self.my_fleche,
            increment: self.my_uv_max_step,
        }
    }

    /// The largest parameter of the bounds of a surface.
    fn largest_parameter(s: &Adaptor3dSurfaceHandle) -> Result<f64, Failure> {
        let mu: f64 = std_max(s.first_u_parameter()?.abs(), s.last_u_parameter()?.abs());
        let mv: f64 = std_max(s.first_v_parameter()?.abs(), s.last_v_parameter()?.abs());
        Ok(std_max(mu, mv))
    }

    /// The infinite surface trimmed around the finite one.
    fn trimmed_infinite(
        finite: &Adaptor3dSurfaceHandle,
        infinite: &Adaptor3dSurfaceHandle,
    ) -> Result<Adaptor3dSurfaceHandle, Failure> {
        let (pmin, pmax) = get_min_max_xyz_pnt(finite)?;
        let ap: f64 = Self::largest_parameter(finite)?;
        trim_inf_surf((pmin, pmax), infinite, ap)
    }

    /// The lines of two infinite surfaces with parallel straight isolines (`None` when the
    /// case does not apply); `done` is set by the number of lines.
    pub(super) fn collinear_lines(
        &mut self,
        pair: Pair<'_>,
        typs: (SurfaceType, SurfaceType),
    ) -> Result<bool, Failure> {
        let side1 = PlSide {
            s: pair.s1,
            typ: typs.0,
        };
        let side2 = PlSide {
            s: pair.s2,
            typ: typs.1,
        };
        let pl = pl_intersection(&side1, &side2)?;
        if !pl.is_ok {
            return Ok(false);
        }
        if pl.points.is_empty() {
            self.done = false;
            return Ok(true);
        }
        let dir = Direction3::from_vector(pl.direction)?;
        for p in pl.points {
            let gl = IntPatchGLine::from_line(
                &Line3::new(p, dir),
                false,
                IntPatchLineTransitions::Undecided,
            )?;
            self.slin.push(Handle::new(gl).cast());
        }
        self.done = true;
        Ok(true)
    }

    /// `ParamParamPerfom(S1, D1, S2, D2, TolArc, TolTang, ListOfPnts, typs1, typs2)`.
    pub(super) fn param_param_perform(
        &mut self,
        pair: Pair<'_>,
        tols: (f64, f64),
        list_of_pnts: &mut Vec<IntSurfPntOn2S>,
        typs: (SurfaceType, SurfaceType),
    ) -> Result<(), Failure> {
        let mut interpp = IntPatchPrmPrmIntersection::new();
        let t = self.prm_tolerances(tols);
        let (inf1, inf2) = (pair.d1.domain_is_infinite(), pair.d2.domain_is_infinite());
        let (s1, s2) = ((pair.s1, pair.d1), (pair.s2, pair.d2));
        if !inf1 && !inf2 {
            let mut clear_flag = true;
            if !list_of_pnts.is_empty() {
                interpp.perform_with_points(s1, s2, t, list_of_pnts)?;
                clear_flag = false;
            }
            interpp.perform(
                pair.s1,
                pair.d1,
                pair.s2,
                pair.d2,
                t.tol_tangency,
                t.epsilon,
                t.deflection,
                t.increment,
                clear_flag,
            )?;
        } else if inf1 ^ inf2 {
            let (a, b) = if inf1 {
                (Self::trimmed_infinite(pair.s2, pair.s1)?, pair.s2.clone())
            } else {
                (pair.s1.clone(), Self::trimmed_infinite(pair.s1, pair.s2)?)
            };
            interpp.perform(
                &a,
                pair.d1,
                &b,
                pair.d2,
                t.tol_tangency,
                t.epsilon,
                t.deflection,
                t.increment,
                true,
            )?;
        } else {
            if typs.0 == SurfaceType::OtherSurface || typs.1 == SurfaceType::OtherSurface {
                self.done = false;
                return Ok(());
            }
            // 'COLLINEAR LINES'.
            if self.collinear_lines(pair, typs)? {
                return Ok(());
            }
            // 'NON - COLLINEAR LINES'.
            let (mut ns1, mut ns2) = (pair.s1.clone(), pair.s2.clone());
            trim_both_surf(
                (pair.s1, typs.0),
                (pair.s2, typs.1),
                1.0e+8,
                (&mut ns1, &mut ns2),
            )?;
            interpp.perform(
                &ns1,
                pair.d1,
                &ns2,
                pair.d2,
                t.tol_tangency,
                t.epsilon,
                t.deflection,
                t.increment,
                true,
            )?;
        }
        self.take_prm_lines(&interpp)
    }

    /// The lines of a done parametric intersection: the lines of other kinds, then the
    /// walking lines.
    fn take_prm_lines(&mut self, interpp: &IntPatchPrmPrmIntersection) -> Result<(), Failure> {
        if !interpp.is_done() {
            return Ok(());
        }
        self.done = true;
        self.tgte = false;
        self.empt = interpp.is_empty()?;
        let lines: Vec<Handle<dyn IntPatchLine>> = (1..=interpp.nb_lines()?)
            .map(|i| interpp.line(i))
            .collect::<Result<_, _>>()?;
        self.slin.extend(
            lines
                .iter()
                .filter(|l| l.arc_type() != IntPatchIType::Walking)
                .cloned(),
        );
        self.slin.extend(
            lines
                .iter()
                .filter(|l| l.arc_type() == IntPatchIType::Walking)
                .cloned(),
        );
        Ok(())
    }
}
