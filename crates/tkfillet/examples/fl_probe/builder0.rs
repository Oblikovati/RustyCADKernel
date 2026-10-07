//! The free functions of ChFi3d_Builder_0 without the data structure (`fl_b0_*`,
//! `fl_bound_*`, `fl_enlarge_box`, `fl_inter_plane_edge`, `fl_reparam_pcurv`,
//! `fl_project_pcurv`); the twin of tools/oracle/tkfillet_probe_builder0.cxx.
use super::blend_func::{geom_guide, geom_surface, surface};
use super::chfi3d::{edge_faces, solid};
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, xyz};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::{FilSpine, Map, SpineHandle};
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomConic, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::GeomFillBoundary;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Vector2, Vector3};
use std::sync::{Arc, Mutex};

/// `fl_box`.
fn bx(b: &BoundingBox3) {
    attempt(|| {
        b.get()?.into_iter().for_each(real);
        Ok(())
    });
}

/// `fl_adaptor_bounds`.
fn adaptor_bounds(s: &dyn Adaptor3dSurface) -> Result<(), Failure> {
    real(s.first_u_parameter()?);
    real(s.last_u_parameter()?);
    real(s.first_v_parameter()?);
    real(s.last_v_parameter()?);
    Ok(())
}

/// `fl_curve2d`.
pub(super) fn curve2d(c: &Handle<dyn Geom2dCurve>) {
    flag(c.is_null());
    let Some(curve) = c.get() else {
        return;
    };
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    real(f);
    real(l);
    attempt(|| {
        pnt2d(curve.value(f)?);
        pnt2d(curve.value(0.5 * (f + l))?);
        pnt2d(curve.value(l)?);
        Ok(())
    });
}

/// `fl_bezier2d`.
fn bezier2d(c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    let b = c.cast::<Geom2dBezierCurve>();
    flag(b.is_null());
    if let Some(b) = b.get() {
        for i in 1..=b.nb_poles() {
            pnt2d(b.pole(i)?);
        }
    }
    Ok(())
}

/// `fl_face_index`.
fn face_index(s: &Shape, f: &Face) -> Result<i32, Failure> {
    if f.is_null() {
        return Ok(0);
    }
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, ShapeEnum::Face, &mut m)?;
    Ok(m.find_index(f) as i32)
}

/// `fl_b0_scalars`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn scalars_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (u, f, l, eps) = (r.real(), r.real(), r.real(), r.real());
    let p = [r.pnt2d(), r.pnt2d(), r.pnt2d(), r.pnt2d()];
    real(b0::in_period(u, f, l, eps));
    let (mu, big_u, mv, big_v) = b0::boite(p[0], p[1]);
    [mu, big_u, mv, big_v].into_iter().for_each(real);
    let (du, dv, mu, big_u, mv, big_v) = b0::boite4(p);
    [du, dv, mu, big_u, mv, big_v].into_iter().for_each(real);
    end();
}

/// `fl_bound_srf`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn bound_srf_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mut s = geom_surface(r);
    let trim = r.flag();
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    let bounds = (r.real(), r.real(), r.real(), r.real());
    let check = r.flag();
    attempt(|| {
        if trim {
            s = Handle::new(GeomRectangularTrimmedSurface::new(
                &s,
                (u1, u2),
                (v1, v2),
                (true, true),
            )?)
            .cast();
        }
        let a = GeomAdaptorSurface::with_surface(&s)?;
        b0::bound_srf(&a, bounds, check)?;
        adaptor_bounds(&a)
    });
    end();
}

/// `fl_bound_fac`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn bound_fac_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = solid(r).expect("probe solid");
    let (t, check) = (r.real(), r.flag());
    let mut ex = Explorer::new(&s, ShapeEnum::Face, ShapeEnum::Shape).expect("faces");
    while ex.more() {
        let f = topods::face(ex.current()).expect("face");
        attempt(|| {
            let (u1, u2, v1, v2) = brep_tools::uv_bounds(&f)?;
            let a = BRepAdaptorSurface::with_face(&f, true)?;
            let bounds = (
                u1 + t * (u2 - u1),
                u2 - t * (u2 - u1),
                v1 + t * (v2 - v1),
                v2 - t * (v2 - v1),
            );
            b0::bound_fac(&a, bounds, check)?;
            adaptor_bounds(&a)
        });
        ex.next().expect("next face");
    }
    end();
}

/// `fl_enlarge_box`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn enlarge_box_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let c = geom_guide(r);
    let (wd, wf) = (r.real(), r.real());
    let s = surface(r);
    let (p, d, wd2, wf2) = (r.pnt2d(), r.dir2d(), r.real(), r.real());
    let (mut b1, mut b2, mut b3, mut b4) = Default::default();
    attempt(|| b0::enlarge_box_curve(&c, wd, wf, &mut b1, &mut b2));
    bx(&b1);
    bx(&b2);
    let pc: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p, d)).cast();
    attempt(|| b0::enlarge_box_pcurve(&s, &pc, (wd2, wf2), &mut b3, &mut b4));
    bx(&b3);
    bx(&b4);
    end();
}

/// The fillet spine of one edge with a constant radius.
fn edge_spine(e: &Edge, radius: f64) -> Result<SpineHandle, Failure> {
    let mut fs = FilSpine::new();
    fs.set_edges(e);
    fs.load()?;
    fs.set_radius(radius)?;
    Ok(Arc::new(Mutex::new(fs)))
}

/// One edge of `fl_b0_faces`.
fn edge_faces_case(
    s: &Shape,
    ef: &Map,
    e: &Edge,
    i: i32,
    (t, radius): (f64, f64),
) -> Result<(), Failure> {
    let (mut f1, mut f2) = (Face::default(), Face::default());
    attempt(|| {
        (f1, f2) = b0::conexfaces(e, ef)?;
        int(face_index(s, &f1)?);
        int(face_index(s, &f2)?);
        Ok(())
    });
    if f1.is_null() || f2.is_null() {
        return Ok(());
    }
    attempt(|| b0::evalconti(e, &f1, &f2).map(|c| int(c as i32)));
    attempt(|| {
        let (a, b) = BRepTool::range(e)?;
        let mut b3 = BoundingBox3::default();
        b0::enlarge_box_edge(e, ef.find_from_index(i)?, a + t * (b - a), &mut b3)?;
        bx(&b3);
        Ok(())
    });
    attempt(|| {
        let sp = edge_spine(e, radius)?;
        let (s1, s2) = (
            BRepAdaptorSurface::with_face(&f1, true)?,
            BRepAdaptorSurface::with_face(&f2, true)?,
        );
        flag(b0::kparticular(&sp, 1, &s1, &s2)?);
        Ok(())
    });
    Ok(())
}

/// The vertices of `fl_b0_faces`.
fn vertex_states(verts: &IndexedDataMapOfShapeListOfShape, ef: &Map) -> Result<(), Failure> {
    for i in 1..=verts.extent() {
        let es = verts.find_from_index(i)?;
        int(es.len() as i32);
        if es.len() != 3 {
            continue;
        }
        let edges: Vec<Edge> = es.iter().map(topods::edge).collect::<Result<_, _>>()?;
        let e3 = [edges[0].clone(), edges[1].clone(), edges[2].clone()];
        attempt(|| b0::edge_state(&e3, ef).map(|st| int(st as i32)));
    }
    Ok(())
}

/// `fl_b0_faces`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn faces_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = solid(r).expect("probe solid");
    let (t, radius) = (r.real(), r.real());
    let mut ef = Map::new();
    ef.fill(&s, ShapeEnum::Edge, ShapeEnum::Face)
        .expect("edge map");
    let mut verts = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&s, ShapeEnum::Vertex, ShapeEnum::Edge, &mut verts)
        .expect("vertex map");
    let edges = edge_faces(&s).expect("edges");
    int(edges.extent() as i32);
    for i in 1..=edges.extent() {
        let e = topods::edge(edges.find_key(i).expect("edge")).expect("edge");
        edge_faces_case(&s, &ef, &e, i as i32, (t, radius)).expect("edge case");
    }
    vertex_states(&verts, &ef).expect("vertices");
    end();
}

/// `fl_b0_spines`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn spines_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (pd, mut vd) = (r.pnt(), Vector3::from(r.xyz()));
    let (pf, mut vf) = (r.pnt(), Vector3::from(r.xyz()));
    let rad = r.real();
    attempt(|| {
        let c = b0::circular_spine(pd, vd, pf, vf, rad)?;
        flag(c.is_none());
        if let Some((c, w1, w2)) = c {
            let circle = c.get().ok_or_else(|| Failure::new("null circle"))?;
            pnt(circle.location());
            xyz(circle.axis().direction().xyz());
            xyz(circle.x_axis().direction().xyz());
            real(circle.radius());
            real(w1);
            real(w2);
        }
        Ok(())
    });
    attempt(|| {
        let b = b0::spine(pd, &mut vd, pf, &mut vf, rad)?;
        let b = b.get().ok_or_else(|| Failure::new("null spine"))?;
        for i in 1..=b.nb_poles() {
            pnt(b.pole(i)?);
        }
        xyz(*vd.xyz());
        xyz(*vf.xyz());
        Ok(())
    });
    end();
}

/// `fl_inter_plane_edge`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn inter_plane_edge_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = surface(r);
    let c = geom_guide(r);
    let (f, l, sens, tolc) = (r.real(), r.real(), r.flag(), r.real());
    attempt(|| {
        let hc: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_range(&c, f, l)?).cast();
        let w = b0::inter_plane_edge(&s, &hc, sens, tolc)?;
        flag(w.is_some());
        real(w.unwrap_or(-7.0));
        Ok(())
    });
    end();
}

/// `fl_b0_pcurves`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn pcurves_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (uv1, uv2, pd, pf, rev) = (r.pnt2d(), r.pnt2d(), r.real(), r.real(), r.flag());
    let (c, s, tol) = (geom_guide(r), geom_surface(r), r.real());
    let mut pc: Handle<dyn Geom2dCurve> = Handle::default();
    attempt(|| {
        pc = b0::compute_pcurv(uv1, uv2, (pd, pf), rev)?;
        curve2d(&pc);
        Ok(())
    });
    attempt(|| {
        let hc: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_range(&c, pd, pf)?).cast();
        let hs: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast();
        let (ok, reached) = b0::check_same_parameter(&hc, &pc, &hs, tol)?;
        flag(ok);
        real(reached);
        Ok(())
    });
    attempt(|| {
        let mut reached = -1.0;
        let pc2 = b0::compute_pcurv_geom(&c, (uv1, uv2), &s, (pd, pf), tol, &mut reached, rev)?;
        real(reached);
        curve2d(&pc2);
        Ok(())
    });
    attempt(|| {
        let (mut pc3, mut reached) = (pc.clone(), -1.0);
        flag(b0::same_parameter_geom(
            &c,
            &mut pc3,
            &s,
            (pd, pf),
            tol,
            &mut reached,
        )?);
        real(reached);
        curve2d(&pc3);
        Ok(())
    });
    end();
}

/// `fl_reparam_pcurv`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn reparam_pcurv_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mut poles = [r.pnt2d(), r.pnt2d(), r.pnt2d(), r.pnt2d()];
    let mut knots = [r.real(), r.real()];
    let (trim, t1, t2, uf, ul) = (r.flag(), r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        let mut mults = [4, 4];
        let (p, k, m) = (
            Array1::from_slice(&mut poles),
            Array1::from_slice(&mut knots),
            Array1::from_slice(&mut mults),
        );
        let mut c: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dBSplineCurve::new(&p, &k, &m, 3, false)?).cast();
        if trim {
            c = Handle::new(Geom2dTrimmedCurve::new(&c, t1, t2, true, true)?).cast();
        }
        b0::reparam_pcurv(uf, ul, &mut c)?;
        curve2d(&c);
        Ok(())
    });
    end();
}

/// `fl_project_pcurv`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn project_pcurv_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let c = geom_guide(r);
    let (f, l) = (r.real(), r.real());
    let (s, tol) = (surface(r), r.real());
    attempt(|| {
        let hc: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_range(&c, f, l)?).cast();
        let (mut pc, mut reached): (Handle<dyn Geom2dCurve>, f64) = (Handle::default(), -1.0);
        b0::project_pcurv(&hc, &s, &mut pc, tol, &mut reached)?;
        real(reached);
        curve2d(&pc);
        Ok(())
    });
    end();
}

/// `fl_boundary`.
fn boundary(b: &Handle<dyn GeomFillBoundary>) -> Result<(), Failure> {
    let (f, l) = b.bounds()?;
    real(f);
    real(l);
    flag(b.has_normals());
    flag(b.is_degenerated());
    for u in [f, 0.5 * (f + l), l] {
        let (p, v) = b.d1(u)?;
        pnt(p);
        xyz(*v.xyz());
        if b.has_normals() {
            xyz(*b.norm(u)?.xyz());
        }
    }
    Ok(())
}

/// The boundary and pcurve outputs of an mkbound overload.
fn bound_and_curve(
    result: (Handle<dyn GeomFillBoundary>, Handle<dyn Geom2dCurve>),
) -> Result<(), Failure> {
    boundary(&result.0)?;
    bezier2d(&result.1)
}

/// `fl_b0_bounds`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn bounds_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = surface(r);
    let (p1, d1, p2, d2, red) = (r.pnt2d(), r.dir2d(), r.pnt2d(), r.dir2d(), r.flag());
    let (v1, v2) = (Vector3::from(r.xyz()), Vector3::from(r.xyz()));
    let (s1, s2, t3d, ta, free) = (r.integer(), r.integer(), r.real(), r.real(), r.flag());
    attempt(|| {
        let (mut e1, mut e2) = (d1, d2);
        bezier2d(&b0::build_pcurve(p1, &mut e1, p2, &mut e2, red)?.cast())?;
        [e1.x(), e1.y(), e2.x(), e2.y()].into_iter().for_each(real);
        Ok(())
    });
    attempt(|| {
        bezier2d(&b0::build_pcurve_on(
            &s,
            p1,
            Vector2::from(d1),
            p2,
            Vector2::from(d2),
            red,
        )?)
    });
    attempt(|| bezier2d(&b0::build_pcurve_3d(&s, p1, v1, p2, v2, red)?));
    attempt(|| boundary(&b0::mkbound_points(&s, p1, p2, (t3d, ta), free)?));
    attempt(|| bound_and_curve(b0::mkbound_segment(&s, p1, p2, (t3d, ta), free)?));
    attempt(|| {
        let gs: Handle<dyn GeomSurface> = s
            .cast::<GeomAdaptorSurface>()
            .get()
            .ok_or_else(|| Failure::new("surface"))?
            .surface();
        boundary(&b0::mkbound_geom(&gs, p1, p2, (t3d, ta), free)?)
    });
    attempt(|| {
        bound_and_curve(b0::mkbound_tangents_2d(
            &s,
            (s1, p1, Vector2::from(d1)),
            (s2, p2, Vector2::from(d2)),
            (t3d, ta),
        )?)
    });
    attempt(|| {
        let (mut w1, mut w2) = (v1, v2);
        bound_and_curve(b0::mkbound_tangents_3d(
            &s,
            (s1, p1, &mut w1),
            (s2, p2, &mut w2),
            (t3d, ta),
        )?)?;
        xyz(*w1.xyz());
        xyz(*w2.xyz());
        Ok(())
    });
    end();
}
