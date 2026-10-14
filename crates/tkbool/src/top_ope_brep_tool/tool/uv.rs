//! TopOpeBRepTool_TOOL.cxx: UV points, tangents and translations of p-curves,
//! parameters of UV points on iso edges, the position of a UV point on the bounds of
//! a face and the UV tolerances.
use super::TopOpeBRepToolTOOL;
use super::{TOOL_INFFIRST, TOOL_ONFIRST, TOOL_ONLAST, TOOL_SUPLAST};
use crate::top_ope_brep_tool::c2df::TopOpeBRepToolC2DF;
use crate::top_ope_brep_tool::geometry::{fun_tool_line2d, fun_tool_quad2d};
use crate::top_ope_brep_tool::normal::min_duv;
use crate::top_ope_brep_tool::project::fun_tool_proj_ponf;
use crate::top_ope_brep_tool::shape_tool::TopOpeBRepToolShapeTool;
use crate::top_ope_brep_tool::tool_iso::{uviso, uviso_edge};
use crate::top_ope_brep_tool::topology::{
    fun_tool_getdxx, fun_tool_paron_ef, fun_tool_value_on_face,
};
use crate::top_ope_brep_tool::two_d::{
    fc2d_curve_on_surface, fc2d_editable_curve_on_surface, fc2d_has_old_curve_on_surface,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2, Vector3};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// Standard_NullObject of a null p-curve (a null dereference natively).
fn null_pcurve(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("TopOpeBRepTool_TOOL::{member}: null p-curve; expected a 2d curve"),
        "TopOpeBRepTool_TOOL",
    )
}

impl TopOpeBRepToolTOOL {
    /// `UVF(par, C2DF)`: the point of the p-curve at a parameter;
    /// `TopOpeBRepToolTOOL::uvf(0.5, &c2df)?`.
    pub fn uvf(par: f64, c2df: &TopOpeBRepToolC2DF) -> Result<Point2, Failure> {
        let (pc, _, _, _) = c2df.pc();
        pc.get().ok_or_else(|| null_pcurve("UVF"))?.value(par)
    }

    /// `ParISO(uv, E, F, par)`: for an iso edge, the parameter of the UV point along the
    /// iso direction (1e7 and false otherwise); `TopOpeBRepToolTOOL::par_iso(uv, &e, &f, &mut par)?`.
    pub fn par_iso(uv: Point2, e: &Edge, f: &Face, par: &mut f64) -> Result<bool, Failure> {
        *par = 1.0e7;
        let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
        let (iso, isou, isov) = uviso_edge(e, f, &mut d2d, &mut o2d)?;
        if !iso {
            return Ok(false);
        }
        if isou {
            *par = uv.y() - o2d.y();
            if d2d.y() < 0. {
                *par = -*par;
            }
        }
        if isov {
            *par = uv.x() - o2d.x();
            if d2d.x() < 0. {
                *par = -*par;
            }
        }
        Ok(true)
    }

    /// `ParE2d(p2d, E, F, par, dist)`: the parameter of the UV point on the p-curve of
    /// the edge and its distance, read directly on an iso line, else by
    /// Geom2dAPI_ProjectPointOnCurve; `TopOpeBRepToolTOOL::par_e2d(uv, &e, &f)?`.
    pub fn par_e2d(p2d: Point2, e: &Edge, f: &Face) -> Result<(f64, f64), Failure> {
        let bc2d = BRepAdaptorCurve2d::with_edge_on_face(e, f)?;
        let c2d = bc2d.geom2d_adaptor().curve();
        if bc2d.get_type()? == CurveType::Line {
            let (mut dir2d, mut loc) = (Direction2::X, Point2::default());
            let (_, iso_u, iso_v) = uviso(&c2d, &mut dir2d, &mut loc)?;
            let (mut par, mut dist) = (0., 0.);
            if iso_u {
                (par, dist) = (p2d.y() - loc.y(), (p2d.x() - loc.x()).abs());
            }
            if iso_v {
                (par, dist) = (p2d.x() - loc.x(), (p2d.y() - loc.y()).abs());
            }
            if iso_u || iso_v {
                return Ok((par, dist));
            }
        }
        let proj = ProjectPointOnCurve::with_curve(p2d, &c2d)?;
        Ok((
            proj.lower_distance_parameter()?,
            p2d.distance(proj.nearest_point()?),
        ))
    }

    /// `Tg2d(iv, E, C2DF)`: the unit UV tangent of the p-curve at the vertex `iv` of
    /// the edge; `TopOpeBRepToolTOOL::tg2d(1, &e, &c2df)?`.
    pub fn tg2d(iv: i32, e: &Edge, c2df: &TopOpeBRepToolC2DF) -> Result<Vector2, Failure> {
        let (pc, _, _, _) = c2df.pc();
        let par = Self::par_e(iv, e)?;
        let (_, tg2d) = pc.get().ok_or_else(|| null_pcurve("Tg2d"))?.d1(par)?;
        Ok(Vector2::from(Direction2::from_xy(tg2d.xy())?))
    }

    /// `Tg2dApp(iv, E, C2DF, factor)`: the unit UV tangent of the p-curve at `factor`
    /// of the way from the vertex `iv` to the other one;
    /// `TopOpeBRepToolTOOL::tg2d_app(1, &e, &c2df, 0.1)?`.
    pub fn tg2d_app(
        iv: i32,
        e: &Edge,
        c2df: &TopOpeBRepToolC2DF,
        factor: f64,
    ) -> Result<Vector2, Failure> {
        let (pc, _, _, _) = c2df.pc();
        let i_oov = if iv == 1 { 2 } else { 1 };
        let par = Self::par_e(iv, e)?;
        let oopar = Self::par_e(i_oov, e)?;
        let par_e = (1. - factor) * par + factor * oopar;
        let (_, tg2d) = pc.get().ok_or_else(|| null_pcurve("Tg2dApp"))?.d1(par_e)?;
        Ok(Vector2::from(Direction2::from_xy(tg2d.xy())?))
    }

    /// `tryTg2dApp(iv, E, C2DF, factor)`: `Tg2dApp` on a non-line conic p-curve, `Tg2d`
    /// otherwise; `TopOpeBRepToolTOOL::try_tg2d_app(1, &e, &c2df, 0.1)?`.
    pub fn try_tg2d_app(
        iv: i32,
        e: &Edge,
        c2df: &TopOpeBRepToolC2DF,
        factor: f64,
    ) -> Result<Vector2, Failure> {
        let (pc, _, _, _) = c2df.pc();
        let isquad = fun_tool_quad2d(&pc)?;
        let line = fun_tool_line2d(&pc)?;
        if !isquad || line {
            return Self::tg2d(iv, e, c2df);
        }
        Self::tg2d_app(iv, e, c2df, factor)
    }

    /// `tryOriEinF(par, e, f)`: the orientation code of the edge in the face (with the
    /// closing check), else from the UV point beside the p-curve at a tenth of the
    /// smallest UV span of the face (2 when it falls out of the face bounds, 1 inside);
    /// 0 without stored p-curve; `TopOpeBRepToolTOOL::try_ori_e_in_f(0.5, &e, &f)?`.
    pub fn try_ori_e_in_f(par: f64, e: &Edge, f: &Face) -> Result<i32, Failure> {
        let oeinf = Self::oriin_sor(e, f, true)?;
        if oeinf != 0 {
            return Ok(oeinf);
        }
        if fc2d_has_old_curve_on_surface(e, f)?.is_none() {
            return Ok(0);
        }
        let (pc, _, _, _) = fc2d_editable_curve_on_surface(e, f, false)?;
        let (uv, tg2d) = pc.get().ok_or_else(|| null_pcurve("tryOriEinF"))?.d1(par)?;
        let n2d = Vector2::from(Direction2::new(-tg2d.y(), tg2d.x())?);
        let delta = min_duv(f)? * 1.0e-1;
        let ouv = uv.translated(n2d * (delta));
        Ok(if Self::out_uv_bounds(ouv, f)? { 2 } else { 1 })
    }

    /// `IsonCLO(PC, onU, xfirst, xperiod, xtol)`: whether the p-curve is the iso in the
    /// direction asked on the first bound or one period further (within `xtol`);
    /// `TopOpeBRepToolTOOL::ison_clo(&pc, true, 0., 2. * PI, 1e-9)?`.
    pub fn ison_clo(
        pc: &Handle<dyn Geom2dCurve>,
        on_u: bool,
        xfirst: f64,
        xperiod: f64,
        xtol: f64,
    ) -> Result<bool, Failure> {
        let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
        let (isouv, isou, isov) = uviso(pc, &mut d2d, &mut o2d)?;
        if !isouv || !((on_u && isou) || (!on_u && isov)) {
            return Ok(false);
        }
        let dxx = if on_u {
            (o2d.x() - xfirst).abs()
        } else {
            (o2d.y() - xfirst).abs()
        };
        Ok(dxx < xtol || (xperiod - dxx).abs() < xtol)
    }

    /// `IsonCLO(C2DF, onU, xfirst, xperiod, xtol)`: on the p-curve of the C2DF;
    /// `TopOpeBRepToolTOOL::ison_clo_c2df(&c2df, true, 0., 2. * PI, 1e-9)?`.
    pub fn ison_clo_c2df(
        c2df: &TopOpeBRepToolC2DF,
        on_u: bool,
        xfirst: f64,
        xperiod: f64,
        xtol: f64,
    ) -> Result<bool, Failure> {
        let (pc, _, _, _) = c2df.pc();
        Self::ison_clo(&pc, on_u, xfirst, xperiod, xtol)
    }

    /// `TrslUV(t2d, C2DF)`: the p-curve of the C2DF translated in place;
    /// `TopOpeBRepToolTOOL::trsl_uv(t, &mut c2df)?`.
    pub fn trsl_uv(t2d: Vector2, c2df: &mut TopOpeBRepToolC2DF) -> Result<(), Failure> {
        let (pc, f, l, tol) = c2df.pc();
        pc.get()
            .ok_or_else(|| null_pcurve("TrslUV"))?
            .translate(t2d)?;
        c2df.set_pc(&pc, f, l, tol);
        Ok(())
    }

    /// `TrslUVModifE(t2d, F, E)`: the p-curve of the edge on the face translated and
    /// stored on the edge; false without p-curve;
    /// `TopOpeBRepToolTOOL::trsl_uv_modif_e(t, &f, &e)?`.
    pub fn trsl_uv_modif_e(t2d: Vector2, f: &Face, e: &Edge) -> Result<bool, Failure> {
        let (pc, _, _, tol) = fc2d_curve_on_surface(e, f, false)?;
        let Some(curve) = pc.get() else {
            return Ok(false);
        };
        curve.translate(t2d)?;
        BRepBuilder::default().update_edge_pcurve_on_face(e, &pc, f, tol)?;
        Ok(true)
    }

    /// The UV step of a periodic direction brought back within half a period.
    fn periodic_step(x1: f64, x2: f64, period: f64) -> f64 {
        let (u1, u2) = elclib::adjust_periodic(0., period, precision::p_confusion(), x1, x2);
        let mut dx = u2 - u1;
        if dx > period / 2. {
            dx -= period;
        }
        dx
    }

    /// `Getduv(f, uv, dir, factor, duv)`: on a quadric face, the UV direction from `uv`
    /// to the projection of its point moved by `factor * dir` (false when farther than
    /// 100 times the face tolerance), periodic steps taken the short way;
    /// `TopOpeBRepToolTOOL::getduv(&f, uv, dir, 0.1)?`.
    pub fn getduv(
        f: &Face,
        uv: Point2,
        dir: Vector3,
        factor: f64,
    ) -> Result<Option<Direction2>, Failure> {
        if !Self::is_quad_face(f)? {
            return Ok(None);
        }
        let mut bndf = BoundingBox3::default();
        BRepBndLib::add_close(f, &mut bndf)?;
        let mut p = fun_tool_value_on_face(uv, f)?;
        p.translate(dir * (factor));
        let (mut uvtr, mut d) = (Point2::default(), 0.);
        fun_tool_proj_ponf(p, f, &mut uvtr, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)?;
        if d > BRepTool::face_tolerance(f)? * 1.0e2 {
            return Ok(None);
        }
        let mut duv = Vector2::between(uv, uvtr);
        let s = TopOpeBRepToolShapeTool::basis_surface_of_face(f)?;
        let surface = s.get().ok_or_else(|| null_pcurve("Getduv"))?;
        if surface.is_u_periodic() && duv.x().abs() > surface.u_period()? / 2. {
            duv.set_x(Self::periodic_step(uv.x(), uvtr.x(), surface.u_period()?));
        }
        if surface.is_v_periodic() && duv.y().abs() > surface.v_period()? / 2. {
            duv.set_y(Self::periodic_step(uv.y(), uvtr.y(), surface.v_period()?));
        }
        Ok(Some(Direction2::from_xy(duv.xy())?))
    }

    /// `uvApp(f, e, pare, eps, uvapp)`: the UV point of the edge parameter moved by
    /// `eps` times the step to the nearest iso inside the face (FUN_tool_getdxx); the
    /// UV point is written even when the step fails, as natively;
    /// `TopOpeBRepToolTOOL::uv_app(&f, &e, 0.5, 0.1, &mut uv)?`.
    pub fn uv_app(
        f: &Face,
        e: &Edge,
        pare: f64,
        eps: f64,
        uvapp: &mut Point2,
    ) -> Result<bool, Failure> {
        if !fun_tool_paron_ef(e, pare, f, uvapp)? {
            return Ok(false);
        }
        let Some(dxx) = fun_tool_getdxx(f, e, pare)? else {
            return Ok(false);
        };
        uvapp.translate(dxx * eps);
        Ok(true)
    }

    /// The position code of a coordinate on its bounds.
    fn st_on_bounds(x: f64, (xf, xl): (f64, f64), tol: f64) -> i32 {
        let mut on = 0;
        if (xf - x).abs() < tol {
            on = TOOL_ONFIRST;
        }
        if (xl - x).abs() < tol {
            on = TOOL_ONLAST;
        }
        if x < xf - tol {
            on = TOOL_INFFIRST;
        }
        if x > xl + tol {
            on = TOOL_SUPLAST;
        }
        on
    }

    /// `stuvF(uv, f, onU, onV)`: the position of U and V on the bounds of the face: 1 on
    /// the first bound, 2 on the last one, -1 before, -2 after, 0 inside (within the
    /// resolutions of the face tolerance); `let (onu, onv) = TopOpeBRepToolTOOL::stuv_f(uv, &f)?;`.
    pub fn stuv_f(uv: Point2, f: &Face) -> Result<(i32, i32), Failure> {
        let bs = BRepAdaptorSurface::with_face(f, true)?;
        let tolf = bs.tolerance()?;
        let (tolu, tolv) = (bs.u_resolution(tolf)?, bs.v_resolution(tolf)?);
        let ub = (bs.first_u_parameter()?, bs.last_u_parameter()?);
        let vb = (bs.first_v_parameter()?, bs.last_v_parameter()?);
        Ok((
            Self::st_on_bounds(uv.x(), ub, tolu),
            Self::st_on_bounds(uv.y(), vb, tolv),
        ))
    }

    /// `outUVbounds(uv, F)`: whether the UV point is out of the parameter bounds of the
    /// face; `TopOpeBRepToolTOOL::out_uv_bounds(uv, &f)?`.
    pub fn out_uv_bounds(uv: Point2, f: &Face) -> Result<bool, Failure> {
        let bs = BRepAdaptorSurface::with_face(f, true)?;
        let out_u = uv.x() > bs.last_u_parameter()? || uv.x() < bs.first_u_parameter()?;
        let out_v = uv.y() > bs.last_v_parameter()? || uv.y() < bs.first_v_parameter()?;
        Ok(out_u || out_v)
    }

    /// `TolUV(F, tol3d)`: the greater of the U and V resolutions of a 3d tolerance;
    /// `TopOpeBRepToolTOOL::tol_uv(&f, 1e-7)?`.
    pub fn tol_uv(f: &Face, tol3d: f64) -> Result<f64, Failure> {
        let bs = BRepAdaptorSurface::with_face(f, true)?;
        Ok(std_max(bs.u_resolution(tol3d)?, bs.v_resolution(tol3d)?))
    }

    /// `TolP(E, F)`: the resolution of the edge tolerance on its p-curve;
    /// `TopOpeBRepToolTOOL::tol_p(&e, &f)?`.
    pub fn tol_p(e: &Edge, f: &Face) -> Result<f64, Failure> {
        let bc2d = BRepAdaptorCurve2d::with_edge_on_face(e, f)?;
        bc2d.resolution(BRepTool::edge_tolerance(e)?)
    }
}
