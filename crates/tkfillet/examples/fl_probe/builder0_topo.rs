//! The ChFi3d_Builder_0 topology searches and the smoothness of guides
//! (`fl_b0_topology`, `fl_b0_smooth`); the twin of the last part of
//! tools/oracle/tkfillet_probe_builder0.cxx.
use super::blend_func::geom_guide;
use super::chfi3d::{edge_faces, solid};
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::Map;
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;

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
