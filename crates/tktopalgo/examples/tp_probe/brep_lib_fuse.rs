//! BRepLib_FuseEdges probe operation and the shape records 10 and 11, the Rust twin of
//! tools/oracle/tktopalgo_probe_breplib_fuse.cxx (the request layout is documented there).
use super::brep_lib_common::{geom_curve, shape};
use super::brep_lib_package::{guarded, sub_shapes};
use super::brep_lib_shapes::shape_record;
use super::common::{Reader, flag, int};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_lib::{BRepLibFuseEdges, BRepLibMakeEdge, BRepLibMakeFace};

/// `tp_shape_record_fuse`: the shape records 10 and 11.
pub fn shape_record_fuse(r: &mut Reader<'_>, kind: i32) -> Result<Shape, Failure> {
    if kind == 10 {
        let w = topods::wire(&shape_record(r)?)?;
        let m = BRepLibMakeFace::from_wire(&w, r.flag())?;
        if !m.is_done() {
            return Ok(Shape::default());
        }
        return Ok(m.face()?.into_shape());
    }
    let builder = BRepBuilder::default();
    let c = geom_curve(r)?;
    let curve = c.get().expect("a curve");
    let nv = r.integer();
    let mut vs: Vec<Vertex> = Vec::new();
    for _ in 0..nv {
        let mut v = Vertex::default();
        builder.make_vertex_at(&mut v, curve.value(r.real())?, 1e-7)?;
        vs.push(v);
    }
    let mut w = Wire::default();
    builder.make_wire(&mut w);
    for _ in 0..r.integer() {
        let (a, b) = (r.integer() as usize, r.integer() as usize);
        let (p1, p2) = (r.real(), r.real());
        let m = BRepLibMakeEdge::from_curve_vertices_params(&c, &vs[a], &vs[b], p1, p2)?;
        if m.is_done() {
            builder.add(&mut w, &m.edge()?.into_shape())?;
        }
    }
    Ok(w.into_shape())
}

/// `tp_fuse_chains`: the chain count.
fn fuse_chains(fe: &mut BRepLibFuseEdges, edges: &IndexedMapOfShape) -> Result<i32, Failure> {
    let m = fe.edges()?;
    let count = m.extent() as i32;
    int(count);
    for k in 1..=count {
        let Ok(l) = m.find(&k) else {
            int(-1);
            continue;
        };
        int(l.extent());
        for e in l.iter() {
            int(edges.find_index(e) as i32);
            int(e.orientation() as i32);
        }
    }
    Ok(count)
}

/// `tp_fuse_results`.
fn fuse_results(fe: &mut BRepLibFuseEdges, s: &Shape, count: i32) -> Result<(), Failure> {
    let re = fe.result_edges()?;
    for k in 1..=count {
        match re.find(&k) {
            Ok(e) => shape(e)?,
            Err(_) => int(-1),
        }
    }
    shape(fe.shape()?)?;
    let fm = fe.faces()?;
    let faces = sub_shapes(s, ShapeEnum::Face)?;
    for i in 1..=faces.extent() {
        flag(fm.is_bound(faces.find_key(i)?));
    }
    Ok(())
}

/// `blp_fuseedges`.
pub fn fuseedges_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let concat = r.flag();
        let n = r.integer();
        let edges = sub_shapes(&s, ShapeEnum::Edge)?;
        let mut avoid = IndexedMapOfShape::default();
        for _ in 0..n {
            avoid.add(edges.find_key(r.integer() as usize)?.clone());
        }
        let mode = r.integer();
        let mut fe = BRepLibFuseEdges::new(&s, false)?;
        if concat {
            fe.set_concat_bspl(true);
        }
        fe.avoid_edges(&avoid);
        match mode {
            1 => {
                fe.perform()?;
                shape(fe.shape()?)
            }
            2 => {
                int(fe.nb_vertices()?);
                shape(fe.shape()?)
            }
            _ => {
                let count = fuse_chains(&mut fe, &edges)?;
                int(fe.nb_vertices()?);
                fuse_results(&mut fe, &s, count)
            }
        }
    });
}
