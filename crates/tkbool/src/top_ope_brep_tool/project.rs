// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_PROJECT.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `FUN_tool_*` projections of points on curves, edges, surfaces and faces. The
//! results are the native output arguments, written only where natively written (the
//! edge and face projections first set the distance to 1).
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPC, ExtPC2d};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtPF;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The native default tolerance of `Extrema_ExtPC(P, C, Uinf, Usup)`.
const EXT_PC_TOLF: f64 = 1.0e-10;

/// The parameter range of a projection on a curve: `(pmin, pmax)`.
pub type ParameterRange = (f64, f64);

/// `FUN_tool_bounds(E, f, l)`: the range of the adaptor of the edge;
/// `let (f, l) = fun_tool_bounds(&edge)?;`.
pub fn fun_tool_bounds(e: &Edge) -> Result<(f64, f64), Failure> {
    let bac = BRepAdaptorCurve::with_edge(e)?;
    Ok((bac.first_parameter()?, bac.last_parameter()?))
}

/// The index of the smallest square distance (the first of equal ones), from 1.
fn nearest_index(
    n: i32,
    square_distance: impl Fn(i32) -> Result<f64, Failure>,
) -> Result<i32, Failure> {
    let mut dist2_min: f64 = square_distance(1)?;
    let mut index: i32 = 1;
    for i in 2..=n {
        let dist2 = square_distance(i)?;
        if dist2 < dist2_min {
            (dist2_min, index) = (dist2, i);
        }
    }
    Ok(index)
}

/// `FUN_tool_getindex(ponc)`: the index of the nearest extremum of a done
/// Extrema_ExtPC; `fun_tool_getindex(&ponc)?`.
pub fn fun_tool_getindex(ponc: &ExtPC<'_>) -> Result<i32, Failure> {
    nearest_index(ponc.nb_ext()?, |i| ponc.square_distance(i))
}

/// `FUN_tool_getindex(ponc)` of an Extrema_ExtPC2d; `fun_tool_getindex2d(&ponc)?`.
pub fn fun_tool_getindex2d(ponc: &ExtPC2d<'_>) -> Result<i32, Failure> {
    nearest_index(ponc.nb_ext()?, |i| ponc.square_distance(i))
}

/// The bound of `pmin`, `pmax` (in that order) within `tole` of the point, written
/// with its distance.
fn on_bound(
    (pmin, pmax): ParameterRange,
    tole: f64,
    square_distance: impl Fn(f64) -> Result<f64, Failure>,
    (param, dist): (&mut f64, &mut f64),
) -> Result<bool, Failure> {
    for par in [pmin, pmax] {
        let d2 = square_distance(par)?;
        if d2 < tole * tole {
            (*param, *dist) = (par, d2.sqrt());
            return Ok(true);
        }
    }
    Ok(false)
}

/// `FUN_tool_projPonC(P, tole, BAC, pmin, pmax, param, dist)`: the nearest extremum on
/// the range, else a bound within `tole`; false otherwise (outputs untouched);
/// `fun_tool_proj_ponc_tol(p, 1e-7, &bac, (0., 1.), &mut par, &mut d)?`.
pub fn fun_tool_proj_ponc_tol(
    p: Point3,
    tole: f64,
    bac: &BRepAdaptorCurve,
    range: ParameterRange,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    let ponc = ExtPC::with_range(p, bac, range.0, range.1, EXT_PC_TOLF)?;
    if !(ponc.is_done() && ponc.nb_ext()? > 0) {
        return on_bound(
            range,
            tole,
            |par| Ok(bac.value(par)?.square_distance(&p)),
            (param, dist),
        );
    }
    let i = fun_tool_getindex(&ponc)?;
    (*param, *dist) = (ponc.point(i)?.parameter(), ponc.square_distance(i)?.sqrt());
    Ok(true)
}

/// `FUN_tool_projPonC(P, BAC, pmin, pmax, param, dist)`: with the tolerance of the
/// adaptor; `fun_tool_proj_ponc_range(p, &bac, (0., 1.), &mut par, &mut d)?`.
pub fn fun_tool_proj_ponc_range(
    p: Point3,
    bac: &BRepAdaptorCurve,
    range: ParameterRange,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    fun_tool_proj_ponc_tol(p, bac.tolerance()?, bac, range, param, dist)
}

/// `FUN_tool_projPonC(P, BAC, param, dist)`: on the whole adaptor;
/// `fun_tool_proj_ponc(p, &bac, &mut par, &mut d)?`.
pub fn fun_tool_proj_ponc(
    p: Point3,
    bac: &BRepAdaptorCurve,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    let range = (bac.first_parameter()?, bac.last_parameter()?);
    fun_tool_proj_ponc_tol(p, bac.tolerance()?, bac, range, param, dist)
}

/// `FUN_tool_projPonC2D(P, tole, BAC2D, pmin, pmax, param, dist)`: the point projected
/// on the face of the adaptor (writing `dist`), then on the p-curve; natively NbExt is
/// read before the done test, so a failed extrema raises StdFail_NotDone;
/// `fun_tool_proj_ponc2d_tol(p, 1e-7, &bac2d, (0., 1.), &mut par, &mut d)?`.
pub fn fun_tool_proj_ponc2d_tol(
    p: Point3,
    tole: f64,
    bac2d: &BRepAdaptorCurve2d,
    range: ParameterRange,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    let mut p2d = Point2::default();
    if !fun_tool_proj_ponf(
        p,
        &bac2d.face(),
        &mut p2d,
        dist,
        ExtFlag::MinMax,
        ExtAlgo::Grad,
    )? {
        return Ok(false);
    }
    let ponc2d = ExtPC2d::with_range(p2d, bac2d, range.0, range.1, EXT_PC_TOLF)?;
    let nbext = ponc2d.nb_ext()?;
    if !(ponc2d.is_done() && nbext > 0) {
        return on_bound(
            range,
            tole,
            |par| Ok(bac2d.value(par)?.square_distance(p2d)),
            (param, dist),
        );
    }
    let i = fun_tool_getindex2d(&ponc2d)?;
    (*param, *dist) = (
        ponc2d.point(i)?.parameter(),
        ponc2d.square_distance(i)?.sqrt(),
    );
    Ok(true)
}

/// `FUN_tool_projPonC2D(P, BAC2D, pmin, pmax, param, dist)`: with the edge tolerance;
/// `fun_tool_proj_ponc2d_range(p, &bac2d, (0., 1.), &mut par, &mut d)?`.
pub fn fun_tool_proj_ponc2d_range(
    p: Point3,
    bac2d: &BRepAdaptorCurve2d,
    range: ParameterRange,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    let tole = BRepTool::edge_tolerance(&bac2d.edge())?;
    fun_tool_proj_ponc2d_tol(p, tole, bac2d, range, param, dist)
}

/// `FUN_tool_projPonC2D(P, BAC2D, param, dist)`: on the whole adaptor;
/// `fun_tool_proj_ponc2d(p, &bac2d, &mut par, &mut d)?`.
pub fn fun_tool_proj_ponc2d(
    p: Point3,
    bac2d: &BRepAdaptorCurve2d,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    let tole = BRepTool::edge_tolerance(&bac2d.edge())?;
    let range = (bac2d.first_parameter()?, bac2d.last_parameter()?);
    fun_tool_proj_ponc2d_tol(p, tole, bac2d, range, param, dist)
}

/// `FUN_tool_projPonS(P, S, UV, dist, anExtFlag, anExtAlgo)` (natively defaulted to
/// MINMAX and Grad): the nearest projection on the natural bounds of the surface;
/// false (outputs untouched) when it fails;
/// `fun_tool_proj_pons(p, &s, &mut uv, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)?`.
pub fn fun_tool_proj_pons(
    p: Point3,
    s: &Handle<dyn GeomSurface>,
    uv: &mut Point2,
    dist: &mut f64,
    flag: ExtFlag,
    algo: ExtAlgo,
) -> Result<bool, Failure> {
    let surface = s.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "FUN_tool_projPonS: null surface; expected a surface",
            "TopOpeBRepTool",
        )
    })?;
    let mut pons = ProjectPointOnSurf::new();
    pons.init_surface(s, surface.bounds(), algo)?;
    pons.set_extrema_flag(flag);
    pons.perform(p)?;
    if !pons.extrema().is_done() || pons.nb_points()? == 0 {
        return Ok(false);
    }
    *dist = pons.lower_distance()?;
    let (u, v) = pons.lower_distance_parameters()?;
    *uv = Point2::new(u, v);
    Ok(true)
}

/// `FUN_tool_projPonE(P, tole, E, param, dist)`: `dist` set to 1, then the projection
/// on the edge range, the parameter snapped to a bound within the parametric
/// confusion; `fun_tool_proj_pone_tol(p, 1e-7, &edge, &mut par, &mut d)?`.
pub fn fun_tool_proj_pone_tol(
    p: Point3,
    tole: f64,
    e: &Edge,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    *dist = 1.;
    let bac = BRepAdaptorCurve::with_edge(e)?;
    let range = (bac.first_parameter()?, bac.last_parameter()?);
    if !fun_tool_proj_ponc_tol(p, tole, &bac, range, param, dist)? {
        return Ok(false);
    }
    let (f, l) = fun_tool_bounds(e)?;
    let tolp: f64 = precision::parametric(precision::confusion());
    if (f - *param).abs() < tolp {
        *param = f;
    }
    if (l - *param).abs() < tolp {
        *param = l;
    }
    Ok(true)
}

/// `FUN_tool_projPonE(P, E, param, dist)`: with the edge tolerance;
/// `fun_tool_proj_pone(p, &edge, &mut par, &mut d)?`.
pub fn fun_tool_proj_pone(
    p: Point3,
    e: &Edge,
    param: &mut f64,
    dist: &mut f64,
) -> Result<bool, Failure> {
    fun_tool_proj_pone_tol(p, BRepTool::edge_tolerance(e)?, e, param, dist)
}

/// `FUN_tool_projPonboundedF(P, F, UV, dist)`: `dist` set to 1, then the nearest of the
/// BRepExtrema_ExtPF extrema below 1e7; `fun_tool_proj_ponbounded_f(p, &face, &mut uv, &mut d)?`.
pub fn fun_tool_proj_ponbounded_f(
    p: Point3,
    f: &Face,
    uv: &mut Point2,
    dist: &mut f64,
) -> Result<bool, Failure> {
    *dist = 1.;
    let v = BRepLibMakeVertex::new(p)?.vertex()?;
    let ponf = BRepExtremaExtPF::with_vertex_face(&v, f, ExtFlag::MinMax, ExtAlgo::Grad)?;
    if !ponf.is_done() || ponf.nb_ext() == 0 {
        return Ok(false);
    }
    let (mut dmin2, mut imin) = (1.0e14_f64, 0);
    for i in 1..=ponf.nb_ext() {
        let d2 = ponf.square_distance(i)?;
        if d2 < dmin2 {
            (imin, dmin2) = (i, d2);
        }
    }
    if imin == 0 {
        return Ok(false);
    }
    let (u, v) = ponf.parameter(imin)?;
    (*dist, *uv) = (dmin2.sqrt(), Point2::new(u, v));
    Ok(true)
}

/// `FUN_tool_projPonF(P, F, UV, dist, anExtFlag, anExtAlgo)`: `dist` set to 1, then the
/// projection on the surface of the face (with its location);
/// `fun_tool_proj_ponf(p, &face, &mut uv, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)?`.
pub fn fun_tool_proj_ponf(
    p: Point3,
    f: &Face,
    uv: &mut Point2,
    dist: &mut f64,
    flag: ExtFlag,
    algo: ExtAlgo,
) -> Result<bool, Failure> {
    *dist = 1.;
    let s = BRepTool::surface(f)?;
    fun_tool_proj_pons(p, &s, uv, dist, flag, algo)
}
