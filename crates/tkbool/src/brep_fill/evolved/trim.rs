// Rust translation of the static TrimFace, AddDegeneratedEdge and TrimEdge of
// ModelingAlgorithms/TKBool/BRepFill/BRepFill_Evolved.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming of the tubes and revolutions: their parallel edges cut between the
//! vertices of the bisectors, their faces bounded by the wires of the trimming edges.
use super::statics::{double_or_not_in_face, edge_vertices};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{self, WireExplorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge, BRepLibMakeWire, BRepLibWireError};

/// The vertices of `ver` (with their parameters) sorted by parameter.
fn sort_by_parameter(ver: &mut [Shape], par: &mut [f64]) {
    let mut change = true;
    while change {
        change = false;
        for i in 0..par.len().saturating_sub(1) {
            if par[i] > par[i + 1] {
                par.swap(i, i + 1);
                ver.swap(i, i + 1);
                change = true;
            }
        }
    }
}

/// The vertices not in the control edges, or in two, removed; then of two equal
/// neighbours one kept (both removed when in two control edges).
fn filter_vertices(
    control: &[Shape],
    ver: &mut Vec<Shape>,
    par: &mut Vec<f64>,
) -> Result<(), Failure> {
    let mut k = 0;
    while k < ver.len() {
        if double_or_not_in_face(control, &topods::vertex(&ver[k])?)? {
            ver.remove(k);
            par.remove(k);
        } else {
            k += 1;
        }
    }
    let mut k: isize = 0;
    while (k as usize) + 1 < ver.len() {
        let ku = k as usize;
        if ver[ku].is_same(&ver[ku + 1]) {
            ver.remove(ku + 1);
            par.remove(ku + 1);
            if double_or_not_in_face(control, &topods::vertex(&ver[ku])?)? {
                ver.remove(ku);
                par.remove(ku);
            }
            k -= 1;
        }
        k += 1;
    }
    Ok(())
}

/// `TrimEdge(Edge, TheEdgesControle, TheVer, ThePar, S)` (static): the vertices sorted
/// by parameter (and filtered by the control edges for a non degenerated edge), then
/// a copy of the edge between each odd vertex and the next.
pub fn trim_edge(
    edge: &Edge,
    the_edges_controle: &[Shape],
    the_ver: &mut Vec<Shape>,
    the_par: &mut Vec<f64>,
) -> Result<Vec<Shape>, Failure> {
    sort_by_parameter(the_ver, the_par);
    if !BRepTool::degenerated(edge)? {
        filter_vertices(the_edges_controle, the_ver, the_par)?;
    }
    let b = BRepBuilder::default();
    let mut s = Vec::new();
    let mut k = 0;
    while k + 1 < the_ver.len() {
        let mut new_edge = edge.as_ref().empty_copied()?;
        let (o1, o2) = if new_edge.orientation() == Orientation::Reversed {
            (Orientation::Reversed, Orientation::Forward)
        } else {
            (Orientation::Forward, Orientation::Reversed)
        };
        b.add(&mut new_edge, &the_ver[k].oriented(o1))?;
        b.add(&mut new_edge, &the_ver[k + 1].oriented(o2))?;
        b.range(&topods::edge(&new_edge)?, the_par[k], the_par[k + 1], false)?;
        // NIZHNY-EAP 1999-12-22
        brep_lib::update_tolerances(&new_edge, false)?;
        s.push(new_edge);
        k += 2;
    }
    Ok(s)
}

/// The UV ends of an edge on the face in the edge orientation.
fn oriented_uv_points(ce: &Edge, f: &Face) -> Result<(Point2, Point2), Failure> {
    let (pa, pb) = BRepTool::uv_points_on_face(ce, f)?;
    Ok(if ce.as_ref().orientation() == Orientation::Reversed {
        (pb, pa)
    } else {
        (pa, pb)
    })
}

/// A degenerated edge of the surface of `f` from `p` to `q` at the vertex `v`, added
/// to the wire.
fn add_degenerated(
    f: &Face,
    w: &mut Shape,
    v: &Vertex,
    p: Point2,
    q: Point2,
) -> Result<(), Failure> {
    let s = BRepTool::surface(f)?;
    let c2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        p,
        Direction2::from_vector(Vector2::between(p, q))?,
    ))
    .cast();
    let (first, last) = (0., p.distance(q));
    let ne = BRepLibMakeEdge::from_pcurve(&c2d, &s)?.edge()?;
    let b = BRepBuilder::default();
    b.degenerated(&ne, true)?;
    let mut ne = ne.into_shape();
    b.add(&mut ne, &v.as_ref().oriented(Orientation::Forward))?;
    b.add(&mut ne, &v.as_ref().oriented(Orientation::Reversed))?;
    b.range(&topods::edge(&ne)?, first, last, false)?;
    b.add(w, &ne)
}

/// One pass of AddDegeneratedEdge: the first gap in UV between consecutive edges (or
/// the closing gap of a closed wire) filled; whether one was filled.
fn fill_one_gap(f: &Face, w: &mut Shape, tol_conf: f64) -> Result<bool, Failure> {
    let mut we = WireExplorer::with_wire_on_face(&topods::wire(w)?, f)?;
    let (mut vf, mut pf, mut prev_p) = (Vertex::default(), Point2::default(), Point2::default());
    let (mut v2, mut p2) = (Vertex::default(), Point2::default());
    while we.more() {
        let ce = we.current().clone();
        let (v1, v2c) = edge_vertices(&ce);
        let (p1, p2c) = oriented_uv_points(&ce, f)?;
        (v2, p2) = (v2c, p2c);
        if vf.as_ref().is_null() {
            (vf, pf) = (v1, p1);
        } else if !p1.is_equal(prev_p, tol_conf) {
            add_degenerated(f, w, &v1, prev_p, p1)?;
            return Ok(true);
        }
        prev_p = p2;
        we.next()?;
    }
    if vf.as_ref().is_same(v2.as_ref()) && !pf.is_equal(p2, tol_conf) {
        add_degenerated(f, w, &vf, p2, pf)?;
        return Ok(true);
    }
    Ok(false)
}

/// `AddDegeneratedEdge(F, W)` (static): the missing degenerated edges of the wire on a
/// non planar face (vertices at nodes of the map), found by comparing the UV points
/// of consecutive vertices within 1e-4.
fn add_degenerated_edge(f: &Face, w: &mut Shape) -> Result<(), Failure> {
    let s = BRepTool::surface(f)?;
    if s.is_instance_name("Geom_Plane") {
        return Ok(());
    }
    if s.is_instance_name("Geom_RectangularTrimmedSurface") {
        let rts: Handle<rustycad_tkg3d::geom::GeomRectangularTrimmedSurface> = s.cast();
        if let Some(r) = rts.get()
            && r.basis_surface().is_instance_name("Geom_Plane")
        {
            return Ok(());
        }
    }
    while fill_one_gap(f, w, 1.0e-4)? {}
    Ok(())
}

/// One wire of TrimFace: the first edge then every edge that connects (degenerated
/// edges dropped), restarting after each success.
fn connected_wire(the_edges: &mut Vec<Shape>) -> Result<Wire, Failure> {
    let first = the_edges.remove(0);
    let mut m_wire = BRepLibMakeWire::from_edge(&topods::edge(&first)?)?;
    let mut good_wire = m_wire.wire()?;
    loop {
        let mut add_edge = false;
        let mut i = 0;
        while i < the_edges.len() && !add_edge {
            let e = topods::edge(&the_edges[i])?;
            if BRepTool::degenerated(&e)? {
                the_edges.remove(i);
                add_edge = true;
                good_wire = m_wire.wire()?;
                continue;
            }
            m_wire.add_edge(&e)?;
            if m_wire.error() == BRepLibWireError::WireDone {
                the_edges.remove(i);
                add_edge = true;
                good_wire = m_wire.wire()?;
                continue;
            }
            i += 1;
        }
        if !add_edge {
            return Ok(good_wire);
        }
    }
}

/// `TrimFace(Face, TheEdges, S)` (static): the faces (copies of `face`) bounded by the
/// wires connecting the edges, completed with degenerated edges.
pub fn trim_face(face: &Face, the_edges: &mut Vec<Shape>) -> Result<Vec<Shape>, Failure> {
    let mut s = Vec::new();
    while !the_edges.is_empty() {
        let good_wire = connected_wire(the_edges)?;
        let mut face_cut = face.as_ref().empty_copied()?;
        face_cut.set_orientation(Orientation::Forward);
        let fc = topods::face(&face_cut)?;
        brep_tools::update_face(&fc)?;
        let mut w = good_wire.into_shape();
        add_degenerated_edge(&fc, &mut w)?;
        BRepBuilder::default().add(&mut face_cut, &w)?;
        face_cut.set_orientation(face.as_ref().orientation());
        s.push(face_cut);
    }
    Ok(s)
}
