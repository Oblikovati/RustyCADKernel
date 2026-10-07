// Rust translation of ChFi3d_conexfaces, ChFi3d_EdgeState, ChFi3d_evalconti and
// ChFi3d_KParticular (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces around an edge and the classifications the builders read from them.
use crate::chfi_ds::{Map, SpineHandle, State, lock};
use crate::chfi3d::concave_side;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity, SurfaceType};

/// `ChFi3d_conexfaces(E, F1, F2, EFMap)`: the first face of the edge and the next face
/// that is another face or the same face along a seam; null faces when missing
/// (Standard_NoSuchObject when the edge is not in the map);
/// `let (f1, f2) = conexfaces(&e, &map)?;`.
pub fn conexfaces(e: &Edge, ef_map: &Map) -> Result<(Face, Face), Failure> {
    let (mut f1, mut f2) = (Face::default(), Face::default());
    for shape in ef_map.find_from_key(e)?.iter() {
        if f1.is_null() {
            f1 = topods::face(shape)?;
            continue;
        }
        f2 = topods::face(shape)?;
        if !f2.is_same(&f1) || BRepTool::is_closed_on_face(e, &f1)? {
            break;
        }
        f2 = Face::default();
    }
    Ok((f1, f2))
}

/// `ChFi3d::ConcaveSide` of two faces (adaptors restricted to the faces).
fn sides(f1: &Face, f2: &Face, e: &Edge) -> Result<(i32, Orientation, Orientation), Failure> {
    let (s1, s2) = (
        BRepAdaptorSurface::with_face(f1, true)?,
        BRepAdaptorSurface::with_face(f2, true)?,
    );
    concave_side(&s1, &s2, e)
}

/// `ChFi3d_EdgeState(E, EFMap)`: the concavity state of a vertex of three edges:
/// FreeBoundary when an edge has one face, AllSame when the three edges have the same
/// concave sides, OnDiff or OnSame otherwise; `edge_state(&[e1, e2, e3], &map)?`.
pub fn edge_state(e: &[Edge; 3], ef_map: &Map) -> Result<State, Failure> {
    let (f1, f2) = conexfaces(&e[0], ef_map)?;
    let (f3, f4) = conexfaces(&e[1], ef_map)?;
    let (f5, f6) = conexfaces(&e[2], ef_map)?;
    if f2.is_null() || f4.is_null() || f6.is_null() {
        return Ok(State::FreeBoundary);
    }
    let (_, o01, o02) = sides(&f1, &f2, &e[0])?;
    let (i, o11, o12) = sides(&f3, &f4, &e[1])?;
    let (j, o21, o22) = sides(&f5, &f6, &e[2])?;
    if o01 == o11 && o02 == o21 && o12 == o22 {
        return Ok(State::AllSame);
    }
    if o12 == o22 || i == 10 || j == 10 {
        return Ok(State::OnDiff);
    }
    Ok(State::OnSame)
}

/// `ChFi3d_evalconti(E, F1, F2)`: CN along the seam of a cone, a sphere or a torus, G1
/// otherwise; `evalconti(&e, &f, &f)?`.
pub fn evalconti(_e: &Edge, f1: &Face, f2: &Face) -> Result<Continuity, Failure> {
    if !f1.is_same(f2) {
        return Ok(Continuity::G1);
    }
    let mut f = f1.clone();
    f.set_orientation(Orientation::Forward);
    let s = BRepAdaptorSurface::with_face(&f, false)?;
    match s.get_type()? {
        SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus => Ok(Continuity::CN),
        _ => Ok(Continuity::G1),
    }
}

/// `ChFi3d_KParticular(Spine, IE, S1, S2)`: whether the fillet of the edge `IE` is a
/// particular case built by ChFiKPart: a constant radius between a plane and a plane,
/// cylinder or cone along a line or circle placed on their axes;
/// `kparticular(&spine, 1, &s1, &s2)?`.
pub fn kparticular(
    spine: &SpineHandle,
    ie: i32,
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
) -> Result<bool, Failure> {
    let mut guard = lock(spine);
    if let Some(fs) = guard.as_fil_spine()
        && !fs.is_constant_on_edge(ie)?
    {
        return Ok(false);
    }
    let (st1, st2) = (s1.get_type()?, s2.get_type()?);
    if st1 != SurfaceType::Plane && st2 != SurfaceType::Plane {
        return Ok(false);
    }
    let (plane, other) = if st1 == SurfaceType::Plane {
        (s1, s2)
    } else {
        (s2, s1)
    };
    let other_type = other.get_type()?;
    if !matches!(
        other_type,
        SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Cone
    ) {
        return Ok(false);
    }
    let ct = guard.spine_mut().current_elementary_spine(ie)?.get_type()?;
    if ct != CurveType::Line && ct != CurveType::Circle {
        return Ok(false);
    }
    particular_axes(plane, other, other_type, ct)
}

/// The axis conditions of `ChFi3d_KParticular` on the plane and the other surface.
fn particular_axes(
    plane: &BRepAdaptorSurface,
    other: &BRepAdaptorSurface,
    other_type: SurfaceType,
    ct: CurveType,
) -> Result<bool, Failure> {
    let pa = precision::angular();
    match other_type {
        SurfaceType::Plane => Ok(ct == CurveType::Line),
        SurfaceType::Cylinder => {
            let d1 = plane.plane()?.axis().direction();
            let d2 = other.cylinder()?.axis().direction();
            Ok((ct == CurveType::Line && d1.is_normal(d2, pa))
                || (ct == CurveType::Circle && d1.is_parallel(d2, pa)))
        }
        _ => {
            let d1 = plane.plane()?.axis().direction();
            let d2 = other.cone()?.axis().direction();
            Ok(ct == CurveType::Circle && d1.is_parallel(d2, pa))
        }
    }
}
