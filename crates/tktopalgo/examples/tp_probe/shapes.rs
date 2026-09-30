//! The shape program reader of the TKTopAlgo probe, the Rust twin of
//! tools/oracle/tktopalgo_probe_shapes.cxx (`tp_shapes`, `tp_shape_index`,
//! `tp_same_index`): any shape built with BRep_Builder from numbers. The op codes are
//! documented in the native file.
use super::face::surface;
use super::print::Reader;
use super::shapes_geometry::{curve2d, curve3d, framed_plane, location, polygon, triangulation};
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepCurve3D};
use rustycad_tkbrep::topods::{
    CompSolid, Compound, Edge, Face, Shape, Shell, Solid, Vertex, Wire, edge, face, vertex,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::toploc::Location;

/// `TpShapes`: the tables of one program.
pub struct TpShapes {
    pub surfaces: Vec<Handle<dyn GeomSurface>>,
    pub curves: Vec<Handle<dyn GeomCurve>>,
    pub pcurves: Vec<Handle<dyn Geom2dCurve>>,
    pub locations: Vec<Location>,
    pub triangulations: Vec<Handle<Triangulation>>,
    pub shapes: Vec<Shape>,
}

fn orientation(r: &mut Reader<'_>) -> Orientation {
    Orientation::ALL[r.integer() as usize]
}

impl TpShapes {
    fn shape(&self, index: i32) -> Shape {
        self.shapes[index as usize].clone()
    }
    fn edge(&self, index: i32) -> Result<Edge, Failure> {
        edge(&self.shape(index))
    }
    fn surface(&self, index: i32) -> Handle<dyn GeomSurface> {
        self.surfaces[index as usize].clone()
    }
    fn location(&self, index: i32) -> Location {
        self.locations[index as usize].clone()
    }
    fn pcurve(&self, index: i32) -> Handle<dyn Geom2dCurve> {
        if index < 0 {
            return Handle::default();
        }
        self.pcurves[index as usize].clone()
    }
    fn face(&self, index: i32) -> Result<Face, Failure> {
        face(&self.shape(index))
    }
    fn vertex(&self, index: i32) -> Result<Vertex, Failure> {
        vertex(&self.shape(index))
    }
}

/// `tp_container`: an empty wire, shell, solid, compound or compsolid.
fn container(kind: i32) -> Shape {
    let b = BRepBuilder::default();
    match kind {
        0 => {
            let mut w = Wire::default();
            b.make_wire(&mut w);
            w.into_shape()
        }
        1 => {
            let mut s = Shell::default();
            b.make_shell(&mut s);
            s.into_shape()
        }
        2 => {
            let mut s = Solid::default();
            b.make_solid(&mut s);
            s.into_shape()
        }
        3 => {
            let mut c = Compound::default();
            b.make_compound(&mut c);
            c.into_shape()
        }
        _ => {
            let mut c = CompSolid::default();
            b.make_comp_solid(&mut c);
            c.into_shape()
        }
    }
}

/// `tp_create_op`: the ops creating a geometry or a shape.
fn create_op(r: &mut Reader<'_>, t: &mut TpShapes, op: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    match op {
        1 => t.surfaces.push(surface(r)?),
        2 => t.curves.push(curve3d(r, t)?),
        3 => t.pcurves.push(curve2d(r, t)?),
        4 => t.locations.push(location(r, t)?),
        5 => t.surfaces.push(framed_plane(r)?),
        28 => t.triangulations.push(triangulation(r)?),
        10 => {
            let (x, y, z) = (r.real(), r.real(), r.real());
            let mut v = Vertex::default();
            b.make_vertex_at(&mut v, rustycad_tkmath::gp::Point3::new(x, y, z), r.real())?;
            t.shapes.push(v.into_shape());
        }
        11 => {
            let mut e = Edge::default();
            b.make_edge(&mut e)?;
            t.shapes.push(e.into_shape());
        }
        19 => t.shapes.push(container(r.integer())),
        20 => {
            let (s, l, tol) = (r.integer(), r.integer(), r.real());
            let mut f = Face::default();
            if s < 0 {
                b.make_face(&mut f);
            } else {
                b.make_face_on_located_surface(&mut f, &t.surface(s), &t.location(l), tol)?;
            }
            t.shapes.push(f.into_shape());
        }
        21 => {
            let i = r.integer();
            t.shapes.push(t.shape(i).oriented(orientation(r)));
        }
        22 => {
            let i = r.integer();
            t.shapes
                .push(t.shape(i).located(&t.location(r.integer()), false)?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `tp_edge_op`: the ops changing the representations of an edge.
fn edge_op(r: &mut Reader<'_>, t: &TpShapes, op: i32) -> Result<bool, Failure> {
    if !(12..=17).contains(&op) {
        return Ok(false);
    }
    let b = BRepBuilder::default();
    let e = t.edge(r.integer())?;
    match op {
        12 => {
            let (c, l, tol) = (r.integer(), r.integer(), r.real());
            let curve = if c < 0 {
                Handle::default()
            } else {
                t.curves[c as usize].clone()
            };
            b.update_edge_located_curve(&e, &curve, &t.location(l), tol)?;
        }
        13 => {
            let (c, s, l, tol) = (r.integer(), r.integer(), r.integer(), r.real());
            b.update_edge_pcurve(&e, &t.pcurve(c), &t.surface(s), &t.location(l), tol, None)?;
        }
        14 => {
            let (c1, c2, s, l, tol) =
                (r.integer(), r.integer(), r.integer(), r.integer(), r.real());
            let (p1, p2) = (t.pcurve(c1), t.pcurve(c2));
            b.update_edge_pcurves(&e, (&p1, &p2), &t.surface(s), &t.location(l), tol, None)?;
        }
        15 => {
            let (f, l) = (r.real(), r.real());
            b.range(&e, f, l, r.flag())?;
        }
        16 => {
            let (s, l, f) = (r.integer(), r.integer(), r.real());
            b.range_on_surface(&e, &t.surface(s), &t.location(l), f, r.real())?;
        }
        _ => {
            let (sp, sr) = (r.flag(), r.flag());
            b.same_parameter(&e, sp)?;
            b.same_range(&e, sr)?;
            b.degenerated(&e, r.flag())?;
        }
    }
    Ok(true)
}

/// Ops 18 and 23..27: children, vertex representations, flags and tolerances.
fn vertex_op(r: &mut Reader<'_>, t: &mut TpShapes, op: i32, i: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    match op {
        18 => {
            let (c, o) = (r.integer(), r.integer());
            // o = -1: the child with its own orientation.
            let child = if o < 0 {
                t.shape(c)
            } else {
                t.shape(c).oriented(Orientation::ALL[o as usize])
            };
            b.add(&mut t.shapes[i as usize], &child)?;
        }
        23 => {
            let (p, e) = (r.real(), r.integer());
            b.update_vertex_on_edge(&t.vertex(i)?, p, &t.edge(e)?, r.real())?;
        }
        24 => {
            let (p, e, s, l) = (r.real(), r.integer(), r.integer(), r.integer());
            let (s, l) = (t.surface(s), t.location(l));
            b.update_vertex_on_edge_surface(&t.vertex(i)?, p, &t.edge(e)?, &s, &l, r.real())?;
        }
        25 => {
            let (u, v, f) = (r.real(), r.real(), r.integer());
            b.update_vertex_on_face(&t.vertex(i)?, u, v, &t.face(f)?, r.real())?;
        }
        26 => {
            let (which, value) = (r.integer(), r.flag());
            if which == 0 {
                t.shape(i).set_closed(value)?;
            } else {
                t.shape(i).set_infinite(value)?;
            }
        }
        27 => {
            let tol = r.real();
            let s = t.shape(i);
            match s.shape_type()? {
                ShapeEnum::Vertex => b.update_vertex_tolerance(&vertex(&s)?, tol)?,
                ShapeEnum::Edge => b.update_edge_tolerance(&edge(&s)?, tol)?,
                _ => b.update_face_tolerance(&face(&s)?, tol)?,
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Ops 29, 30 and 33..36: polygons, face-bound pcurves and an extra 3d curve.
fn polygon_op(r: &mut Reader<'_>, t: &TpShapes, op: i32, i: i32) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    let e = t.edge(i)?;
    match op {
        29 | 30 => {
            let (tri, l) = (r.integer(), r.integer());
            let p1 = polygon(r)?;
            let triangulation = t.triangulations[tri as usize].clone();
            if op == 29 {
                b.update_edge_polygon_on_triangulation(&e, &p1, &triangulation, &t.location(l))?;
            } else {
                let p2 = polygon(r)?;
                b.update_edge_polygons_on_triangulation(
                    &e,
                    (&p1, &p2),
                    &triangulation,
                    &t.location(l),
                )?;
            }
        }
        33 => {
            let (c, f) = (r.integer(), r.integer());
            b.update_edge_pcurve_on_face(&e, &t.pcurve(c), &t.face(f)?, r.real())?;
        }
        34 => {
            let (c1, c2, f) = (r.integer(), r.integer(), r.integer());
            let (p1, p2) = (t.pcurve(c1), t.pcurve(c2));
            b.update_edge_pcurves_on_face(&e, (&p1, &p2), &t.face(f)?, r.real())?;
        }
        35 => {
            let (f, first) = (r.integer(), r.real());
            b.range_on_face(&e, &t.face(f)?, first, r.real())?;
        }
        _ => {
            let c = r.integer();
            let rep = BRepCurve3D::new(&t.curves[c as usize], &t.location(r.integer()));
            edge_tshape(&e, "tp_shapes")?.change_curves(|list| list.push(Handle::new(rep).cast()));
        }
    }
    Ok(())
}

/// `tp_shapes`: runs the ops of one program.
pub fn tp_shapes(r: &mut Reader<'_>) -> Result<TpShapes, Failure> {
    let mut t = TpShapes {
        surfaces: Vec::new(),
        curves: Vec::new(),
        pcurves: Vec::new(),
        locations: vec![Location::default()],
        triangulations: Vec::new(),
        shapes: Vec::new(),
    };
    let nops = r.integer();
    for _ in 0..nops {
        let op = r.integer();
        if create_op(r, &mut t, op)? || edge_op(r, &t, op)? {
            continue;
        }
        let i = r.integer();
        if !vertex_op(r, &mut t, op, i)? {
            polygon_op(r, &t, op, i)?;
        }
    }
    Ok(t)
}

/// `tp_shape_index`: the index of `s` (IsEqual) in the table, -1 for a null shape, -2
/// when it is not there.
pub fn tp_shape_index(t: &TpShapes, s: &Shape) -> i32 {
    if s.is_null() {
        return -1;
    }
    t.shapes
        .iter()
        .position(|candidate| candidate.is_equal(s))
        .map_or(-2, |i| i as i32)
}

/// `tp_same_index`: the same with IsSame.
pub fn tp_same_index(t: &TpShapes, s: &Shape) -> i32 {
    if s.is_null() {
        return -1;
    }
    t.shapes
        .iter()
        .position(|candidate| candidate.is_same(s))
        .map_or(-2, |i| i as i32)
}
