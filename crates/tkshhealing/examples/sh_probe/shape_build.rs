//! ShapeBuild_Vertex and ShapeBuild_Edge probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_shapebuild.cxx (which documents the requests).
use super::brep_lib_common::{curve2d_points, shape, type_name};
use super::common::{Reader, attempt, end, flag, pnt, real};
use super::shapes::{TpShapes, tp_shapes};
use super::wire_data::{sh_edge, sh_face, sh_shape};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{Transform2, Vector2};
use rustycad_tkshhealing::shape_build::{ShapeBuildEdge, ShapeBuildVertex};

/// `sh_vertex mode p1 p2 tol1 tol2 factor`.
pub fn vertex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (p1, p2) = (r.pnt(), r.pnt());
    let (t1, t2, f) = (r.real(), r.real(), r.real());
    let made = || -> Result<Vertex, Failure> {
        if mode == 0 {
            return ShapeBuildVertex.combine_points(p1, p2, t1, t2, f);
        }
        let builder = BRepBuilder::default();
        let (mut a, mut b) = (Vertex::default(), Vertex::default());
        builder.make_vertex_at(&mut a, p1, t1)?;
        builder.make_vertex_at(&mut b, p2, t2)?;
        ShapeBuildVertex.combine_vertices(&a, &b, f)
    };
    let v = made().expect("probe vertex");
    pnt(BRepTool::pnt(&v).expect("probe vertex point"));
    real(BRepTool::vertex_tolerance(&v).expect("probe vertex tolerance"));
    end();
}

/// `sh_be_trsf`: none, a translation, a rotation or a scale.
fn trsf(r: &mut Reader<'_>) -> Transform2 {
    let mut t = Transform2::default();
    match r.integer() {
        1 => {
            let x = r.real();
            t.set_translation(Vector2::new(x, r.real()));
        }
        2 => {
            let p = r.pnt2d();
            t.set_rotation(p, r.real());
        }
        3 => {
            let p = r.pnt2d();
            t.set_scale(p, r.real());
        }
        _ => {}
    }
    t
}

fn pcurve(c: &Handle<dyn Geom2dCurve>, f: f64, l: f64) {
    type_name(c);
    real(f);
    real(l);
    if let Some(curve) = c.get() {
        curve2d_points(|u| curve.value(u), f, l);
    }
}

fn vertex_of(t: &TpShapes, i: i32) -> Result<Vertex, Failure> {
    topods::vertex(&sh_shape(t, i))
}

/// `sh_be_edit`.
fn edit(r: &mut Reader<'_>, t: &TpShapes, op: i32) -> Result<(), Failure> {
    let sbe = ShapeBuildEdge;
    match op {
        0 => {
            let e = sh_edge(t, r.integer())?;
            let (v1, v2) = (vertex_of(t, r.integer())?, vertex_of(t, r.integer())?);
            shape(&sbe.copy_replace_vertices(&e, &v1, &v2)?.into_shape())?;
            shape(&e)
        }
        1 | 3 => {
            let (to, from) = (sh_edge(t, r.integer())?, sh_edge(t, r.integer())?);
            if op == 1 {
                let a = r.real();
                sbe.copy_ranges(&to, &from, a, r.real())?;
            } else {
                sbe.copy_pcurves(&to, &from)?;
            }
            shape(&to)?;
            shape(&from)
        }
        2 => {
            let e = sh_edge(t, r.integer())?;
            let first = r.real();
            sbe.set_range_3d(&e, first, r.real())?;
            shape(&e)
        }
        _ => {
            let e = sh_edge(t, r.integer())?;
            let copy = sbe.copy(&e, r.flag())?;
            shape(&copy)?;
            flag(copy.is_same(&e));
            Ok(())
        }
    }
}

/// `sh_be_pcurves`.
fn pcurves(r: &mut Reader<'_>, t: &TpShapes, op: i32) -> Result<(), Failure> {
    let sbe = ShapeBuildEdge;
    let e = sh_edge(t, r.integer())?;
    match op {
        5 => sbe.remove_pcurve_on_face(&e, &sh_face(t, r.integer())?)?,
        6 => {
            let s = t.surfaces[r.integer() as usize].clone();
            sbe.remove_pcurve_on_located_surface(&e, &s, &t.locations[r.integer() as usize])?;
        }
        7 => sbe.remove_pcurve_on_surface(&e, &t.surfaces[r.integer() as usize])?,
        8 => {
            let c = t.pcurves[r.integer() as usize].clone();
            sbe.replace_pcurve(&e, &c, &sh_face(t, r.integer())?)?;
        }
        9 => {
            let old = sh_face(t, r.integer())?;
            flag(sbe.reassign_pcurve(&e, &old, &sh_face(t, r.integer())?)?);
        }
        11 => sbe.remove_curve_3d(&e)?,
        _ => flag(sbe.build_curve_3d(&e)),
    }
    shape(&e)
}

/// `sh_be_make`.
fn make(r: &mut Reader<'_>, t: &TpShapes, op: i32, e: &mut Edge) -> Result<(), Failure> {
    let sbe = ShapeBuildEdge;
    let index = |r: &mut Reader<'_>| r.integer() as usize;
    match op {
        13 => sbe.make_edge_on_curve(e, &t.curves[index(r)], &t.locations[index(r)]),
        14 => {
            let (c, l) = (&t.curves[index(r)], &t.locations[index(r)]);
            let p1 = r.real();
            sbe.make_edge_on_curve_range(e, c, l, p1, r.real())
        }
        15 => {
            let c = &t.pcurves[index(r)];
            sbe.make_edge_on_face(e, c, &sh_face(t, r.integer())?)
        }
        16 => {
            let (c, f) = (&t.pcurves[index(r)], sh_face(t, r.integer())?);
            let p1 = r.real();
            sbe.make_edge_on_face_range(e, c, &f, p1, r.real())
        }
        17 => {
            let (c, s) = (&t.pcurves[index(r)], &t.surfaces[index(r)]);
            sbe.make_edge_on_surface(e, c, s, &t.locations[index(r)])
        }
        _ => {
            let (c, s, l) = (
                &t.pcurves[index(r)],
                &t.surfaces[index(r)],
                &t.locations[index(r)],
            );
            let p1 = r.real();
            sbe.make_edge_on_surface_range(e, c, s, l, p1, r.real())
        }
    }
}

fn transform(r: &mut Reader<'_>, t: &TpShapes) {
    let c = t.pcurves[r.integer() as usize].clone();
    let tr = trsf(r);
    let (uf, f, l) = (r.real(), r.real(), r.real());
    attempt(|| {
        let (res, f, l) = ShapeBuildEdge.transform_pcurve(&c, tr, uf, f, l)?;
        pcurve(&res, f, l);
        Ok(())
    });
}

/// `sh_edge program op ...`.
pub fn edge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shape program");
    let op = r.integer();
    match op {
        0..=4 => attempt(|| edit(&mut r, &t, op)),
        10 => transform(&mut r, &t),
        5..=12 => attempt(|| pcurves(&mut r, &t, op)),
        _ => {
            let mut e = Edge::default();
            attempt(|| {
                make(&mut r, &t, op, &mut e)?;
                shape(&e)
            });
        }
    }
    end();
}
