//! TopOpeBRepTool_TOOL.cxx: the matter angles between vectors and faces, the inner
//! normals near an edge of a face, the vector inside a face at an edge, the state of
//! a point on a face and whether an edge lies on a face.
use super::TopOpeBRepToolTOOL;
use crate::top_ope_brep_tool::geometry::{fun_quad_ct, fun_tool_nc2d_inside_s, fun_tool_ng_s};
use crate::top_ope_brep_tool::normal::nt;
use crate::top_ope_brep_tool::project::{fun_tool_bounds, fun_tool_proj_ponf};
use crate::top_ope_brep_tool::shape_tool::TopOpeBRepToolShapeTool;
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use crate::top_ope_brep_tool::topology::{
    fun_tool_getgeomxx_ng, fun_tool_nggeom_f, fun_tool_orient_e_in_f_forward, fun_tool_par_f_tol,
    fun_tool_paron_ef, fun_tool_paron_ef_tol, fun_tool_plane_face, fun_tool_quad_face,
    fun_tool_value_on_face,
};
use crate::top_ope_brep_tool::two_d::{
    fc2d_editable_curve_on_surface, fc2d_has_old_curve_on_surface,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3, Vector2, Vector3};
use std::f64::consts::PI;

impl TopOpeBRepToolTOOL {
    /// `Matter(d1, dR2, Ref)`: the angle from `d1` to the reversed `dR2` around `Ref`
    /// in [0, 2 PI) (0 or PI within Precision::Angular());
    /// `TopOpeBRepToolTOOL::matter_vec(d1, d2, r)?`.
    pub fn matter_vec(d1: Vector3, d_r2: Vector3, r: Vector3) -> Result<f64, Failure> {
        let d2 = d_r2.reversed();
        let tola = precision::angular();
        let ang = d1.angle(d2)?;
        if ang < tola {
            return Ok(0.);
        }
        if PI - ang < tola {
            return Ok(PI);
        }
        let ang = d1.angle_with_ref(d2, r)?;
        Ok(if ang < 0. { 2. * PI + ang } else { ang })
    }

    /// `Matter(d1, dR2)`: the matter angle of UV vectors around +Z;
    /// `TopOpeBRepToolTOOL::matter_2d(d1, d2)?`.
    pub fn matter_2d(d1: Vector2, d_r2: Vector2) -> Result<f64, Failure> {
        let v1 = Vector3::new(d1.x(), d1.y(), 0.);
        let v_r2 = Vector3::new(d_r2.x(), d_r2.y(), 0.);
        Self::matter_vec(v1, v_r2, Vector3::new(0., 0., 1.))
    }

    /// `Matter(xx1, nt1, xx2, nt2, tola, ang)`: the matter angle between two faces seen
    /// in the plane normal to their edge, from their inner vectors and normals; `None`
    /// when the edge directions `xxi ^ nti` agree (not opposite);
    /// `TopOpeBRepToolTOOL::matter_dirs(xx1, nt1, xx2, nt2, 1e-9)?`.
    pub fn matter_dirs(
        xx1: Direction3,
        nt1: Direction3,
        xx2: Direction3,
        nt2: Direction3,
        tola: f64,
    ) -> Result<Option<f64>, Failure> {
        let mut z1 = xx1.crossed(nt1)?;
        let z2 = xx2.crossed(nt2)?;
        if z2.dot(z1) >= 0. {
            return Ok(None);
        }
        // -nti points towards the matter of fi, so zi gives the opposite sense.
        z1.reverse();
        let ang = xx1.angle_with_ref(xx2, z1);
        if ang.abs() < tola {
            return Ok(Some(0.));
        }
        Ok(Some(if ang < 0. { 2. * PI + ang } else { ang }))
    }

    /// The normal and inner vector of a face at the edge parameter (the UV point within
    /// `tolf` of the face).
    fn nt_and_xx(
        f: &Face,
        e: &Edge,
        par: f64,
        tolf: f64,
    ) -> Result<Option<(Direction3, Direction3)>, Failure> {
        let mut uv = Point2::default();
        if !fun_tool_paron_ef_tol(e, par, f, &mut uv, tolf)? {
            return Ok(None);
        }
        let mut ntf = Direction3::Z;
        if !nt(uv, f, &mut ntf)? {
            return Ok(None);
        }
        let Some(xx) = Self::xx(uv, f, par, e)? else {
            return Ok(None);
        };
        Ok(Some((ntf, xx)))
    }

    /// `Matter(f1, f2, e, par, tola, ang)`: the matter angle of two faces sharing the
    /// edge at its parameter; `TopOpeBRepToolTOOL::matter_faces(&f1, &f2, &e, 0.5, 1e-9)?`.
    pub fn matter_faces(
        f1: &Face,
        f2: &Face,
        e: &Edge,
        par: f64,
        tola: f64,
    ) -> Result<Option<f64>, Failure> {
        let tolf1 = BRepTool::face_tolerance(f1)? * 1.0e2;
        let Some((nt1, xx1)) = Self::nt_and_xx(f1, e, par, tolf1)? else {
            return Ok(None);
        };
        let tolf2 = BRepTool::face_tolerance(f2)? * 2.0e2;
        let Some((nt2, xx2)) = Self::nt_and_xx(f2, e, par, tolf2)? else {
            return Ok(None);
        };
        Self::matter_dirs(xx1, nt1, xx2, nt2, tola)
    }

    /// `MatterKPtg(f1, f2, e, ang)`: for tangent faces, 0 when the point of `f2` near
    /// the point of `f1` beside the edge is below `f1` (against its normal), 2 PI
    /// otherwise; `TopOpeBRepToolTOOL::matter_k_ptg(&f1, &f2, &e)?`.
    pub fn matter_k_ptg(f1: &Face, f2: &Face, e: &Edge) -> Result<Option<f64>, Failure> {
        let (f, l) = fun_tool_bounds(e)?;
        let x: f64 = 0.45678;
        let pare = (1. - x) * f + x * l;
        let eps: f64 = 0.123;
        let mut uv1 = Point2::default();
        fun_tool_paron_ef(e, pare, f1, &mut uv1)?;
        let mut nt1 = Direction3::Z;
        if !nt(uv1, f1, &mut nt1)? {
            return Ok(None);
        }
        let mut uvapp1 = Point2::default();
        if !Self::uv_app(f1, e, pare, eps, &mut uvapp1)? {
            return Ok(None);
        }
        let pf1 = fun_tool_value_on_face(uvapp1, f1)?;
        let (mut uv2, mut d) = (Point2::default(), 0.);
        let ok2 = fun_tool_proj_ponf(pf1, f2, &mut uv2, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)?;
        let pf2 = fun_tool_value_on_face(uv2, f2)?;
        if !ok2 {
            return Ok(None);
        }
        let v12 = Direction3::from_xyz(*Vector3::between(pf1, pf2).xyz())?;
        Ok(Some(if v12.dot(nt1) < 0. { 0. } else { 2. * PI }))
    }

    /// `XX(uv, f, par, e, XX)`: the unit vector inside the face at the edge parameter
    /// (normal to the edge and to the face normal at `uv`), reversed for an edge
    /// REVERSED in the face; `None` for a null vector or an edge not of the face;
    /// `TopOpeBRepToolTOOL::xx(uv, &f, 0.5, &e)?`.
    pub fn xx(uv: Point2, f: &Face, par: f64, e: &Edge) -> Result<Option<Direction3>, Failure> {
        let ng = fun_tool_nggeom_f(uv, f)?;
        let geomxx = fun_tool_getgeomxx_ng(f, e, par, Direction3::from_xyz(*ng.xyz())?)?;
        let tol = precision::confusion() * 1.0e2;
        if geomxx.magnitude() < tol {
            return Ok(None);
        }
        let (ok, oef) = fun_tool_orient_e_in_f_forward(e, f)?;
        if !ok {
            return Ok(None);
        }
        let mut xx = Direction3::from_xyz(*geomxx.xyz())?;
        if oef == Orientation::Reversed {
            xx.reverse();
        }
        Ok(Some(xx))
    }

    /// `Getstp3dF(p, f, uv, st)`: the UV of the projection of the point on the face and
    /// its state: ON within the face tolerance, else OUT when the point is on the side
    /// of the face normal, IN otherwise; false (state UNKNOWN) when the projection or
    /// the normal fails. `uv` is written by a successful projection, as natively;
    /// `let (ok, st) = TopOpeBRepToolTOOL::getstp3d_f(p, &f, &mut uv)?;`.
    pub fn getstp3d_f(p: Point3, f: &Face, uv: &mut Point2) -> Result<(bool, State), Failure> {
        let tol3d = BRepTool::face_tolerance(f)?;
        let mut d = 0.;
        if !fun_tool_proj_ponf(p, f, uv, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)? {
            return Ok((false, State::Unknown));
        }
        if d < tol3d {
            return Ok((true, State::On));
        }
        let ppr = fun_tool_value_on_face(*uv, f)?;
        let mut ntf = Direction3::Z;
        if !nt(*uv, f, &mut ntf)? {
            return Ok((false, State::Unknown));
        }
        let dppr = Direction3::from_xyz(*Vector3::between(p, ppr).xyz())?;
        let st = if dppr.dot(ntf) < 0. {
            State::Out
        } else {
            State::In
        };
        Ok((true, st))
    }

    /// The inner UV normal of a stored p-curve at the parameter and the face normal at
    /// its point; `None` when the edge has no stored p-curve or no orientation in the
    /// face.
    fn ng_app_start(
        par: f64,
        e: &Edge,
        f: &Face,
    ) -> Result<Option<(Point2, Direction2, Direction3)>, Failure> {
        let s = TopOpeBRepToolShapeTool::basis_surface_of_face(f)?;
        if fc2d_has_old_curve_on_surface(e, f)?.is_none() {
            return Ok(None);
        }
        let (pc, _, _, _) = fc2d_editable_curve_on_surface(e, f, false)?;
        let ffor = topods::face(&f.oriented(Orientation::Forward))?;
        let orieinf = Self::try_ori_e_in_f(par, e, &ffor)?;
        if orieinf == 0 {
            return Ok(None);
        }
        let mut uv = Point2::default();
        if !fun_tool_paron_ef(e, par, f, &mut uv)? {
            return Ok(None);
        }
        let ng = fun_tool_ng_s(uv, &s)?;
        let (uv, tg2d) = pc.get().ok_or_else(ng_app_null)?.d1(par)?;
        let mut n2d_inside_s = fun_tool_nc2d_inside_s(Direction2::from_xy(tg2d.xy())?)?;
        if orieinf == 2 {
            n2d_inside_s.reverse();
        }
        Ok(Some((uv, n2d_inside_s, ng)))
    }

    /// `NgApp(par, e, f, tola, ngApp)`: on a non-planar quadric face, the normal at a UV
    /// point moved inside the face from the edge point (steps of 0.45678, growing by
    /// 1.25, at most 5) as soon as it differs from the normal at the edge by more than
    /// `tola`; `None` when the normals stay equal or the edge has no stored p-curve;
    /// `TopOpeBRepToolTOOL::ng_app(0.5, &e, &f, 1e-9)?`.
    pub fn ng_app(par: f64, e: &Edge, f: &Face, tola: f64) -> Result<Option<Direction3>, Failure> {
        let s = TopOpeBRepToolShapeTool::basis_surface_of_face(f)?;
        if s.is_null() || fun_tool_plane_face(f)? || !fun_tool_quad_face(f)? {
            return Ok(None);
        }
        let Some((uv, n2d_inside_s, ng)) = Self::ng_app_start(par, e, f)? else {
            return Ok(None);
        };
        let mut duv = Vector2::from(n2d_inside_s) * (0.45678);
        let (nmax, mut delta) = (5, 0.45678_f64);
        for _ in 1..=nmax {
            // FUN_tool_ngS returns a unit direction: the native magnitude test passes.
            let newng = fun_tool_ng_s(uv.translated(duv), &s)?;
            if !ng.is_equal(&newng, tola) {
                // `gp_Dir(newng)` of the vector normalizes the unit direction again.
                return Ok(Some(Direction3::from_xyz(*Vector3::from(newng).xyz())?));
            }
            delta *= 1.25;
            duv = Vector2::from(n2d_inside_s) * (delta);
        }
        Ok(None)
    }

    /// `tryNgApp(par, e, f, tola, Ng)`: `NgApp`, else the geometric normal at the edge
    /// point; `None` when the edge point is not on the face;
    /// `TopOpeBRepToolTOOL::try_ng_app(0.5, &e, &f, 1e-9)?`.
    pub fn try_ng_app(
        par: f64,
        e: &Edge,
        f: &Face,
        tola: f64,
    ) -> Result<Option<Direction3>, Failure> {
        let mut uv = Point2::default();
        if !fun_tool_paron_ef(e, par, f, &mut uv)? {
            return Ok(None);
        }
        let ng = Direction3::from_xyz(*fun_tool_nggeom_f(uv, f)?.xyz())?;
        Ok(Some(Self::ng_app(par, e, f, tola)?.unwrap_or(ng)))
    }

    /// `IsQuad(E)`: whether the edge curve is a line or a conic;
    /// `TopOpeBRepToolTOOL::is_quad_edge(&e)?`.
    pub fn is_quad_edge(e: &Edge) -> Result<bool, Failure> {
        Ok(fun_quad_ct(BRepAdaptorCurve::with_edge(e)?.get_type()?))
    }

    /// `IsQuad(F)`: whether the basis surface of the face is a quadric;
    /// `TopOpeBRepToolTOOL::is_quad_face(&f)?`.
    pub fn is_quad_face(f: &Face) -> Result<bool, Failure> {
        fun_tool_quad_face(f)
    }

    /// The axis of the conic of an edge on a plane, `None` for other curves.
    fn conic_axis(bc: &BRepAdaptorCurve) -> Result<Option<Direction3>, Failure> {
        Ok(match bc.get_type()? {
            CurveType::Circle => Some(bc.circle()?.axis().direction()),
            CurveType::Ellipse => Some(bc.ellipse()?.axis().direction()),
            CurveType::Hyperbola => Some(bc.hyperbola()?.axis().direction()),
            CurveType::Parabola => Some(bc.parabola()?.axis().direction()),
            _ => None,
        })
    }

    /// The analytic answer of `EdgeONFace` on a plane or a cylinder; `None` for the
    /// general case.
    fn edge_on_face_analytic(
        bs: &BRepAdaptorSurface,
        bc: &BRepAdaptorCurve,
        tge: Vector3,
        ngf: Vector3,
        (tola, tol3d): (f64, f64),
    ) -> Result<Option<bool>, Failure> {
        let st = bs.get_type()?;
        let ct = bc.get_type()?;
        if st == SurfaceType::Plane {
            if ct == CurveType::Line {
                return Ok(Some(true));
            }
            let Some(ne) = Self::conic_axis(bc)? else {
                return Ok(None);
            };
            let prod = Vector3::from(ne).dot(ngf);
            return Ok(Some((1. - prod.abs()).abs() < tola));
        }
        if st != SurfaceType::Cylinder {
            return Ok(None);
        }
        let ne = match ct {
            CurveType::Line => Direction3::from_xyz(*tge.xyz())?,
            CurveType::Circle => bc.circle()?.axis().direction(),
            _ => return Ok(None),
        };
        let axicy = bs.cylinder()?.axis().direction();
        let mut isonfa = (1. - ne.dot(axicy).abs()).abs() < tola;
        if isonfa && ct == CurveType::Circle {
            isonfa = (bc.circle()?.radius() - bs.cylinder()?.radius()).abs() < tol3d;
        }
        Ok(Some(isonfa))
    }

    /// `EdgeONFace(par, ed, uv, fa, isonfa)`: whether the edge, tangent to the face at
    /// the edge parameter (its point at `uv`), lies on the face: analytically on planes
    /// and cylinders, else by the distance of an edge point near the parameter to its
    /// projection on the face; `None` when it cannot be computed;
    /// `TopOpeBRepToolTOOL::edge_on_face(0.5, &e, uv, &f)?`.
    pub fn edge_on_face(
        par: f64,
        ed: &Edge,
        uv: Point2,
        fa: &Face,
    ) -> Result<Option<bool>, Failure> {
        if BRepTool::degenerated(ed)? {
            return Ok(Some(true));
        }
        let tola = precision::angular() * 1.0e2;
        let Some(tge) = tggeom_e(par, ed)? else {
            return Ok(None);
        };
        let ngf = fun_tool_nggeom_f(uv, fa)?;
        if tge.dot(ngf).abs() >= tola {
            return Ok(Some(false));
        }
        let bs = BRepAdaptorSurface::with_face(fa, true)?;
        let bc = BRepAdaptorCurve::with_edge(ed)?;
        let tole = bc.tolerance()?;
        let tol1de = bc.resolution(tole)?;
        let tolf = bs.tolerance()?;
        let tol3d = std_max(tole, tolf) * 1.0e2;
        if let Some(found) = Self::edge_on_face_analytic(&bs, &bc, tge, ngf, (tola, tol3d))? {
            return Ok(Some(found));
        }
        let x: f64 = 0.12345;
        let (f, l) = fun_tool_bounds(ed)?;
        let onf = (par - f).abs() < tol1de;
        let opar = if onf {
            (1. - x) * f + x * l
        } else {
            (1. - x) * f + x * par
        };
        let opc = bc.value(opar)?;
        let mut ouv = Point2::default();
        if !fun_tool_par_f_tol(ed, opar, fa, &mut ouv, tolf)? {
            return Ok(None);
        }
        let ops = bs.value(ouv.x(), ouv.y())?;
        Ok(Some(opc.distance(&ops) < tol3d))
    }
}

/// Standard_NullObject of the null p-curve of `NgApp`.
fn ng_app_null() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepTool_TOOL::NgApp: null p-curve; expected a stored 2d curve",
        "TopOpeBRepTool_TOOL",
    )
}
