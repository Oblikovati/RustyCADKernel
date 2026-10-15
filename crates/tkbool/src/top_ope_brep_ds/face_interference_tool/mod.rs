// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_FaceInterferenceTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_FaceInterferenceTool`: the transition of a face along an edge where
//! several faces meet, accumulated with a surface transition (with the free functions
//! FUN_Parameters and FUN_ComputeGeomData).
use super::export::config::dir;
use super::export::{fds_config3d, fds_data, fds_has_same_domain3d, iref};
use super::{
    TopOpeBRepDSConfig, TopOpeBRepDSCurve, TopOpeBRepDSDataStructure, TopOpeBRepDSInterference,
    TopOpeBRepDSShapeShapeInterference,
};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolShapeTool, fun_tool_find_pin_e, fun_tool_nggeom_f,
    fun_tool_orient_e_in_f_forward, fun_tool_paron_ef, fun_tool_plane_face, fun_tool_proj_ponf,
    tggeom_e,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_lprop::BRepLPropSLProps;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::toptrans::SurfaceTransition;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};
use std::cell::Cell;

thread_local! {
    /// `STATIC_TOREVERSE` (xpu150498): the edge of `Init` is opposite to its geometry.
    static STATIC_TOREVERSE: Cell<bool> = const { Cell::new(false) };
}

/// `FUN_RaiseError()` (static): Standard_ProgramError.
fn raise() -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        "TopOpeBRepDS_FaceInterferenceTool",
        "TopOpeBRepDS_FaceInterferenceTool",
    )
}

/// `FUN_Parameters(Pnt, F, u, v)`: the UV of the nearest point of the face surface
/// within 1000 times its tolerance; `fun_parameters(p, &face)?`.
pub fn fun_parameters(pnt: Point3, f: &Shape) -> Result<Option<(f64, f64)>, Failure> {
    let face = topods::face(f)?;
    let surf = BRepAdaptorSurface::with_face(&face, true)?;
    let uvtol = surf.tolerance()?;
    let extps = ExtPS::with_surface(pnt, &surf, uvtol, uvtol, ExtFlag::Min, ExtAlgo::Grad)?;
    if !extps.is_done() || extps.nb_ext()? == 0 {
        return Ok(None);
    }
    let (u, v) = extps.point(1)?.parameter();
    let d2 = extps.square_distance(1)?;
    let tol_f = BRepTool::face_tolerance(&face)?;
    Ok((d2 < tol_f * tol_f * 1.0e6).then_some((u, v)))
}

/// `FUN_ComputeGeomData(F, uv, Norm)`: the normal of the face at `uv`;
/// `fun_compute_geom_data_norm(&face, uv)?`.
pub fn fun_compute_geom_data_norm(f: &Shape, uv: Point2) -> Result<Direction3, Failure> {
    dir(fun_tool_nggeom_f(uv, &topods::face(f)?)?)
}

/// `FUN_sphere(F)` (static): the basis surface of the face is a sphere.
fn is_sphere(f: &Shape) -> Result<bool, Failure> {
    let su = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f)?)?;
    Ok(GeomAdaptorSurface::with_surface(&su)?.get_type()? == SurfaceType::Sphere)
}

/// The geometry of a face at a point: normal, principal directions and curvatures.
pub type FaceGeomData = (Direction3, Direction3, Direction3, f64, f64);

/// A direction normal to `d1` for an umbilic point (`FUN_ComputeGeomData`).
fn umbilic_d2(d1: Direction3) -> Result<Direction3, Failure> {
    let (x, y, z, tol) = (d1.x(), d1.y(), d1.z(), precision::confusion());
    let (nx, ny, nz) = (x.abs() < tol, y.abs() < tol, z.abs() < tol);
    if (nx && ny) || (nx && nz) {
        return Ok(Direction3::X);
    }
    if ny && nz {
        return Ok(Direction3::Y);
    }
    Direction3::new(y * z, x * z, -2. * x * y)
}

/// `FUN_ComputeGeomData(F, uv, Norm, D1, D2, Cur1, Cur2)`: the normal, principal
/// directions and curvatures of the face at `uv` (for an umbilic point of a plane or
/// sphere, the normal and a normal direction); Standard_ProgramError where the
/// curvature is undefined; `fun_compute_geom_data(&face, uv)?`.
pub fn fun_compute_geom_data(f: &Shape, uv: Point2) -> Result<FaceGeomData, Failure> {
    let face = topods::face(f)?;
    let surf = BRepAdaptorSurface::with_face(&face, true)?;
    let (uu, vv) = (uv.x(), uv.y());
    let sphere = is_sphere(f)?;
    let mut plane = fun_tool_plane_face(f)?;
    let mut props = BRepLPropSLProps::with_parameters(&surf, uu, vv, 2, precision::confusion())?;
    if !props.is_curvature_defined()? {
        return Err(Failure::with_type(
            "Standard_ProgramError",
            "TopOpeBRepDS_FaceInterferenceTool::Init",
            "TopOpeBRepDS_FaceInterferenceTool",
        ));
    }
    if !props.is_umbilic()? {
        let (cur1, cur2) = (props.max_curvature()?, props.min_curvature()?);
        let (d1, d2) = props.curvature_directions()?;
        let norm = dir(fun_tool_nggeom_f(uv, &face)?)?;
        return Ok((norm, d1, d2, cur1, cur2));
    }
    let cur = props.mean_curvature()?;
    plane = plane || cur.abs() < 1.0e-8;
    let norm = if plane {
        dir(fun_tool_nggeom_f(uv, &face)?)?
    } else if sphere {
        let center = surf.sphere()?.location();
        let value = surf.value(uu, vv)?;
        dir(Vector3::between(center, value))?
    } else {
        return Err(Failure::new("FUN_ComputeGeomData"));
    };
    Ok((norm, norm, umbilic_d2(norm)?, cur, cur))
}

/// The face interference tool on a data structure (`TopOpeBRepDS_FaceInterferenceTool`
/// holds a pointer to it); natively the face orientation, index, point, parameter and
/// tolerance start uninitialized (FORWARD, 0, origin, 0, 0 here); the native `isLine`
/// is always false (set by `Init`), so only the curvature frames are used;
/// `let mut t = TopOpeBRepDSFaceInterferenceTool::new(&ds);`.
pub struct TopOpeBRepDSFaceInterferenceTool<'a> {
    my_pbds: &'a TopOpeBRepDSDataStructure,
    myrefdef: bool,
    my_face_orientation: Orientation,
    my_face_oriented: i32,
    my_tool: SurfaceTransition,
    my_edge: Shape,
    my_pnt_on_ed: Point3,
    my_par_on_ed: f64,
    my_on_ed_def: bool,
    my_tole: f64,
}

/// The shape/shape interference of a handle (none for another type).
fn ssi(i: &Handle<TopOpeBRepDSInterference>) -> Handle<TopOpeBRepDSShapeShapeInterference> {
    i.cast()
}

mod members;
