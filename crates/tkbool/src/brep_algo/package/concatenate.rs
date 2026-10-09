// Rust translation of BRepAlgo::ConcatenateWire
// (ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The wire of the G1 or C1 concatenations of the B-spline conversions of its edges.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex, Wire};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomGeometry, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_convert::{self, HArrayOfBSplineCurves};
use rustycad_tkgeombase::geom_lprop;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeWire};

/// The B-spline of an edge on its range in the frame of its location, made C1 where
/// it can be (`GeomConvert::C0BSplineToC1BSplineCurve` with the default angular
/// tolerance).
pub(super) fn edge_bspline(edge: &Edge) -> Result<(Handle<GeomBSplineCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let a_tr_curve: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
        &found.curve,
        found.first,
        found.last,
        true,
        true,
    )?)
    .cast();
    let mut bs =
        geom_convert::curve_to_bspline_curve(&a_tr_curve, ParameterisationType::TgtThetaOver2)?;
    bs.get()
        .ok_or_else(null_curve)?
        .transform(found.location.transformation())?;
    geom_convert::c0_bspline_to_c1_bspline_curve(&mut bs, precision::confusion(), 1.0e-7)?;
    Ok((bs, found.first, found.last))
}

/// The edges of the wire in their connection order.
fn wire_edges(w: &Wire) -> Result<Vec<Edge>, Failure> {
    let mut edges: Vec<Edge> = Vec::new();
    let mut w_exp = WireExplorer::with_wire(w)?;
    while w_exp.more() {
        edges.push(w_exp.current().clone());
        w_exp.next()?;
    }
    Ok(edges)
}

/// The vertex of an edge at its start (or end) in the wire.
fn wire_end(edge: &Edge, at_start: bool) -> Vertex {
    let reversed: bool = edge.orientation() == Orientation::Reversed;
    if at_start != reversed {
        topexp::first_vertex(edge, false)
    } else {
        topexp::last_vertex(edge, false)
    }
}

/// An array error as a native exception.
fn array_failure(e: impl std::fmt::Debug) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepAlgo: {e:?}; expected an index in the array bounds"),
        "NCollection_Array1",
    )
}

/// The B-splines of the edges (reversed in the wire sense) and the vertex tolerances
/// between them; the first parameter of the first edge and the last of the last.
#[allow(clippy::type_complexity)] // The curves, the tolerances and the closing parameters.
fn edge_curves(
    edges: &[Edge],
) -> Result<
    (
        Array1<'static, Handle<GeomBSplineCurve>>,
        Array1<'static, f64>,
        (f64, f64),
    ),
    Failure,
> {
    let nb_curve = edges.len() as i32;
    let mut tab: Array1<'static, Handle<GeomBSplineCurve>> =
        Array1::with_bounds(0, nb_curve - 1).map_err(array_failure)?;
    let mut tabtolvertex: Array1<'static, f64> =
        Array1::with_bounds(0, nb_curve - 2).map_err(array_failure)?;
    let (mut first0, mut last) = (0., 0.);
    for (index, edge) in edges.iter().enumerate() {
        let (bs, first, l) = edge_bspline(edge)?;
        last = l;
        let reversed: bool = edge.orientation() == Orientation::Reversed;
        if index >= 1 {
            if reversed {
                bs.get().ok_or_else(null_curve)?.reverse()?;
            }
            let tolleft: f64 = BRepTool::vertex_tolerance(&topexp::last_vertex(edge, false))?;
            let tolright: f64 = BRepTool::vertex_tolerance(&topexp::first_vertex(edge, false))?;
            tabtolvertex[index as i32 - 1] = tolleft.max(tolright);
        } else {
            first0 = first;
            if reversed {
                bs.get().ok_or_else(null_curve)?.reverse()?;
            }
        }
        tab[index as i32] = bs;
    }
    Ok((tab, tabtolvertex, (first0, last)))
}

/// `BRepAlgo::ConcatenateWire(Wire, Option, AngularTolerance)` (natively 1e-4): the
/// wire of the G1 (or C1) concatenation of the B-splines of its edges, closed in G1
/// when its ends join within their tolerance; one edge is converted alone;
/// `brep_algo::concatenate_wire(&wire, Continuity::G1, 1e-4)?`.
pub fn concatenate_wire(w: &Wire, option: Continuity, tol_angular: f64) -> Result<Wire, Failure> {
    let edges = wire_edges(w)?;
    let mut make_result = BRepLibMakeWire::new();
    if edges.len() <= 1 {
        let edge = edges.first().cloned().unwrap_or_default();
        let (bs, _, _) = edge_bspline(&edge)?;
        if edge.orientation() == Orientation::Reversed {
            bs.get().ok_or_else(null_curve)?.reverse()?;
        }
        make_result.add_edge(&BRepLibMakeEdge::from_curve(&bs.cast())?.edge()?)?;
        return make_result.wire();
    }
    let (mut tab, tabtolvertex, (first0, last)) = edge_curves(&edges)?;
    let (vfirst, vlast) = (
        wire_end(&edges[0], true),
        wire_end(&edges[edges.len() - 1], false),
    );
    let toler: f64 = BRepTool::vertex_tolerance(&vfirst)?.max(BRepTool::vertex_tolerance(&vlast)?);
    let (pfirst, plast) = (BRepTool::pnt(&vfirst)?, BRepTool::pnt(&vlast)?);
    let (mut closed_flag, mut closed_tolerance) = (false, 0.0);
    // The C0 continuity test at the closing point.
    if pfirst.distance(&plast) <= toler {
        let c_last: Handle<dyn GeomCurve> = tab[edges.len() as i32 - 1].clone().cast();
        let c_first: Handle<dyn GeomCurve> = tab[0].clone().cast();
        if geom_lprop::continuity_with_tolerances(
            &c_last,
            &c_first,
            last,
            first0,
            true,
            true,
            toler,
            tol_angular,
        )? >= Continuity::G1
        {
            (closed_tolerance, closed_flag) = (toler, true);
        }
    }
    let mut concatcurve: HArrayOfBSplineCurves = Handle::default();
    if option == Continuity::G1 {
        geom_convert::concat_g1(
            &mut tab,
            &tabtolvertex,
            &mut concatcurve,
            &mut closed_flag,
            closed_tolerance,
        )?;
    } else {
        let mut array_of_indices: Handle<HArray1<i32>> = Handle::default();
        geom_convert::concat_c1(
            &mut tab,
            &tabtolvertex,
            &mut array_of_indices,
            &mut concatcurve,
            &mut closed_flag,
            closed_tolerance,
        )?;
    }
    let curves = concatcurve.get().ok_or_else(null_curve)?;
    let array = curves.array1();
    for index in array.lower()..=array.upper() {
        let curve: Handle<dyn GeomCurve> =
            array.value(index).map_err(array_failure)?.clone().cast();
        make_result.add_edge(&BRepLibMakeEdge::from_curve(&curve)?.edge()?)?;
    }
    make_result.wire()
}

/// Standard_NullObject of a null curve (natively a crash).
pub(super) fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepAlgo: null curve; expected a curve",
        "BRepAlgo",
    )
}
