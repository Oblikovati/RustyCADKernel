//! The ChFi3d_Builder_0 topology searches, the smoothness of guides and the
//! intersection curves (`fl_b0_topology`, `fl_b0_smooth`, `fl_compute_curves`); the twin of the last part of
//! tools/oracle/tkfillet_probe_builder0.cxx.
use super::blend_func::{geom_guide, surface};
use super::builder0::curve2d;
use super::chfi_ds::{load, segment};
use super::chfi3d::{edge_faces, solid};
use super::common::{Reader, attempt, end, flag, int, pnt, real, xyz};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::{ElSpine, Map, Spine, SpineHandle, lock};
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, GeomAdaptorTransformedSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::gp::Transform3;
use std::sync::{Arc, Mutex};

/// `fl_index`.
fn index(s: &Shape, x: &Shape, kind: ShapeEnum) -> Result<i32, Failure> {
    if x.is_null() {
        return Ok(0);
    }
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, kind, &mut m)?;
    Ok(m.find_index(x) as i32)
}

fn map_of(s: &Shape, sub: ShapeEnum, ancestor: ShapeEnum) -> Map {
    let mut m = Map::new();
    m.fill(s, sub, ancestor).expect("map");
    m
}

/// The faces part of `fl_b0_topology`.
fn faces_part(s: &Shape) -> Result<(), Failure> {
    let mut ex = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?;
    while ex.more() {
        let f = topods::face(ex.current())?;
        attempt(|| seam(s, b0::couture(&f)?));
        let mut ev = Explorer::new(&f, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while ev.more() {
            let v = topods::vertex(ev.current())?;
            attempt(|| seam(s, b0::couture_on_vertex(&f, &v)?));
            ev.next()?;
        }
        let mut ee = Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ee.more() {
            let e = topods::edge(ee.current())?;
            attempt(|| b0::is_pseudo_seam(&e, &f).map(flag));
            attempt(|| {
                let (mut x, mut v) = (Edge::default(), Vertex::default());
                b0::cherche_element(&topexp::first_vertex(&e, false), &e, &f, &mut x, &mut v)?;
                int(index(s, &x, ShapeEnum::Edge)?);
                int(index(s, &v, ShapeEnum::Vertex)?);
                Ok(())
            });
            ee.next()?;
        }
        ex.next()?;
    }
    Ok(())
}

/// The seam flag and edge index of `ChFi3d_Couture`.
fn seam(s: &Shape, e: Option<Edge>) -> Result<(), Failure> {
    flag(e.is_some());
    int(index(s, &e.unwrap_or_default(), ShapeEnum::Edge)?);
    Ok(())
}

/// The edges part of `fl_b0_topology`.
fn edges_part(s: &Shape) -> Result<(), Failure> {
    let edges = edge_faces(s)?;
    for i in 1..=edges.extent() {
        attempt(|| {
            let list = edges.find_from_index(i)?;
            let (f1, f2) = b0::edge_common_faces(list)?;
            int(index(s, &f1, ShapeEnum::Face)?);
            int(index(s, &f2, ShapeEnum::Face)?);
            let mut f = Face::default();
            b0::cherche_face1(list, &f1, &mut f)?;
            int(index(s, &f, ShapeEnum::Face)?);
            Ok(())
        });
    }
    Ok(())
}

/// The counts of one vertex of `fl_b0_topology`.
fn vertex_counts(s: &Shape, v: &Vertex, (ve, vf, ef): (&Map, &Map, &Map)) {
    attempt(|| vf.find_from_key(v).map(|faces| int(b0::nbface(faces))));
    attempt(|| b0::nb_not_degenerated_edges(v, ve).map(int));
    attempt(|| b0::number_of_edges(v, ve).map(int));
    attempt(|| b0::number_of_sharp_edges(v, ve, ef).map(int));
    attempt(|| {
        let (b, e1, e2) = b0::cherche_bords_libres(ve, v)?;
        flag(b);
        int(index(s, &e1, ShapeEnum::Edge)?);
        int(index(s, &e2, ShapeEnum::Edge)?);
        Ok(())
    });
}

/// The two-edge searches of one vertex of `fl_b0_topology`.
fn vertex_pair(s: &Shape, v: &Vertex, (a, b): (&Edge, &Edge), ef: &Map) {
    attempt(|| b0::angle_edge(v, a, b).map(real));
    attempt(|| {
        let x = b0::cherche_vertex(a, b)?;
        flag(x.is_some());
        int(index(s, &x.unwrap_or_default(), ShapeEnum::Vertex)?);
        Ok(())
    });
    attempt(|| {
        let first = ef.find_from_key(a)?.iter().next().cloned().ok_or_else(|| {
            Failure::with_message("Standard_NoSuchObject", "NCollection_List::First")
        })?;
        let f = topods::face(&first)?;
        let (mut x, mut vx) = (Edge::default(), Vertex::default());
        b0::cherche_edge(
            v,
            &[a.clone().into_shape(), b.clone().into_shape()],
            &f,
            &mut x,
            &mut vx,
        )?;
        int(index(s, &x, ShapeEnum::Edge)?);
        int(index(s, &vx, ShapeEnum::Vertex)?);
        Ok(())
    });
}

/// `fl_b0_topology`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn topology_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = solid(r).expect("probe solid");
    let (ef, ve, vf) = (
        map_of(&s, ShapeEnum::Edge, ShapeEnum::Face),
        map_of(&s, ShapeEnum::Vertex, ShapeEnum::Edge),
        map_of(&s, ShapeEnum::Vertex, ShapeEnum::Face),
    );
    faces_part(&s).expect("faces");
    edges_part(&s).expect("edges");
    let mut verts = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&s, ShapeEnum::Vertex, ShapeEnum::Edge, &mut verts)
        .expect("vertices");
    for i in 1..=verts.extent() {
        let v = topods::vertex(verts.find_key(i).expect("vertex")).expect("vertex");
        vertex_counts(&s, &v, (&ve, &vf, &ef));
        let es: Vec<Edge> = verts
            .find_from_index(i)
            .expect("edges")
            .iter()
            .map(|x| topods::edge(x).expect("edge"))
            .collect();
        if es.len() >= 2 {
            vertex_pair(&s, &v, (&es[0], &es[1]), &ef);
        }
    }
    end();
}

/// `fl_b0_smooth`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn smooth_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let kind = r.integer();
    let guide = geom_guide(r);
    let n = r.integer();
    let mut poles: Vec<_> = (0..n).map(|_| r.pnt()).collect();
    let (f, l) = (r.real(), r.real());
    attempt(|| {
        let mut c: Handle<dyn GeomCurve> = if kind == 1 {
            Handle::new(GeomBezierCurve::new(&Array1::from_slice(&mut poles))?).cast()
        } else {
            guide.clone()
        };
        if f < l {
            c = Handle::new(GeomTrimmedCurve::new(&c, f, l, true, true)?).cast();
        }
        attempt(|| b0::is_smooth(&c).map(flag));
        attempt(|| {
            let bs = b0::approx_by_c2(&c)?;
            flag(bs.is_null());
            if let Some(bs) = bs.get() {
                int(bs.degree());
                int(bs.nb_poles());
                let (bf, bl) = (bs.first_parameter(), bs.last_parameter());
                real(bf);
                real(bl);
                pnt(bs.value(0.5 * (bf + bl))?);
            }
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `fl_curve3d`.
pub(super) fn curve3d(c: &Handle<dyn GeomCurve>) {
    flag(c.is_null());
    let Some(curve) = c.get() else {
        return;
    };
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    real(f);
    real(l);
    attempt(|| {
        pnt(curve.value(f)?);
        pnt(curve.value(0.5 * (f + l))?);
        pnt(curve.value(l)?);
        Ok(())
    });
}

/// The surface of a GeomAdaptor_Surface handle adapted by
/// GeomAdaptor_TransformedSurface with the identity.
fn transformed(s: &Handle<dyn Adaptor3dSurface>) -> Handle<dyn Adaptor3dSurface> {
    let geom = s
        .cast::<GeomAdaptorSurface>()
        .get()
        .expect("a GeomAdaptor_Surface")
        .surface();
    Handle::new(
        GeomAdaptorTransformedSurface::with_surface(&geom, Transform3::default()).expect("adaptor"),
    )
    .cast()
}

/// `fl_compute_curves`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn compute_curves_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (s1, s2) = (surface(r), surface(r));
    let deb = [r.real(), r.real(), r.real(), r.real()];
    let fin = [r.real(), r.real(), r.real(), r.real()];
    let (tol3d, tol2d, whole, wrap) = (r.real(), r.real(), r.flag(), r.flag());
    let (s1, s2) = if wrap {
        (transformed(&s1), transformed(&s2))
    } else {
        (s1, s2)
    };
    let mut out = b0::IntersectionCurves {
        tolreached: -1.0,
        ..Default::default()
    };
    attempt(|| {
        b0::compute_curves((&s1, &s2), (deb, fin), (tol3d, tol2d), whole, &mut out).map(flag)
    });
    real(out.tolreached);
    curve3d(&out.c3d);
    curve2d(&out.pc1);
    curve2d(&out.pc2);
    end();
}

/// The elementary spine after `ChFi3d_PerformElSpine`.
fn el_spine_state(es: &ElSpine, ns: i32) -> Result<(), Failure> {
    real(es.first_parameter()?);
    real(es.last_parameter()?);
    int(es.nb_vertices());
    for i in 1..=es.nb_vertices() {
        let axis = es.vertex_with_tangent(i)?;
        pnt(axis.location());
        xyz(axis.direction().xyz());
    }
    let kind = es.get_type()?;
    int(kind as i32);
    if kind == CurveType::BSplineCurve {
        let b = es.bspline()?;
        let b = b.get().ok_or_else(|| Failure::new("null elspine curve"))?;
        int(b.degree());
        int(b.nb_knots());
        int(b.nb_poles());
        flag(b.is_periodic());
    }
    let (f, l) = (es.first_parameter()?, es.last_parameter()?);
    for i in 0..=ns {
        pnt(es.value(f + (l - f) * f64::from(i) / f64::from(ns))?);
    }
    Ok(())
}

/// `fl_el_spine`: see tools/oracle/tkfillet_probe_builder0.cxx.
pub fn el_spine_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let n = r.integer();
    let pts: Vec<Point3> = (0..n).map(|_| r.pnt()).collect();
    let (rev, closed, wf, wl) = (r.integer(), r.flag(), r.real(), r.real());
    let (per, c3, tol, ns) = (r.flag(), r.flag(), r.real(), r.integer());
    let mut spine = Spine::new();
    for (i, w) in pts.windows(2).enumerate() {
        spine.set_edges(&segment(w[0], w[1], (rev >> i) & 1 == 1).expect("segment"));
    }
    load(&mut spine, closed);
    let spine: SpineHandle = Arc::new(Mutex::new(spine));
    let es = Handle::new(ElSpine::new());
    let e = es.get().expect("elementary spine");
    e.set_first_parameter(wf);
    e.set_last_parameter(wl);
    e.set_periodic(per);
    attempt(|| {
        let mut guard = lock(&spine);
        let (p, v) = guard.spine_mut().d1(wf)?;
        e.set_first_point_and_tgt(p, v);
        let (p, v) = guard.spine_mut().d1(wl)?;
        e.set_last_point_and_tgt(p, v);
        Ok(())
    });
    attempt(|| {
        let continuity = if c3 { Continuity::C3 } else { Continuity::C2 };
        b0::perform_el_spine(&es, &spine, continuity, tol, false)?;
        el_spine_state(e, ns)
    });
    end();
}
