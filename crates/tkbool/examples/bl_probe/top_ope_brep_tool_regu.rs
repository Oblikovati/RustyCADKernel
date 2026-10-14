//! TopOpeBRepTool CLASSI, REGUW, CORRISO, FuseEdges and PurgeInternalEdges probe
//! operations, the Rust twin of tools/oracle/tkbool_probe_topopebreptool_regu.cxx (the
//! records are documented there).
use super::brep_fill_generator::failed;
use super::common::{Reader, attempt, end, flag, int, real};
use super::shape::shape as pr_shape;
use super::top_ope_brep_tool_tool::solid;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbool::top_ope_brep_tool::*;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Plane3, Point3, Vector2};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPICopy, BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex,
};

/// `bl_tt_plane`.
fn plane() -> Result<Face, Failure> {
    BRepBuilderAPIMakeFace::from_plane(Plane3::default())?.face()
}

/// `bl_tt_squares`.
fn squares(r: &mut Reader<'_>) -> Result<Vec<Shape>, Failure> {
    let n = r.integer();
    let mut w = Vec::new();
    for _ in 0..n {
        let (x0, y0, x1, y1, ccw) = (r.real(), r.real(), r.real(), r.real(), r.flag());
        let p = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
        let mut poly = BRepBuilderAPIMakePolygon::new();
        for k in 0..4 {
            let (x, y) = p[if ccw { k } else { 3 - k }];
            poly.add_point(Point3::new(x, y, 0.0))?;
        }
        poly.close()?;
        w.push(poly.wire()?.into_shape());
    }
    Ok(w)
}

/// `bl_tt_index`.
fn index_of(w: &[Shape], s: &Shape) -> i32 {
    w.iter()
        .position(|x| x.is_same(s))
        .map_or(0, |i| i as i32 + 1)
}

/// `tt_classi squares`.
pub fn classi_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let w = match squares(&mut r) {
        Ok(w) => w,
        Err(error) => return failed(error),
    };
    let mut c = TopOpeBRepToolCLASSI::new();
    c.init2d(&plane().expect("plane face"));
    for i in 0..w.len() {
        for j in 0..w.len() {
            if i == j {
                continue;
            }
            attempt(|| {
                let b = c.classi_bnd2d(&w[i], &w[j], 1.0e-7, true)?;
                int(b);
                int(c.classi_bnd2d(&w[i], &w[j], 1.0e-7, false)?);
                int(c.classip2d(&w[i], &w[j], b)?);
                Ok(())
            });
        }
    }
    attempt(|| {
        let mut l: List<Shape> = List::default();
        w.iter().for_each(|s| {
            l.append(s.clone());
        });
        let mut m = ShapeListMap::default();
        flag(c.classilist(&l, &mut m)?);
        for s in &w {
            let found = m.seek(s);
            flag(found.is_some());
            if let Some(lw) = found {
                int(lw.extent());
                lw.iter().for_each(|x| int(index_of(&w, x)));
            }
        }
        Ok(())
    });
    end();
}

/// The face of the closed wire of `tt_regularize`.
fn walk_face(points: &[Vertex], ids: &[usize]) -> Result<Face, Failure> {
    let bb = BRepBuilder::default();
    let mut w = Wire::default();
    bb.make_wire(&mut w);
    let m = ids.len();
    for i in 0..m {
        let e = BRepBuilderAPIMakeEdge::from_vertices(&points[ids[i]], &points[ids[(i + 1) % m]])?
            .edge()?;
        bb.add(&mut w, &e)?;
    }
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &w, true)?.face()
}

/// `bl_tt_sorted_faces`.
fn sorted_faces(l: &List<Shape>) -> Result<(), Failure> {
    let mut rows: Vec<Vec<f64>> = Vec::new();
    for f in l.iter() {
        let mut b = BoundingBox3::default();
        BRepBndLib::add(f, &mut b, true)?;
        let mut row = b.get()?.to_vec();
        let nw = Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)?.count();
        let ne = Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?.count();
        row.extend([nw as f64, ne as f64, f.orientation() as i32 as f64]);
        rows.push(row);
    }
    // std::sort of std::vector<double>: lexicographic with `<`.
    rows.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    for row in rows {
        row[..6].iter().copied().for_each(real);
        row[6..].iter().for_each(|v| int(*v as i32));
    }
    Ok(())
}

/// `tt_regularize n (x y)... m (i)...`.
pub fn regularize_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let points: Vec<Vertex> = (0..n)
        .map(|_| {
            let x = r.real();
            BRepBuilderAPIMakeVertex::new(Point3::new(x, r.real(), 0.0))
                .and_then(|m| m.vertex())
                .expect("vertex")
        })
        .collect();
    let m = r.integer();
    let ids: Vec<usize> = (0..m).map(|_| r.integer() as usize).collect();
    let face = match walk_face(&points, &ids) {
        Ok(f) => f,
        Err(error) => return failed(error),
    };
    attempt(|| {
        let (mut ow, mut es) = (ShapeListMap::default(), ShapeListMap::default());
        flag(regularize_wires(&face, &mut ow, &mut es)?);
        int(ow.extent() as i32);
        for w in Explorer::new(&face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            int(ow.seek(&w).map_or(-1, |l| l.extent()));
        }
        int(es.extent() as i32);
        Ok(())
    });
    attempt(|| {
        let (mut lof, mut es) = (List::default(), ShapeListMap::default());
        flag(regularize(&face, &mut lof, &mut es)?);
        int(lof.extent());
        sorted_faces(&lof)
    });
    end();
}

/// The first face of the solid on a U-periodic surface.
fn periodic_face(s: &Shape) -> Result<Option<Face>, Failure> {
    for fs in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        let f = topods::face(&fs)?;
        if BRepTool::surface(&f)?
            .get()
            .is_some_and(|su| su.is_u_periodic())
        {
            return Ok(Some(f));
        }
    }
    Ok(None)
}

/// The CORRISO queries of `tt_corriso`.
fn corriso_queries(c: &mut TopOpeBRepToolCORRISO, fin: &Face, fsp: &Face) {
    attempt(|| {
        for i in 1..=2 {
            let (closed, p) = c.refclosed(i).unwrap_or((false, -1.0));
            flag(closed);
            real(p);
        }
        real(c.tol(1, 1.0e-7)?);
        real(c.tol(2, 1.0e-7)?);
        Ok(())
    });
    attempt(|| {
        flag(c.init(fsp)?);
        int(c.eds().extent());
        flag(c.uv_closed()?);
        Ok(())
    });
    attempt(|| {
        for es in c.eds().iter() {
            let e = topods::edge(es)?;
            attempt(|| {
                let (ok, iv) = c.edge_with_faulty_uv(&e)?;
                flag(ok);
                int(iv);
                Ok(())
            });
            attempt(|| {
                let (code, sp) = c.edge_out_of_bounds_uv(&e, true, 1.0e-7)?;
                int(code);
                real(sp);
                Ok(())
            });
        }
        Ok(())
    });
    attempt(|| {
        let mut m = EdgeFlagMap::default();
        let eds = c.eds().clone();
        flag(c.edges_with_faulty_uv(&eds, 3, &mut m, false)?);
        int(m.extent() as i32);
        flag(c.edges_out_of_bounds_uv(&eds, true, 1.0e-7, &mut m)?);
        int(m.extent() as i32);
        Ok(())
    });
    attempt(|| {
        let (mut cl, mut fy): (List<Shape>, List<Shape>) = (List::default(), List::default());
        for es in c.eds().iter() {
            if BRepTool::is_closed_on_face(&topods::edge(es)?, fin)? {
                cl.append(es.clone());
            }
        }
        int(cl.extent());
        flag(c.purge_fy_closing_e(&cl, &mut fy)?);
        int(fy.extent());
        Ok(())
    });
}

/// The purge and correction of `tt_corriso`.
fn corriso_purge(fin: &Face, fsp: &Face) {
    attempt(|| {
        let old: DataMap<Shape, i32, ShapeMapHasher> = DataMap::default();
        let mut nok: IndexedMap<Shape> = IndexedMap::default();
        let mut lof: List<Shape> = List::default();
        lof.append(fsp.clone().into_shape());
        flag(purge_closing_edges(fin, &lof, &old, &mut nok)?);
        int(nok.extent() as i32);
        let mut loff: List<Shape> = List::default();
        flag(make_faces(fin, &lof, &nok, &mut loff)?);
        int(loff.extent());
        for f in loff.iter() {
            pr_shape(f)?;
        }
        Ok(())
    });
    attempt(|| {
        let mut g = fsp.clone();
        flag(correct_on_uviso(fin, &mut g)?);
        flag(g.is_same(fsp));
        let mut c2 = TopOpeBRepToolCORRISO::with_face(fin)?;
        flag(c2.init(&g)?);
        flag(c2.uv_closed()?);
        Ok(())
    });
}

/// `tt_corriso solid ie du dv`.
pub fn corriso_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = match solid(&mut r) {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let ie = r.integer();
    let t = Vector2::new(r.real(), r.real());
    let Some(fin) = periodic_face(&s).expect("faces") else {
        int(-1);
        return end();
    };
    let mut copy = BRepBuilderAPICopy::with_shape(&fin, true, false).expect("copy");
    let fsp = topods::face(copy.shape().expect("copy shape")).expect("face");
    attempt(|| {
        for (k, es) in Explorer::new(&fsp, ShapeEnum::Edge, ShapeEnum::Shape)?.enumerate() {
            if k as i32 + 1 == ie {
                flag(TopOpeBRepToolTOOL::trsl_uv_modif_e(
                    t,
                    &fsp,
                    &topods::edge(&es)?,
                )?);
                break;
            }
        }
        Ok(())
    });
    let mut c = TopOpeBRepToolCORRISO::with_face(&fin).expect("corriso");
    corriso_queries(&mut c, &fin, &fsp);
    corriso_purge(&fin, &fsp);
    end();
}

/// `tt_fuse_edges solid1 solid2`.
pub fn fuse_edges_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fused = solid(&mut r).and_then(|s1| {
        let s2 = solid(&mut r)?;
        let fuse = BRepAlgoAPIFuse::from_shapes(&s1, &s2)?;
        Ok(fuse.shape().clone())
    });
    let s = match fused {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    attempt(|| {
        let mut fe = TopOpeBRepToolFuseEdges::new(&s, false)?;
        int(fe.edges()?.extent() as i32);
        int(fe.nb_vertices()?);
        int(fe.result_edges()?.extent() as i32);
        int(fe.faces()?.extent() as i32);
        pr_shape(fe.shape()?)
    });
    end();
}

/// `bl_tt_vertex_at`.
fn vertex_at(s: &Shape, p: Point3) -> Result<Vertex, Failure> {
    for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let v = topods::vertex(&v)?;
        if BRepTool::pnt(&v)?.distance(&p) < 1.0e-9 {
            return Ok(v);
        }
    }
    Ok(Vertex::default())
}

/// `tt_purge_internal a`.
pub fn purge_internal_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let a = r.real();
    attempt(|| {
        let mut poly = BRepBuilderAPIMakePolygon::new();
        for (x, y) in [(0.0, 0.0), (a, 0.0), (a, a), (0.0, a)] {
            poly.add_point(Point3::new(x, y, 0.0))?;
        }
        poly.close()?;
        let f = BRepBuilderAPIMakeFace::from_wire(&poly.wire()?, false)?.face()?;
        let (v0, v2) = (
            vertex_at(&f, Point3::new(0.0, 0.0, 0.0))?,
            vertex_at(&f, Point3::new(a, a, 0.0))?,
        );
        let d = BRepBuilderAPIMakeEdge::from_vertices(&v0, &v2)?.edge()?;
        let mut w = Explorer::new(&f, ShapeEnum::Wire, ShapeEnum::Shape)?
            .current()
            .clone();
        if let Some(t) = w.tshape().get() {
            t.set_free(true);
        }
        BRepBuilder::default().add(&mut w, &d.oriented(Orientation::Internal))?;
        let mut p = TopOpeBRepToolPurgeInternalEdges::new(&f, true)?;
        int(p.nb_edges()?);
        int(p.faces()?.extent() as i32);
        flag(p.is_done());
        pr_shape(p.shape()?)
    });
    end();
}
