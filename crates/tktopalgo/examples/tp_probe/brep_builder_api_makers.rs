//! BRepBuilderAPI maker probe operations (the package class, MakeVertex, MakeEdge,
//! MakeEdge2d, MakePolygon, MakeWire, MakeFace, MakeShell, MakeSolid), the Rust twin of
//! tools/oracle/tktopalgo_probe_brepbuilderapi_makers.cxx (the request layout is
//! documented there). The first-call readers are the BRepLib probe's, instantiated on the
//! BRepBuilderAPI makers as the native templates are.
use super::brep_lib_common::{
    circ, circ2d, elips, elips2d, geom_curve, geom2d_curve, hypr, hypr2d, lin, lin2d, parab,
    parab2d, shape, vertex_ref, vertices,
};
use super::brep_lib_edge::{
    PlaneScope, PrecisionScope, Tail, edge_dispatch, edge_result, gp_ctor, point2, point3, tail,
    vertex_index,
};
use super::brep_lib_face::{face_request_fn, gp_face_ctor, surface2};
use super::brep_lib_package::guarded;
use super::brep_lib_shapes::shape_record;
use super::brep_lib_wire::{
    edge_ref, edges, merged_solid, polygon_first_fn, shells, solid_first_fn, solid_of,
    wire_first_fn, wire_of,
};
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real};
use super::face::{surface, tp_face};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, CompSolid, Edge, Face, Shape, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomElementarySurface, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    self, BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeEdge2d, BRepBuilderAPIMakeFace,
    BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShell,
    BRepBuilderAPIMakeSolid, BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_lib;

edge_dispatch!(BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeEdge2d);
polygon_first_fn!(polygon_first, BRepBuilderAPIMakePolygon);
wire_first_fn!(wire_first, BRepBuilderAPIMakeWire);
solid_first_fn!(solid_first, BRepBuilderAPIMakeSolid);
face_request_fn!(gp_face, face_request, BRepBuilderAPIMakeFace, face_maker);

/// `tp_bba_history`: Generated and Modified counts and IsDeleted of a built shape.
fn history(m: &mut dyn BRepBuilderAPIMakeShape, s: &Shape) {
    attempt(|| {
        int(m.generated(s)?.extent());
        Ok(())
    });
    attempt(|| {
        int(m.modified(s)?.extent());
        Ok(())
    });
    attempt(|| {
        flag(m.is_deleted(s)?);
        Ok(())
    });
}

/// `tp_bba_made`: the virtual IsDone, Check, and Shape() (its digest when `digest`, else
/// its null flag) with its history.
pub fn made(m: &mut dyn BRepBuilderAPIMakeShape, digest: bool) {
    flag(m.is_done());
    attempt(|| {
        m.check()?;
        int(1);
        Ok(())
    });
    attempt(|| {
        let s = m.shape()?.clone();
        if digest {
            shape(&s)?;
        } else {
            flag(s.is_null());
        }
        if !s.is_null() {
            history(m, &s);
        }
        Ok(())
    });
}

/// `bba_vertex`.
pub fn vertex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    let p = r.pnt();
    attempt(|| {
        let mut m = BRepBuilderAPIMakeVertex::new(p)?;
        made(&mut m, true);
        shape(&m.vertex()?.into_shape())
    });
    end();
}

/// `bba_package`: the setters when requested, then both precisions and the plane; the
/// defaults are restored after.
pub fn package_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let prec = r.real();
    if prec > 0. {
        brep_builder_api::set_precision(prec);
    }
    attempt(|| {
        if r.flag() {
            let p = r.pnt();
            brep_builder_api::set_plane(&Handle::new(GeomPlane::from_point(p, r.dir())?));
        }
        real(brep_builder_api::precision());
        real(brep_lib::precision());
        let plane = brep_builder_api::plane();
        flag(plane.object_address() == brep_lib::plane().object_address());
        let frame = plane
            .get()
            .map(GeomElementarySurface::position)
            .unwrap_or_default();
        pnt(frame.location());
        dir(frame.direction());
        dir(frame.x_direction());
        Ok(())
    });
    brep_builder_api::set_precision(precision::confusion());
    brep_builder_api::set_plane(&Handle::default());
    end();
}

fn edge_body(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<(), Failure> {
    let n = r.integer();
    let first = r.integer();
    let mut m = edge_first(first, r, vs)?;
    for _ in 1..n {
        let id = r.integer();
        edge_init(&mut m, id, r, vs)?;
    }
    edge_result!(m, true, vs);
    made(&mut m, true);
    Ok(())
}
/// `bba_edge`.
pub fn edge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    attempt(|| {
        let vs = vertices(&mut r)?;
        edge_body(&mut r, &vs)
    });
    end();
}

fn edge2d_body(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<(), Failure> {
    let n = r.integer();
    let first = r.integer();
    let mut m = edge2d_first(first, r, vs)?;
    for _ in 1..n {
        let id = r.integer();
        let c = geom2d_curve(r)?;
        init_curve2d(&mut m, &c, tail(id - 30, r, vs, point2))?;
    }
    let error_known = !m.is_done();
    edge_result!(m, error_known, vs);
    made(&mut m, true);
    Ok(())
}
/// `bba_edge2d`.
pub fn edge2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    attempt(|| {
        let _plane = PlaneScope::new(&mut r)?;
        let vs = vertices(&mut r)?;
        edge2d_body(&mut r, &vs)
    });
    end();
}

fn polygon_body(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<(), Failure> {
    let mut m = polygon_first(r, vs)?;
    flag(m.added());
    flag(m.is_done());
    let n = r.integer();
    for _ in 0..n {
        match r.integer() {
            0 => m.add_point(r.pnt())?,
            1 => m.add_vertex(&vertex_ref(r, vs))?,
            _ => m.close()?,
        }
        flag(m.added());
        flag(m.is_done());
    }
    shape(m.first_vertex())?;
    int(vertex_index(m.first_vertex(), vs));
    shape(m.last_vertex())?;
    int(vertex_index(m.last_vertex(), vs));
    shape(m.edge())?;
    attempt(|| shape(&m.wire()?.into_shape()));
    made(&mut m, true);
    Ok(())
}
/// `bba_polygon`.
pub fn polygon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    attempt(|| {
        let vs = vertices(&mut r)?;
        polygon_body(&mut r, &vs)
    });
    end();
}

/// One Add of `bba_wire`; `known` tells whether Error is set.
fn wire_add(
    m: &mut BRepBuilderAPIMakeWire,
    r: &mut Reader<'_>,
    es: &[Edge],
    known: &mut bool,
) -> Result<(), Failure> {
    let op = r.integer();
    if op == 0 {
        m.add_edge(&edge_ref(r, es))?;
        *known = true;
    } else if op == 2 {
        let k = r.integer();
        let mut list: List<Shape> = List::default();
        for _ in 0..k {
            list.append(edge_ref(r, es).into_shape());
        }
        m.add_list(&list)?;
        *known = true;
    } else {
        let w = wire_of(r, es)?;
        *known = *known || w.nb_children() > 0;
        m.add_wire(&w)?;
    }
    Ok(())
}
fn wire_body(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<(), Failure> {
    let es = edges(r, vs)?;
    let mut known = false;
    let mut m = wire_first(r, &es, &mut known)?;
    flag(m.is_done());
    if known {
        int(m.error() as i32);
    }
    let n = r.integer();
    for _ in 0..n {
        wire_add(&mut m, r, &es, &mut known)?;
        flag(m.is_done());
        if known {
            int(m.error() as i32);
        }
    }
    attempt(|| shape(&m.wire()?.into_shape()));
    shape(m.edge())?;
    shape(m.vertex())?;
    int(vertex_index(m.vertex(), vs));
    made(&mut m, true);
    Ok(())
}
/// `bba_wire`.
pub fn wire_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let vs = vertices(&mut r)?;
        wire_body(&mut r, &vs)
    });
    end();
}

fn solid_body(r: &mut Reader<'_>) -> Result<(), Failure> {
    let nfaces = r.integer();
    let mut faces = Vec::new();
    for _ in 0..nfaces {
        faces.push(tp_face(r)?);
    }
    let shells = shells(r, &faces)?;
    let ctor = r.integer();
    let mut m = solid_first(ctor, r, &shells)?;
    let n = r.integer();
    for _ in 0..n {
        m.add(&shells[r.integer() as usize])?;
    }
    flag(m.is_done());
    let solid: Shape = m.solid()?.into_shape();
    if ctor == 1 {
        merged_solid(&Solid::unchecked(solid), &faces)?;
    } else {
        shape(&solid)?;
    }
    for f in &faces {
        flag(m.is_deleted(&f.face)?);
    }
    made(&mut m, ctor != 1);
    Ok(())
}
/// `bba_solid`.
pub fn solid_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| solid_body(&mut r));
    end();
}

/// `tp_bba_face_maker`: IsDone, Error and the face.
fn face_maker(m: &BRepBuilderAPIMakeFace) {
    flag(m.is_done());
    int(m.error() as i32);
    attempt(|| shape(&m.face()?.into_shape()));
}
/// `bba_face`.
pub fn face_case(values: &[f64]) {
    guarded(values, |r| {
        match face_request(r)? {
            Some(mut m) => {
                face_maker(&m);
                made(&mut m, true);
            }
            None => int(-2),
        }
        Ok(())
    });
}

/// `bba_shell`.
pub fn shell_case(values: &[f64]) {
    guarded(values, |r| {
        let s = surface(r)?;
        let bounded = r.flag();
        let mut b = (0., 0., 0., 0.);
        if bounded {
            b = (r.real(), r.real(), r.real(), r.real());
        }
        let (segment, mode) = (r.flag(), r.integer());
        let mut m = if mode == 1 {
            let mut m = BRepBuilderAPIMakeShell::new();
            flag(m.is_done());
            int(m.error() as i32);
            if !bounded {
                b = s.get().map(|g| g.bounds()).unwrap_or_default();
            }
            m.init(&s, b, segment)?;
            m
        } else if bounded {
            BRepBuilderAPIMakeShell::with_bounds(&s, b, segment)?
        } else {
            BRepBuilderAPIMakeShell::with_surface(&s, segment)?
        };
        flag(m.is_done());
        int(m.error() as i32);
        shape(&m.shell()?.into_shape())?;
        made(&mut m, true);
        Ok(())
    });
}
