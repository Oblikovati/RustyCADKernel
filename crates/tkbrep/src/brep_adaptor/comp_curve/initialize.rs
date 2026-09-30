// Rust translation of the BRepAdaptor_CompCurve::Initialize overloads
// (ModelingData/TKBRep/BRepAdaptor/BRepAdaptor_CompCurve.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepAdaptor_CompCurve::Initialize`: the edge adaptors and the knots of a wire.
use super::{BRepAdaptorCompCurve, CompState};
use crate::brep::BRepTool;
use crate::brep_adaptor::BRepAdaptorCurve;
use crate::brep_tools::WireExplorer;
use crate::topexp;
use crate::topods::{Edge, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use std::sync::Arc;

/// The non-degenerated edges of `wire`, in the order of the wire explorer.
fn explored_edges(wire: &Wire) -> Result<Vec<Edge>, Failure> {
    let mut edges = Vec::new();
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        if !BRepTool::degenerated(explorer.current())? {
            edges.push(explorer.current().clone());
        }
        explorer.next()?;
    }
    Ok(edges)
}

/// The adaptor of each edge and the knots: 0, then the cumulated lengths (`by_length`)
/// or the edge ranks.
fn curves_and_knots(
    edges: &[Edge],
    by_length: bool,
) -> Result<(Vec<Handle<BRepAdaptorCurve>>, Vec<f64>), Failure> {
    let mut curves = Vec::with_capacity(edges.len());
    let mut knots = vec![0.0];
    for (rank, edge) in edges.iter().enumerate() {
        let curve = BRepAdaptorCurve::with_edge(edge)?;
        let knot = if by_length {
            knots[rank] + AbscissaPoint::length(&curve as &dyn Adaptor3dCurve)?
        } else {
            (rank + 1) as f64
        };
        knots.push(knot);
        curves.push(Handle::new(curve));
    }
    Ok((curves, knots))
}

/// The parsing direction: FORWARD edges are parsed forward unless the second edge does
/// not start at the end of the first one (for more than two edges, or two edges of an
/// open wire); a REVERSED first edge inverts the rule.
fn parsing_direction(curves: &[Handle<BRepAdaptorCurve>], wire: &Wire) -> Result<bool, Failure> {
    let count = curves.len();
    if !(count > 2 || (count == 2 && !wire.closed()?)) {
        return Ok(true);
    }
    let (Some(first), Some(second)) = (curves[0].get(), curves[1].get()) else {
        return Ok(true);
    };
    let first_edge = first.edge();
    let reversed = first_edge.orientation() == Orientation::Reversed;
    let common = topexp::common_vertex(&first_edge, &second.edge()).unwrap_or_default();
    let last = topexp::last_vertex(&first_edge, false);
    if common.is_same(&last) {
        return Ok(!reversed);
    }
    Ok(reversed)
}

impl BRepAdaptorCompCurve {
    /// `Initialize(W, KnotByCurvilinearAbcissa)`: one adaptor per non-degenerated edge in
    /// wire order, parameterised on `[0, number of edges]` (or on `[0, wire length]`).
    /// A wire without such an edge keeps the previous edges, knots and range;
    /// `curve.initialize(&wire, false)?`.
    pub fn initialize(&self, wire: &Wire, by_curvilinear_abscissa: bool) -> Result<(), Failure> {
        {
            let mut state = self.state.write();
            state.wire = wire.clone();
            state.p_tol = 0.0;
            state.is_by_ac = by_curvilinear_abscissa;
        }
        let edges = explored_edges(wire)?;
        if edges.is_empty() {
            return Ok(());
        }
        let (curves, knots) = curves_and_knots(&edges, by_curvilinear_abscissa)?;
        let forward = parsing_direction(&curves, wire)?;
        let mut state = self.state.write();
        state.cur_index = (edges.len() as i32 + 1) / 2;
        state.t_first = 0.0;
        state.t_last = knots[knots.len() - 1];
        state.curves = Some(curves);
        state.knots = Some(Arc::new(knots));
        state.forward = forward;
        Ok(())
    }

    /// `Initialize(W, KnotByCurvilinearAbcissa, First, Last, Tol)`: the whole wire, then
    /// its range set to `[first, last]` with the knot tolerance `tolerance`, and the two
    /// extreme edge adaptors trimmed; `curve.initialize_range(&wire, false, 0.5, 1.5, 1e-7)?`.
    pub fn initialize_range(
        &self,
        wire: &Wire,
        by_curvilinear_abscissa: bool,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<(), Failure> {
        self.initialize(wire, by_curvilinear_abscissa)?;
        let mut state = self.state.read().clone();
        state.t_first = first;
        state.t_last = last;
        state.p_tol = tolerance;
        trim_extremities(&mut state)?;
        *self.state.write() = state;
        Ok(())
    }
}

/// Trims the edges holding the two ends of the range to the range.
fn trim_extremities(state: &mut CompState) -> Result<(), Failure> {
    let start = state.prepare(state.t_first, state.cur_index)?;
    let end = state.prepare(state.t_last, state.cur_index)?;
    let (i1, i2) = (start.index, end.index);
    let (f, l) = (start.parameter, end.parameter);
    state.cur_index = (i1 + i2) / 2;
    if i1 == i2 {
        let trimmed = trim_curve(state, i1, ordered(f, l))?;
        return set_curve(state, i1, trimmed);
    }
    let k = state.curve(i1)?.last_parameter()?;
    let first_trim = trim_curve(state, i1, if k > f { (f, k) } else { (k, f) })?;
    let k = state.curve(i2)?.first_parameter()?;
    let last_trim = trim_curve(state, i2, if k <= l { (k, l) } else { (l, k) })?;
    set_curve(state, i1, first_trim)?;
    set_curve(state, i2, last_trim)
}

/// The native `if (l > f) Trim(f, l) else Trim(l, f)` bounds.
fn ordered(f: f64, l: f64) -> (f64, f64) {
    if l > f { (f, l) } else { (l, f) }
}

/// `myCurves->Value(index).Trim(first, last, PTol)` as a BRepAdaptor_Curve.
fn trim_curve(
    state: &CompState,
    index: i32,
    (first, last): (f64, f64),
) -> Result<Handle<BRepAdaptorCurve>, Failure> {
    let trimmed = state.curve(index)?.trim(first, last, state.p_tol)?;
    Ok(trimmed.cast::<BRepAdaptorCurve>())
}

/// `myCurves->SetValue(index, *HC)`.
fn set_curve(
    state: &mut CompState,
    index: i32,
    curve: Handle<BRepAdaptorCurve>,
) -> Result<(), Failure> {
    state.curve(index)?;
    if let Some(curves) = state.curves.as_mut() {
        curves[(index - 1) as usize] = curve;
    }
    Ok(())
}
