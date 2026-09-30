//! The shape reader of the TKTopAlgo probe, the Rust twin of
//! tools/oracle/tktopalgo_probe_shape.cxx (`tp_shape`): vertices, edges, faces, shells
//! and a solid (or a compound) built with BRep_Builder from numbers. The record layout
//! is documented in the native file.
use super::face::{orientation, point};
use super::print::Reader;
use super::shape_geometry::{curve, location, mesh, mesh_polygon, pcurve, surface};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::poly::Polygon3D;
use rustycad_tkmath::toploc::Location;

/// `TpShape`: the shape and its sub-shapes in record order.
pub struct TpShape {
    pub shape: Shape,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
    pub shells: Vec<Shell>,
}

/// The range and record tolerance of each edge, set and used after the faces.
struct EdgeRecord {
    range: (f64, f64),
    tol: f64,
}

/// `np (x y z)... defl`: the 3d polygon of an edge, `None` for np = 0.
fn polygon3d(r: &mut Reader<'_>) -> Result<Option<Handle<Polygon3D>>, Failure> {
    let np = r.integer();
    if np <= 0 {
        return Ok(None);
    }
    let mut nodes = Array1::with_bounds(1, np).map_err(super::face::array_failure)?;
    for i in 1..=np {
        nodes[i] = point(r);
    }
    let polygon = Polygon3D::from_nodes(&nodes);
    polygon.set_deflection(r.real());
    Ok(Some(Handle::new(polygon)))
}

/// `tp_shape_edge`: one edge, with its range and tolerance kept in `records`.
fn edge(
    r: &mut Reader<'_>,
    vertices: &[Vertex],
    records: &mut Vec<EdgeRecord>,
) -> Result<Edge, Failure> {
    let b = BRepBuilder::default();
    let c = curve(r)?;
    let (vf, vl, degenerated, tol) = (r.integer(), r.integer(), r.flag(), r.real());
    let range = (r.real(), r.real());
    records.push(EdgeRecord { range, tol });
    let polygon = polygon3d(r)?;
    let mut e = Edge::default();
    match (&polygon, c.is_null()) {
        (_, false) => {
            b.make_edge_with_curve(&mut e, &c, tol)?;
            if let Some(polygon) = &polygon {
                b.update_edge_polygon3d(&e, polygon, &Location::default())?;
            }
        }
        (Some(polygon), true) => b.make_edge_with_polygon3d(&mut e, polygon)?,
        (None, true) => b.make_edge(&mut e)?,
    }
    b.update_edge_tolerance(&e, tol)?;
    if vf >= 0 {
        b.add(
            &mut e,
            &vertices[vf as usize].oriented(Orientation::Forward),
        )?;
    }
    if vl >= 0 {
        b.add(
            &mut e,
            &vertices[vl as usize].oriented(Orientation::Reversed),
        )?;
    }
    b.degenerated(&e, degenerated)?;
    Ok(e)
}

/// `TpUse`: the pcurves of the FORWARD and REVERSED uses of an edge in a face.
struct PcurveUse {
    edge: usize,
    forward: Handle<dyn Geom2dCurve>,
    reversed: Handle<dyn Geom2dCurve>,
}

/// `tp_shape_pcurves`: one or two pcurves per edge of the face, in order of first use.
fn update_pcurves(
    face: &Face,
    edges: &[Edge],
    uses: &[PcurveUse],
    records: &[EdgeRecord],
) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    for u in uses {
        let (e, tol) = (&edges[u.edge], records[u.edge].tol);
        match (u.forward.is_null(), u.reversed.is_null()) {
            (false, false) => {
                b.update_edge_pcurves_on_face(e, (&u.forward, &u.reversed), face, tol)?
            }
            (false, true) => b.update_edge_pcurve_on_face(e, &u.forward, face, tol)?,
            (true, false) => b.update_edge_pcurve_on_face(e, &u.reversed, face, tol)?,
            (true, true) => {}
        }
    }
    Ok(())
}

/// `tp_shape_wire`: the edges of a wire, their pcurves collected in `uses`.
fn wire(r: &mut Reader<'_>, edges: &[Edge], uses: &mut Vec<PcurveUse>) -> Result<Wire, Failure> {
    let b = BRepBuilder::default();
    let mut w = Wire::default();
    b.make_wire(&mut w);
    let o = orientation(r);
    for _ in 0..r.integer() {
        let k = r.integer() as usize;
        let eo = orientation(r);
        let c = pcurve(r)?;
        b.add(&mut w, &edges[k].oriented(eo))?;
        let index = match uses.iter().rposition(|u| u.edge == k) {
            Some(index) => index,
            None => {
                uses.push(PcurveUse {
                    edge: k,
                    forward: Handle::default(),
                    reversed: Handle::default(),
                });
                uses.len() - 1
            }
        };
        if eo == Orientation::Reversed {
            uses[index].reversed = c;
        } else {
            uses[index].forward = c;
        }
    }
    w.set_orientation(o);
    Ok(w)
}

/// The mesh of a face (after nt > 0) and the polygons of some edges on it.
fn face_mesh(r: &mut Reader<'_>, face: &Face, nt: i32, edges: &[Edge]) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    let triangulation = mesh(r, nt)?;
    for _ in 0..r.integer() {
        let k = r.integer() as usize;
        let polygon = mesh_polygon(r)?;
        b.update_edge_polygon_on_triangulation(
            &edges[k],
            &polygon,
            &triangulation,
            &Location::default(),
        )?;
    }
    b.update_face_triangulation(face, &triangulation, true)
}

/// `tp_shape_face`: one face with its wires, pcurves and mesh.
fn face(r: &mut Reader<'_>, edges: &[Edge], records: &[EdgeRecord]) -> Result<Face, Failure> {
    let b = BRepBuilder::default();
    let kind = r.integer();
    let s = if kind >= 0 {
        surface(r, kind)?
    } else {
        Handle::default()
    };
    let (tol, natural) = (r.real(), r.flag());
    let mut f = Face::default();
    if s.is_null() {
        b.make_face(&mut f);
        b.update_face_tolerance(&f, tol)?;
    } else {
        b.make_face_on_surface(&mut f, &s, tol)?;
    }
    b.natural_restriction(&f, natural)?;
    let mut uses = Vec::new();
    for _ in 0..r.integer() {
        let w = wire(r, edges, &mut uses)?;
        b.add(&mut f, &w)?;
    }
    update_pcurves(&f, edges, &uses, records)?;
    let nt = r.integer();
    if nt > 0 {
        face_mesh(r, &f, nt, edges)?;
    }
    f.set_orientation(orientation(r));
    Ok(f)
}

/// `tp_shape_shell`: faces (oriented and located) of a shell, its closed flag.
fn shell(r: &mut Reader<'_>, faces: &[Face]) -> Result<Shell, Failure> {
    let b = BRepBuilder::default();
    let mut s = Shell::default();
    b.make_shell(&mut s);
    let (o, closed) = (orientation(r), r.flag());
    for _ in 0..r.integer() {
        let k = r.integer() as usize;
        let fo = orientation(r);
        let l = location(r)?;
        b.add(&mut s, &faces[k].oriented(fo).located(&l, false)?)?;
    }
    s.set_closed(closed)?;
    s.set_orientation(o);
    Ok(s)
}

/// `tp_shape_item`: the solid of all the shells, a shell, a face, an edge or a vertex.
fn item(s: &TpShape, kind: i32, index: usize) -> Result<Shape, Failure> {
    Ok(match kind {
        0 => {
            let b = BRepBuilder::default();
            let mut solid = Solid::default();
            b.make_solid(&mut solid);
            for x in &s.shells {
                b.add(&mut solid, x)?;
            }
            solid.into_shape()
        }
        1 => s.shells[index].clone().into_shape(),
        2 => s.faces[index].clone().into_shape(),
        3 => s.edges[index].clone().into_shape(),
        _ => s.vertices[index].clone().into_shape(),
    })
}

/// `tp_shape`: the shape of one record.
pub fn tp_shape(r: &mut Reader<'_>) -> Result<TpShape, Failure> {
    let b = BRepBuilder::default();
    let mut s = TpShape {
        shape: Shape::default(),
        vertices: Vec::new(),
        edges: Vec::new(),
        faces: Vec::new(),
        shells: Vec::new(),
    };
    for _ in 0..r.integer() {
        let p = point(r);
        let mut v = Vertex::default();
        b.make_vertex_at(&mut v, p, r.real())?;
        s.vertices.push(v);
    }
    let mut records = Vec::new();
    for _ in 0..r.integer() {
        s.edges.push(edge(r, &s.vertices, &mut records)?);
    }
    for _ in 0..r.integer() {
        s.faces.push(face(r, &s.edges, &records)?);
    }
    for (e, record) in s.edges.iter().zip(&records) {
        b.range(e, record.range.0, record.range.1, false)?;
    }
    for _ in 0..r.integer() {
        s.shells.push(shell(r, &s.faces)?);
    }
    let mut items = Vec::new();
    for _ in 0..r.integer() {
        let kind = r.integer();
        items.push(item(&s, kind, r.integer() as usize)?);
    }
    s.shape = if items.len() == 1 {
        items.remove(0)
    } else {
        compound(&items)?
    };
    s.shape.set_location(&location(r)?, false)?;
    Ok(s)
}

/// The compound of several items.
fn compound(items: &[Shape]) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut c = Compound::default();
    b.make_compound(&mut c);
    for x in items {
        b.add(&mut c, x)?;
    }
    Ok(c.into_shape())
}
