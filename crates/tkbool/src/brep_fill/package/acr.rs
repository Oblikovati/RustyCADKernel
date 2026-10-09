// Rust translation of BRepFill::ComputeACR, BRepFill::InsertACR and the static TrimEdge
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reduced curvilinear abscissas (ACR) of a wire, and the cuts of a wire at given
//! abscissas.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeWire};

/// `BRepFill::ComputeACR(wire, ACR)`: `ACR(0)` the length of the wire, `ACR(i)` the
/// cumulated length at the end of its i-th edge divided by it (degenerated edges add
/// nothing); for a punctual wire the last value is 1 and the others stay lengths. The
/// array is indexed from 0 to the number of edges.
///
/// ```ignore
/// let mut acr = Array1::with_bounds(0, nb_edges)?;
/// rustycad_tkbool::brep_fill::compute_acr(&wire, &mut acr)?;
/// ```
pub fn compute_acr(wire: &Wire, acr: &mut Array1<'_, f64>) -> Result<(), Failure> {
    acr.init(0.);
    let mut nb_edges: i32 = 0;
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        nb_edges += 1;
        let edge: &Edge = explorer.current();
        let mut cumulated: f64 = at(acr, nb_edges - 1)?;
        if !BRepTool::degenerated(edge)? {
            let curve = BRepAdaptorCurve::with_edge(edge)?;
            cumulated += AbscissaPoint::length(&curve as &dyn Adaptor3dCurve)?;
        }
        put(acr, nb_edges, cumulated)?;
        explorer.next()?;
    }
    // The total length of the wire.
    let total: f64 = at(acr, nb_edges)?;
    put(acr, 0, total)?;
    if total > precision::confusion() {
        for i in 1..=nb_edges {
            put(acr, i, at(acr, i)? / total)?;
        }
    } else {
        // A punctual wire.
        put(acr, nb_edges, 1.)?;
    }
    Ok(())
}

/// The number of edges met by the wire explorer.
fn explored_edges(wire: &Wire) -> Result<i32, Failure> {
    let mut count: i32 = 0;
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        count += 1;
        explorer.next()?;
    }
    Ok(count)
}

/// `BRepFill::InsertACR(wire, ACRcuts, prec)`: the wire with its edges cut at the
/// reduced abscissas `acr_cuts` that fall strictly inside them (by more than `prec`),
/// in the orientation of the input wire.
///
/// ```ignore
/// let cut = rustycad_tkbool::brep_fill::insert_acr(&wire, &cuts, 1e-6)?;
/// ```
pub fn insert_acr(wire: &Wire, acr_cuts: &Array1<'_, f64>, prec: f64) -> Result<Wire, Failure> {
    let nb_edges: i32 = explored_edges(wire)?;
    let mut acr_wire: Array1<'static, f64> =
        Array1::with_bounds(0, nb_edges).map_err(array_error)?;
    compute_acr(wire, &mut acr_wire)?;
    let mut maker = BRepLibMakeWire::new();
    let mut t1: f64 = 0.;
    let mut rank: i32 = 0;
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        rank += 1;
        let t0: f64 = t1;
        t1 = at(&acr_wire, rank)?;
        let paradec: Vec<f64> = cuts_inside(acr_cuts, (t0, t1), prec)?;
        let edge: &Edge = explorer.current();
        if paradec.is_empty() || BRepTool::degenerated(edge)? {
            // Copy the edge.
            maker.add_edge(edge)?;
        } else {
            for piece in cut_pieces(edge, explorer.current_vertex(), &paradec, (t0, t1))? {
                maker.add_edge(&topods::edge(&piece)?)?;
            }
        }
        explorer.next()?;
    }
    let result: Shape = maker.wire()?.into_shape().oriented(wire.orientation());
    topods::wire(&result)
}

/// The cuts strictly inside `(t0 + prec, t1 - prec)`, read as `ACRcuts(i)` for `i` in
/// `1..=Length()` whatever the lower bound of the array (as natively).
fn cuts_inside(
    acr_cuts: &Array1<'_, f64>,
    range: (f64, f64),
    prec: f64,
) -> Result<Vec<f64>, Failure> {
    let (t0, t1) = range;
    let mut inside: Vec<f64> = Vec::new();
    for i in 1..=acr_cuts.len() as i32 {
        let cut: f64 = at(acr_cuts, i)?;
        if t0 + prec < cut && cut < t1 - prec {
            inside.push(cut);
        }
    }
    Ok(inside)
}

/// The pieces of an edge cut at the abscissas `paradec` of the wire (mapped onto a
/// REVERSED edge), in the direction in which the wire runs through it.
fn cut_pieces(
    edge: &Edge,
    vertex: &Vertex,
    paradec: &[f64],
    range: (f64, f64),
) -> Result<Vec<Shape>, Failure> {
    let (t0, t1) = range;
    let seq_order: bool = vertex.is_same(&topexp::first_vertex(edge, false));
    // The wire is always FORWARD; the cut parameters are mirrored for a REVERSED edge.
    let cuts: Vec<f64> = if edge.orientation() == Orientation::Forward {
        paradec.to_vec()
    } else {
        paradec.iter().rev().map(|&c| t0 + t1 - c).collect()
    };
    trim_edge(edge, &cuts, t0, t1, seq_order)
}

/// `TrimEdge`: the pieces of the edge between the abscissas `cut_values` (mapped
/// linearly from `[t0, t1]` onto the curve range), from its first vertex when
/// `seq_order`, from its last one otherwise; each piece keeps the edge orientation.
fn trim_edge(
    edge: &Edge,
    cut_values: &[f64],
    t0: f64,
    t1: f64,
    seq_order: bool,
) -> Result<Vec<Shape>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let orientation: Orientation = edge.orientation();
    let (vf, vl) = topexp::vertices(edge, false);
    let vbid = Vertex::default();
    let piece = |v0: &Vertex, v1: &Vertex, m0: f64, m1: f64| -> Result<Edge, Failure> {
        let mut cut: Edge =
            BRepLibMakeEdge::from_curve_vertices_params(&curve, v0, v1, m0, m1)?.edge()?;
        cut.set_orientation(orientation);
        Ok(cut)
    };
    let ndec: usize = cut_values.len();
    let mut pieces: Vec<Shape> = Vec::new();
    let map = |value: f64| (value - t0) * (last - first) / (t1 - t0) + first;
    if seq_order {
        // From first to last.
        let (mut m0, mut v0) = (first, vf.clone());
        for (j, &value) in cut_values.iter().enumerate() {
            let m1: f64 = map(value);
            let cut: Edge = piece(&v0, &vbid, m0, m1)?;
            m0 = m1;
            v0 = topexp::last_vertex(&cut, false);
            pieces.push(cut.into_shape());
            if j + 1 == ndec {
                pieces.push(piece(&v0, &vl, m0, last)?.into_shape());
            }
        }
        return Ok(pieces);
    }
    // From last to first.
    let (mut m1, mut v1) = (last, vl.clone());
    for j in (0..ndec).rev() {
        let m0: f64 = map(cut_values[j]);
        let cut: Edge = piece(&vbid, &v1, m0, m1)?;
        m1 = m0;
        v1 = topexp::first_vertex(&cut, false);
        pieces.push(cut.into_shape());
        if j == 0 {
            pieces.push(piece(&vf, &v1, first, m1)?.into_shape());
        }
    }
    Ok(pieces)
}

/// Standard_OutOfRange for an array that cannot be allocated.
fn array_error(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    let message = format!("BRepFill::InsertACR: {error}; expected an array of the edge count");
    Failure::with_type("Standard_OutOfRange", &message, "BRepFill")
}
