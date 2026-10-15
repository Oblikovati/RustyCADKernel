// Rust translation of the static ToFuse and NewParameter of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Generator.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether two faces (coplanar planes) or two edges (coaxial lines) fuse, whether an
//! edge fuses at a vertex of a face, and the parameter of a new vertex on an edge.
use crate::loc_ope::find_edges::basis_curve;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomLine, GeomPlane, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::elclib;
use std::f64::consts::PI;

/// The plane of the (unlocated) surface of `f`, a rectangular trimmed surface standing
/// for its basis, moved by the face location.
fn located_plane(f: &Face) -> Result<Option<rustycad_tkmath::gp::Plane3>, Failure> {
    let (mut s, loc) = BRepTool::surface_with_location(f)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(trimmed) = trimmed.get() {
        s = trimmed.basis_surface();
    }
    let plane: Handle<GeomPlane> = s.cast();
    Ok(plane
        .get()
        .map(|p| p.pln().transformed(loc.transformation())))
}

/// `ToFuse(F1, F2)`: both faces lie on coplanar planes (other surface types never
/// fuse).
pub(super) fn to_fuse_faces(f1: &Face, f2: &Face) -> Result<bool, Failure> {
    if f1.is_null() || f2.is_null() {
        return Ok(false);
    }
    let (Some(pl1), Some(pl2)) = (located_plane(f1)?, located_plane(f2)?) else {
        return Ok(false);
    };
    Ok(pl1
        .position()
        .is_coplanar(pl2.position(), precision::confusion(), precision::angular()))
}

/// `ToFuse(E1, E2)`: both edges lie on coaxial lines.
pub(super) fn to_fuse_edges(e1: &Edge, e2: &Edge) -> Result<bool, Failure> {
    if e1.is_null() || e2.is_null() {
        return Ok(false);
    }
    let (c1, _, _) = basis_curve(e1)?;
    let (c2, _, _) = basis_curve(e2)?;
    let (l1, l2) = (c1.cast::<GeomLine>(), c2.cast::<GeomLine>());
    let (Some(l1), Some(l2)) = (l1.get(), l2.get()) else {
        return Ok(false);
    };
    Ok(l1.lin().position().is_coaxial(
        l2.lin().position(),
        precision::angular(),
        precision::confusion(),
    ))
}

/// `ToFuse(E, F, V, toRemove)`: no other edge of `f` at the vertex `v` is kept.
pub(super) fn to_fuse_at(
    e: &Edge,
    f: &Face,
    v: &Vertex,
    to_remove: &MapOfShape,
) -> Result<bool, Failure> {
    for x in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if x.is_same(e) {
            continue;
        }
        let eee = rustycad_tkbrep::topods::edge(&x)?;
        let (vf, vl) = topexp::vertices(&eee, false);
        if (vf.is_same(v) || vl.is_same(v)) && !to_remove.contains(&x) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `NewParameter(Edg, Vtx, NewEdg, NewVtx)`: the parameter of `new_vtx` on the line or
/// circle of `edg` (0 for other curves); on a circle a parameter equal (to the epsilon
/// of 2 pi) to the one of the opposite end already on `new_edg` moves by a period.
pub(super) fn new_parameter(
    edg: &Edge,
    vtx: &Vertex,
    new_edg: &Edge,
    new_vtx: &Vertex,
) -> Result<f64, Failure> {
    let p = BRepTool::pnt(new_vtx)?;
    let (c, _, _) = basis_curve(edg)?;
    if let Some(line) = c.cast::<GeomLine>().get() {
        return Ok(elclib::line_parameter(line.lin().position(), p));
    }
    let circle = c.cast::<GeomCircle>();
    let Some(circle) = circle.get() else {
        return Ok(0.);
    };
    let mut prm = elclib::circle_parameter(circle.circ()?.position(), p);
    // Vtx vient d`une exploration de Edg orientee FORWARD
    let orient = top_abs::reverse(vtx.orientation());
    if orient != Orientation::Forward && orient != Orientation::Reversed {
        return Ok(prm);
    }
    let forward = new_edg.oriented(Orientation::Forward);
    let Some(x) = Explorer::new(&forward, ShapeEnum::Vertex, ShapeEnum::Shape)?
        .find(|x| x.orientation() == orient)
    else {
        return Ok(prm);
    };
    let prmmax = BRepTool::parameter(&rustycad_tkbrep::topods::vertex(&x)?, new_edg)?;
    if (prmmax - prm).abs() <= epsilon(2. * PI) {
        if orient == Orientation::Reversed {
            prm -= 2. * PI;
        } else {
            prm += 2. * PI;
        }
    }
    Ok(prm)
}
