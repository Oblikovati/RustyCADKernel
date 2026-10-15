//! TopOpeBRepTool_TOOL.cxx: the curvature of an edge in a plane and of the apparent
//! contour of a face in the plane of a tangent.
use super::TopOpeBRepToolTOOL;
use crate::top_ope_brep_tool::shape_tool::TopOpeBRepToolShapeTool;
use crate::top_ope_brep_tool::topology::fun_tool_nggeom_f;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_lprop::BRepLPropCLProps;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

/// `FUN_nullprodv(prodv)` (static): `|prodv| < 1e-6`.
fn fun_nullprodv(prodv: f64) -> bool {
    let tola: f64 = 1.0e-6;
    prodv.abs() < tola
}

/// The axis and directness of a cylinder, cone or torus.
fn revolution_axis(
    gs: &GeomAdaptorSurface,
    st: SurfaceType,
) -> Result<(Direction3, bool), Failure> {
    Ok(match st {
        SurfaceType::Cylinder => {
            let c = gs.cylinder()?;
            (c.axis().direction(), c.direct()?)
        }
        SurfaceType::Cone => {
            let c = gs.cone()?;
            (c.axis().direction(), c.direct()?)
        }
        _ => {
            let t = gs.torus()?;
            (t.axis().direction(), t.direct()?)
        }
    })
}

/// The greater absolute principal curvature of the surface at UV (0 when undefined).
fn max_abs_curvature(
    s: &Handle<dyn GeomSurface>,
    uv0: Point2,
    curv: &mut f64,
) -> Result<(), Failure> {
    let mut slprops =
        SLProps::with_parameters(s.clone(), uv0.x(), uv0.y(), 2, precision::confusion())?;
    if slprops.is_curvature_defined()? {
        let min_a = slprops.min_curvature()?.abs();
        let max_a = slprops.max_curvature()?.abs();
        *curv = if max_a > min_a { max_a } else { min_a };
    }
    Ok(())
}

/// `FUN_analyticcS(uv0, S, ngS, tg0, curv, direct)` (static): whether the apparent
/// contour of the surface in the plane (ngS, tg0) is analytic, its curvature in `curv`
/// and the directness of the surface in `direct`. Natively the null test is made on
/// `S` after taking its basis (kept: a null basis of a non-null surface fails in the
/// adaptor).
fn fun_analyticc_s(
    uv0: Point2,
    s: &Handle<dyn GeomSurface>,
    ng_s: Direction3,
    tg0: Direction3,
    curv: &mut f64,
    direct: &mut bool,
) -> Result<bool, Failure> {
    (*curv, *direct) = (0., true);
    let su = TopOpeBRepToolShapeTool::basis_surface(s)?;
    if s.is_null() {
        return Ok(true);
    }
    let gs = GeomAdaptorSurface::with_surface(&su)?;
    let st = gs.get_type()?;
    let mut curvdone = st == SurfaceType::Plane;
    if matches!(
        st,
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Torus
    ) {
        let (axis, d) = revolution_axis(&gs, st)?;
        *direct = d;
        let prod = axis.dot(tg0);
        let is_max_acurv = fun_nullprodv(1. - prod.abs());
        let mut nullcurv = fun_nullprodv(prod);
        if st != SurfaceType::Torus {
            nullcurv = nullcurv || fun_nullprodv(1. - ng_s.dot(tg0).abs());
        }
        if nullcurv {
            (*curv, curvdone) = (0., true);
        }
        if is_max_acurv {
            max_abs_curvature(s, uv0, curv)?;
            curvdone = true;
        }
    }
    if st == SurfaceType::Sphere {
        let sp = gs.sphere()?;
        (*curv, *direct) = (1. / sp.radius(), sp.direct()?);
        curvdone = true;
    }
    Ok(curvdone)
}

impl TopOpeBRepToolTOOL {
    /// The curvature of a non-line edge in the plane normal to `tg0`; `None` without
    /// tangent or in the general case.
    fn curv_e_general(
        bac: &BRepAdaptorCurve,
        par: f64,
        tg0: Direction3,
        tola: f64,
    ) -> Result<(bool, f64), Failure> {
        let mut clprops = BRepLPropCLProps::with_parameter(bac, par, 2, precision::confusion())?;
        if !clprops.is_tangent_defined()? {
            return Ok((false, 0.));
        }
        let curv = clprops.curvature()?.abs();
        let tol = precision::confusion() * 1.0e2;
        if curv < tol {
            return Ok((true, 0.));
        }
        let n = clprops.normal()?;
        let t = clprops.tangent()?;
        let dot = n.crossed(t)?.dot(tg0).abs();
        if dot < tola {
            return Ok((true, 0.));
        }
        // The general case fails with the curvature written, as natively.
        Ok(((1. - dot).abs() < tola, curv))
    }

    /// `CurvE(E, par, tg0, curv)`: the curvature of the edge seen in the plane normal to
    /// `tg0`: for a line, whether it is not along `tg0` (curvature 0); else the curve
    /// curvature when its binormal is along `tg0`, 0 when the curvature or the binormal
    /// component is null; false in the general case (the curvature written natively);
    /// `let (ok, curv) = TopOpeBRepToolTOOL::curv_e(&e, 0.5, tg0)?;`.
    pub fn curv_e(e: &Edge, par: f64, tg0: Direction3) -> Result<(bool, f64), Failure> {
        let bac = BRepAdaptorCurve::with_edge(e)?;
        let tola = rustycad_tkernel::precision::angular() * 1.0e3;
        if bac.get_type()? == CurveType::Line {
            let dot = bac.line()?.direction().dot(tg0);
            return Ok(((1. - dot).abs() >= tola, 0.));
        }
        Self::curv_e_general(&bac, par, tg0, tola)
    }

    /// `CurvF(F, uv, tg0, curv, direct)`: the curvature of the apparent contour of the
    /// face in the plane of its normal and `tg0`, analytic on elementary surfaces, else
    /// a principal curvature along `tg0` (with the directness from its sign); the
    /// curvature and directness as written natively; `false` when undefined;
    /// `let (ok, curv, direct) = TopOpeBRepToolTOOL::curv_f(&f, uv, tg0, true)?;`.
    pub fn curv_f(
        f: &Face,
        uv: Point2,
        tg0: Direction3,
        direct: bool,
    ) -> Result<(bool, f64, bool), Failure> {
        let mut curv: f64 = 0.;
        let mut direct = direct;
        let ng_s = Direction3::from_xyz(*fun_tool_nggeom_f(uv, f)?.xyz())?;
        let s = TopOpeBRepToolShapeTool::basis_surface_of_face(f)?;
        if s.is_null() {
            return Ok((false, curv, direct));
        }
        if fun_analyticc_s(uv, &s, ng_s, tg0, &mut curv, &mut direct)? {
            return Ok((true, curv, direct));
        }
        let mut slprops =
            SLProps::with_parameters(s.clone(), uv.x(), uv.y(), 2, precision::confusion())?;
        if !slprops.is_curvature_defined()? {
            return Ok((false, curv, direct));
        }
        let tola: f64 = 1.0e-6;
        let (max_d, min_d) = slprops.curvature_directions()?;
        let (mincurv, maxcurv) = (slprops.min_curvature()?, slprops.max_curvature()?);
        let dmax = Vector3::from(ng_s.crossed(max_d)?);
        let dmin = Vector3::from(ng_s.crossed(min_d)?);
        let npl = Vector3::from(tg0);
        let iscurmax = (1. - dmax.dot(npl)).abs() < tola;
        if iscurmax {
            (direct, curv) = (maxcurv < 0., maxcurv.abs());
        }
        let iscurmin = (1. - dmin.dot(npl)).abs() < tola;
        if iscurmin {
            (direct, curv) = (mincurv < 0., mincurv.abs());
        }
        Ok((iscurmax || iscurmin, curv, direct))
    }
}
