//! BRepLib wire and solid probe operations (`BRepLib_MakePolygon`, `BRepLib_MakeWire`,
//! `BRepLib_MakeSolid`), the Rust twin of tools/oracle/tktopalgo_probe_breplib_wire.cxx
//! (the request layout is documented there).
use super::brep_lib_common::{circ, shape, vertex_ref, vertices};
use super::brep_lib_edge::{PrecisionScope, vertex_index};
use super::common::{Reader, attempt, end, flag, int};
use super::face::{TpFace, tp_face};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{CompSolid, Edge, Shape, ShapeIterator, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction3, Vector3};
use rustycad_tktopalgo::brep_lib::{BRepLibMakePolygon, BRepLibMakeSolid, BRepLibMakeWire};

/// `tp_edges`.
pub fn edges(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<Vec<Edge>, Failure> {
    let n = r.integer();
    let builder = BRepBuilder::default();
    let mut result = Vec::new();
    for _ in 0..n {
        let kind = r.integer();
        let (vf, vl) = (vertex_ref(r, vs), vertex_ref(r, vs));
        let mut e = Edge::default();
        if kind == 0 {
            let (a, b) = (BRepTool::pnt(&vf)?, BRepTool::pnt(&vl)?);
            let d = Direction3::from_vector(Vector3::between(a, b))?;
            let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(a, d)).cast();
            builder.make_edge_with_curve(&mut e, &line, 1e-7)?;
            builder.range(&e, 0., a.distance(&b), false)?;
        } else if kind == 1 {
            let c = circ(r)?;
            let (a1, a2) = (r.real(), r.real());
            let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(c)).cast();
            builder.make_edge_with_curve(&mut e, &circle, 1e-7)?;
            builder.range(&e, a1, a2, false)?;
        } else {
            builder.make_edge(&mut e)?;
            builder.degenerated(&e, true)?;
        }
        if !vf.is_null() {
            builder.add(&mut e, &vf.oriented(Orientation::Forward))?;
        }
        if !vl.is_null() {
            builder.add(&mut e, &vl.oriented(Orientation::Reversed))?;
        }
        result.push(e);
    }
    Ok(result)
}

/// `tp_edge_ref`: `index orientation`.
pub fn edge_ref(r: &mut Reader<'_>, es: &[Edge]) -> Edge {
    let i = r.integer() as usize;
    Edge::unchecked(es[i].oriented(Orientation::ALL[r.integer() as usize]))
}

/// `tp_wire_of`: `n refs`, a null wire for a negative count.
pub fn wire_of(r: &mut Reader<'_>, es: &[Edge]) -> Result<Wire, Failure> {
    let mut w = Wire::default();
    let n = r.integer();
    if n < 0 {
        return Ok(w);
    }
    let builder = BRepBuilder::default();
    builder.make_wire(&mut w);
    for _ in 0..n {
        builder.add(&mut w, &edge_ref(r, es))?;
    }
    Ok(w)
}

/// `tp_polygon_first` over a polygon maker with the BRepLib member names (the
/// BRepBuilderAPI probe instantiates it on its maker, as the native template).
macro_rules! polygon_first_fn {
    ($name:ident, $maker:ident) => {
        fn $name(r: &mut Reader<'_>, vs: &[Vertex]) -> Result<$maker, Failure> {
            let ctor = r.integer();
            if ctor == 0 {
                return Ok($maker::new());
            }
            if ctor <= 3 {
                let (p1, p2) = (r.pnt(), r.pnt());
                if ctor == 1 {
                    return $maker::from_two_points(p1, p2);
                }
                let p3 = r.pnt();
                if ctor == 2 {
                    return $maker::from_three_points(p1, p2, p3, r.flag());
                }
                let p4 = r.pnt();
                return $maker::from_four_points(p1, p2, p3, p4, r.flag());
            }
            let (v1, v2) = (vertex_ref(r, vs), vertex_ref(r, vs));
            if ctor == 4 {
                return $maker::from_two_vertices(&v1, &v2);
            }
            let v3 = vertex_ref(r, vs);
            if ctor == 5 {
                return $maker::from_three_vertices(&v1, &v2, &v3, r.flag());
            }
            let v4 = vertex_ref(r, vs);
            $maker::from_four_vertices(&v1, &v2, &v3, &v4, r.flag())
        }
    };
}
pub(crate) use polygon_first_fn;

polygon_first_fn!(polygon_first, BRepLibMakePolygon);

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
    Ok(())
}
/// `blm_polygon`.
pub fn polygon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let _scope = PrecisionScope::new(r.real());
    attempt(|| {
        let vs = vertices(&mut r)?;
        polygon_body(&mut r, &vs)
    });
    end();
}

/// `tp_wire_first` over a wire maker with the BRepLib member names.
macro_rules! wire_first_fn {
    ($name:ident, $maker:ident) => {
        fn $name(r: &mut Reader<'_>, es: &[Edge], known: &mut bool) -> Result<$maker, Failure> {
            let ctor = r.integer();
            *known = true;
            if ctor == 0 {
                return Ok($maker::new());
            }
            if ctor == 5 || ctor == 6 {
                let w = wire_of(r, es)?;
                *known = w.nb_children() > 0;
                if ctor == 5 {
                    return $maker::from_wire(&w);
                }
                *known = true;
                return $maker::from_wire_edge(&w, &edge_ref(r, es));
            }
            let e1 = edge_ref(r, es);
            if ctor == 1 {
                return $maker::from_edge(&e1);
            }
            let e2 = edge_ref(r, es);
            if ctor == 2 {
                return $maker::from_two_edges(&e1, &e2);
            }
            let e3 = edge_ref(r, es);
            if ctor == 3 {
                return $maker::from_three_edges(&e1, &e2, &e3);
            }
            $maker::from_four_edges(&e1, &e2, &e3, &edge_ref(r, es))
        }
    };
}
pub(crate) use wire_first_fn;

wire_first_fn!(wire_first, BRepLibMakeWire);

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
        let op = r.integer();
        if op == 0 {
            m.add_edge(&edge_ref(r, &es))?;
            known = true;
        } else if op == 2 {
            let k = r.integer();
            let mut list: List<Shape> = List::default();
            for _ in 0..k {
                list.append(edge_ref(r, &es).into_shape());
            }
            m.add_list(&list)?;
            known = true;
        } else {
            let w = wire_of(r, &es)?;
            known = known || w.nb_children() > 0;
            m.add_wire(&w)?;
        }
        flag(m.is_done());
        if known {
            int(m.error() as i32);
        }
    }
    attempt(|| shape(&m.wire()?.into_shape()));
    shape(m.edge())?;
    shape(m.vertex())?;
    int(vertex_index(m.vertex(), vs));
    Ok(())
}
/// `blm_wire`.
pub fn wire_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let vs = vertices(&mut r)?;
        wire_body(&mut r, &vs)
    });
    end();
}

/// `tp_shells`.
pub fn shells(r: &mut Reader<'_>, faces: &[TpFace]) -> Result<Vec<Shell>, Failure> {
    let n = r.integer();
    let builder = BRepBuilder::default();
    let mut result = Vec::new();
    for _ in 0..n {
        let mut s = Shell::default();
        builder.make_shell(&mut s);
        let count = r.integer();
        for _ in 0..count {
            builder.add(&mut s, &faces[r.integer() as usize].face)?;
        }
        s.set_closed(BRepTool::is_closed(&s)?)?;
        result.push(s);
    }
    Ok(result)
}
/// `tp_solid_of`.
pub fn solid_of(r: &mut Reader<'_>, shells: &[Shell]) -> Result<Solid, Failure> {
    let mut s = Solid::default();
    let builder = BRepBuilder::default();
    builder.make_solid(&mut s);
    let n = r.integer();
    for _ in 0..n {
        builder.add(&mut s, &shells[r.integer() as usize])?;
    }
    Ok(s)
}
/// `tp_solid_first` over a solid maker with the BRepLib member names.
macro_rules! solid_first_fn {
    ($name:ident, $maker:ident) => {
        fn $name(ctor: i32, r: &mut Reader<'_>, shells: &[Shell]) -> Result<$maker, Failure> {
            let builder = BRepBuilder::default();
            match ctor {
                0 => return $maker::new(),
                1 => {
                    let mut cs = CompSolid::default();
                    builder.make_comp_solid(&mut cs);
                    let n = r.integer();
                    for _ in 0..n {
                        let mut s = Solid::default();
                        builder.make_solid(&mut s);
                        builder.add(&mut s, &shells[r.integer() as usize])?;
                        builder.add(&mut cs, &s)?;
                    }
                    return $maker::from_comp_solid(&cs);
                }
                5 => return $maker::from_solid(&solid_of(r, shells)?),
                6 => {
                    let so = solid_of(r, shells)?;
                    return $maker::from_solid_shell(&so, &shells[r.integer() as usize]);
                }
                _ => {}
            }
            let s1 = &shells[r.integer() as usize];
            if ctor == 2 {
                return $maker::from_shell(s1);
            }
            let s2 = &shells[r.integer() as usize];
            if ctor == 3 {
                return $maker::from_two_shells(s1, s2);
            }
            $maker::from_three_shells(s1, s2, &shells[r.integer() as usize])
        }
    };
}
pub(crate) use solid_first_fn;

solid_first_fn!(solid_first, BRepLibMakeSolid);
/// `tp_merged_solid`: the order-free digest of a solid built from a compsolid.
pub fn merged_solid(so: &Solid, faces: &[TpFace]) -> Result<(), Failure> {
    int(so.nb_children());
    let it = ShapeIterator::new(so, true, true);
    let Ok(shell) = it.value() else { return Ok(()) };
    int(shell.nb_children());
    flag(shell.closed()?);
    for f in faces {
        let mut found = false;
        let mut fit = ShapeIterator::new(shell, true, true);
        while let Ok(child) = fit.value() {
            found = found || child.is_same(&f.face);
            fit.next();
        }
        flag(found);
    }
    Ok(())
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
        int(m.face_status(&f.face) as i32);
    }
    Ok(())
}
/// `blm_solid`.
pub fn solid_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| solid_body(&mut r));
    end();
}
