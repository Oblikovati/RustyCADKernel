// Rust translation of IntPatch_Intersection::GeomGeomPerfom and GeomParamPerfom
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGeomPerfom` (two quadrics: the analytic lines become walking lines, which are
//! joined and extended) and `GeomParamPerfom` (a quadric and a parametric surface).
use super::IntPatchIntersection;
use super::infinite::trim_both_surf;
use super::perform::Pair;
use crate::int_patch::{
    IntPatchALine, IntPatchALineToWLine, IntPatchIType, IntPatchImpImpIntersection,
    IntPatchImpImpStatus, IntPatchImpPrmIntersection, IntPatchLine, IntPatchWLine,
    IntPatchWLineTool,
};
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::FRAC_PI_2;

/// The number of points of the walking line of an analytic line.
const NB_POINTS_IN_ALINE: i32 = 200;

/// The box of the parameter bounds of a surface, enlarged by PConfusion.
fn bounds_box(s: &Adaptor3dSurfaceHandle) -> Result<BoundingBox2, Failure> {
    let (uf, ul) = (s.first_u_parameter()?, s.last_u_parameter()?);
    let (vf, vl) = (s.first_v_parameter()?, s.last_v_parameter()?);
    let mut b = BoundingBox2::default();
    for (u, v) in [(uf, vf), (ul, vf), (ul, vl), (uf, vl)] {
        b.add_point(Point2::new(u, v));
    }
    b.enlarge(precision::p_confusion());
    Ok(b)
}

/// The period of a surface in a direction (0 when not periodic).
fn period(s: &Adaptor3dSurfaceHandle, u: bool) -> Result<f64, Failure> {
    match u {
        true if s.is_u_periodic()? => s.u_period(),
        false if s.is_v_periodic()? => s.v_period(),
        _ => Ok(0.0),
    }
}

/// The critical points of a quadric: the apex of a cone, the poles of a sphere.
fn critical_points(s: &Adaptor3dSurfaceHandle, points: &mut Vec<Point3>) -> Result<(), Failure> {
    match s.get_type()? {
        SurfaceType::Cone => points.push(s.cone()?.apex()),
        SurfaceType::Sphere => {
            points.push(s.value(0.0, FRAC_PI_2)?);
            points.push(s.value(0.0, -FRAC_PI_2)?);
        }
        _ => {}
    }
    Ok(())
}

impl IntPatchIntersection {
    /// The lines of the implicit-implicit intersection: the analytic lines become
    /// walking lines, the walking lines are not purged, the restrictions are kept only
    /// when asked; returns whether a walking line was made.
    fn take_imp_imp_lines(
        &mut self,
        pair: Pair<'_>,
        interii: &IntPatchImpImpIntersection,
        keep_rline: bool,
    ) -> Result<bool, Failure> {
        let mut is_wl_exist = false;
        let a_to_w = IntPatchALineToWLine::new(pair.s1, pair.s2, NB_POINTS_IN_ALINE)?;
        for i in 1..=interii.nb_lines()? {
            let line: Handle<dyn IntPatchLine> = interii.line(i)?.clone();
            if line.arc_type() == IntPatchIType::Analytic {
                is_wl_exist = true;
                a_to_w.make_wline(&line.cast::<IntPatchALine>(), &mut self.slin)?;
                continue;
            }
            if line.arc_type() == IntPatchIType::Walking {
                line.cast::<IntPatchWLine>().enable_purging(false);
            }
            if line.arc_type() != IntPatchIType::Restriction || keep_rline {
                self.slin.push(line);
            }
        }
        for i in 1..=interii.nb_pnts()? {
            self.spnt.push(interii.point(i)?.clone());
        }
        Ok(is_wl_exist)
    }

    /// The extension of the walking lines of two quadrics (`ExtendTwoWLines` with the
    /// boxes of the surface bounds, the periods and the apexes and poles).
    fn extend_quadric_lines(&mut self, pair: Pair<'_>, tol_tang: f64) -> Result<(), Failure> {
        let (bx1, bx2) = (bounds_box(pair.s1)?, bounds_box(pair.s2)?);
        let periods: [f64; 4] = [
            period(pair.s1, true)?,
            period(pair.s1, false)?,
            period(pair.s2, true)?,
            period(pair.s2, false)?,
        ];
        let mut critical: Vec<Point3> = Vec::new();
        critical_points(pair.s1, &mut critical)?;
        critical_points(pair.s2, &mut critical)?;
        IntPatchWLineTool::extend_two_wlines(
            &mut self.slin,
            pair.s1,
            pair.s2,
            tol_tang,
            &periods,
            &bx1,
            &bx2,
            &critical,
        )
    }

    /// `GeomGeomPerfom(S1, D1, S2, D2, TolArc, TolTang, ListOfPnts, typs1, typs2,
    /// theIsReqToKeepRLine)`.
    pub(super) fn geom_geom_perform(
        &mut self,
        pair: Pair<'_>,
        tols: (f64, f64),
        list_of_pnts: &mut Vec<IntSurfPntOn2S>,
        typs: (SurfaceType, SurfaceType),
        keep_rline: bool,
    ) -> Result<(), Failure> {
        let interii = IntPatchImpImpIntersection::with_surfaces(
            pair.s1,
            pair.d1,
            pair.s2,
            pair.d2,
            self.my_tol_arc,
            self.my_tol_tang,
            keep_rline,
        )?;
        if !interii.is_done() {
            self.done = false;
            return self.param_param_perform(pair, tols, list_of_pnts, typs);
        }
        self.done = interii.get_status() == IntPatchImpImpStatus::Ok;
        self.empt = interii.is_empty()?;
        if self.empt {
            return Ok(());
        }
        self.tgte = interii.tangent_faces()?;
        if self.tgte {
            self.oppo = interii.opposite_faces()?;
        }
        let is_wl_exist: bool = self.take_imp_imp_lines(pair, &interii, keep_rline)?;
        if typs.0 == SurfaceType::Cylinder && typs.1 == SurfaceType::Cylinder {
            IntPatchWLineTool::join_wlines(
                &mut self.slin,
                &mut self.spnt,
                pair.s1,
                pair.s2,
                tols.1,
            )?;
        }
        if is_wl_exist {
            self.extend_quadric_lines(pair, tols.1)?;
        }
        Ok(())
    }

    /// `GeomParamPerfom(S1, D1, S2, D2, isNotAnalitical, typs1, typs2)`: the
    /// implicit-parametric intersection (on trimmed surfaces for two infinite domains,
    /// unless their isolines give the lines).
    pub(super) fn geom_param_perform(
        &mut self,
        pair: Pair<'_>,
        is_not_analitical: bool,
        typs: (SurfaceType, SurfaceType),
    ) -> Result<(), Failure> {
        let mut interip = IntPatchImpPrmIntersection::new();
        if self.my_is_start_pnt {
            let [u1, v1, u2, v2] = self.my_start;
            match is_not_analitical {
                true => interip.set_start_point(u1, v1),
                false => interip.set_start_point(u2, v2),
            }
        }
        let steps = (
            self.my_tol_arc,
            self.my_tol_tang,
            self.my_fleche,
            self.my_uv_max_step,
        );
        if pair.d1.domain_is_infinite() && pair.d2.domain_is_infinite() {
            if self.collinear_lines(pair, typs)? {
                return Ok(());
            }
            let (mut ns1, mut ns2) = (pair.s1.clone(), pair.s2.clone());
            trim_both_surf(
                (pair.s1, typs.0),
                (pair.s2, typs.1),
                1.0e+5,
                (&mut ns1, &mut ns2),
            )?;
            interip.perform(
                &ns1, pair.d1, &ns2, pair.d2, steps.0, steps.1, steps.2, steps.3,
            )?;
        } else {
            interip.perform(
                pair.s1, pair.d1, pair.s2, pair.d2, steps.0, steps.1, steps.2, steps.3,
            )?;
        }
        if !interip.is_done() {
            return Ok(());
        }
        self.done = true;
        self.empt = interip.is_empty()?;
        if self.empt {
            return Ok(());
        }
        let nb_lines: i32 = interip.nb_lines()?;
        let lines: Vec<Handle<dyn IntPatchLine>> = (1..=nb_lines)
            .map(|i| interip.line(i).cloned())
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
        for i in 1..=interip.nb_pnts()? {
            self.spnt.push(interip.point(i)?.clone());
        }
        Ok(())
    }
}
