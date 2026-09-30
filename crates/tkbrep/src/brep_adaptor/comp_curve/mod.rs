// Rust translation of ModelingData/TKBRep/BRepAdaptor/BRepAdaptor_CompCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepAdaptor_CompCurve`: a wire seen as one 3D curve.
mod adaptor;
mod initialize;
mod prepare;

use super::BRepAdaptorCurve;
use crate::brep::errors::null_object;
use crate::topods::{Edge, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use std::sync::Arc;

/// The edge of the wire at a parameter and the parameter on that edge
/// (`BRepAdaptor_CompCurve::Edge(U, E, UonE)`).
#[derive(Clone, Debug, Default)]
pub struct CompCurveEdge {
    /// The edge (`E`).
    pub edge: Edge,
    /// The parameter on the edge (`UonE`).
    pub parameter: f64,
}

/// The fields of the native class.
#[derive(Clone)]
struct CompState {
    wire: Wire,
    t_first: f64,
    t_last: f64,
    p_tol: f64,
    /// `myCurves`, null (`None`) until an initialisation finds an edge.
    curves: Option<Vec<Handle<BRepAdaptorCurve>>>,
    /// `myKnots`, shared by the shallow copies as the native handle is.
    knots: Option<Arc<Vec<f64>>>,
    cur_index: i32,
    forward: bool,
    is_by_ac: bool,
}

impl Default for CompState {
    fn default() -> Self {
        Self {
            wire: Wire::default(),
            t_first: 0.0,
            t_last: 0.0,
            p_tol: 0.0,
            curves: None,
            knots: None,
            cur_index: -1,
            forward: false,
            is_by_ac: false,
        }
    }
}

impl CompState {
    /// `myCurves`; Standard_NullObject before any edge was loaded (a null handle
    /// dereference natively).
    fn curves(&self) -> Result<&[Handle<BRepAdaptorCurve>], Failure> {
        self.curves
            .as_deref()
            .ok_or_else(|| null_object("BRepAdaptor_CompCurve: no edge is loaded (null myCurves)"))
    }

    /// `myCurves->Value(index)` (1-based); Standard_OutOfRange outside the array.
    fn curve(&self, index: i32) -> Result<&BRepAdaptorCurve, Failure> {
        let curves = self.curves()?;
        usize::try_from(index - 1)
            .ok()
            .and_then(|position| curves.get(position))
            .and_then(|curve| curve.get())
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "BRepAdaptor_CompCurve: curve index {index}; expected 1..={}",
                    curves.len()
                ))
            })
    }

    /// `myKnots->Value(index)` (1-based); Standard_OutOfRange outside the array.
    fn knot(&self, index: i32) -> Result<f64, Failure> {
        let knots = self.knots.as_deref().ok_or_else(|| {
            null_object("BRepAdaptor_CompCurve: no edge is loaded (null myKnots)")
        })?;
        usize::try_from(index - 1)
            .ok()
            .and_then(|position| knots.get(position).copied())
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "BRepAdaptor_CompCurve: knot index {index}; expected 1..={}",
                    knots.len()
                ))
            })
    }
}

/// A wire seen as one 3D curve (`BRepAdaptor_CompCurve`): its non-degenerated edges
/// laid end to end, each on a parameter interval of length 1 or, with
/// `KnotByCurvilinearAbcissa`, of its length.
///
/// ```
/// use rustycad_tkbrep::brep_adaptor::BRepAdaptorCompCurve;
/// use rustycad_tkbrep::topods::Wire;
/// use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
/// let curve = BRepAdaptorCompCurve::with_wire(&Wire::default(), false)?;
/// assert_eq!(curve.first_parameter()?, 0.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepAdaptorCompCurve {
    parent: StandardTransient,
    state: SharedState<CompState>,
}
impl_transient!(
    BRepAdaptorCompCurve,
    "BRepAdaptor_CompCurve",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl BRepAdaptorCompCurve {
    fn from_state(state: CompState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }

    /// `BRepAdaptor_CompCurve(W, KnotByCurvilinearAbcissa)`;
    /// `BRepAdaptorCompCurve::with_wire(&wire, false)?`.
    pub fn with_wire(wire: &Wire, by_curvilinear_abscissa: bool) -> Result<Self, Failure> {
        let curve = Self::from_state(CompState {
            wire: wire.clone(),
            is_by_ac: by_curvilinear_abscissa,
            ..CompState::default()
        });
        curve.initialize(wire, by_curvilinear_abscissa)?;
        Ok(curve)
    }

    /// `BRepAdaptor_CompCurve(W, KnotByCurvilinearAbcissa, First, Last, Tol)`: the wire
    /// trimmed to `[first, last]`; `BRepAdaptorCompCurve::with_range(&wire, false, 0.0, 1.0, 1e-7)?`.
    pub fn with_range(
        wire: &Wire,
        by_curvilinear_abscissa: bool,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let curve = Self::from_state(CompState {
            wire: wire.clone(),
            t_first: first,
            t_last: last,
            p_tol: tolerance,
            is_by_ac: by_curvilinear_abscissa,
            ..CompState::default()
        });
        curve.initialize_range(wire, by_curvilinear_abscissa, first, last, tolerance)?;
        Ok(curve)
    }

    /// `Wire()`; `curve.wire()`.
    pub fn wire(&self) -> Wire {
        self.state.read().wire.clone()
    }

    /// `Edge(U, E, UonE)`: the edge holding the parameter `u` of the wire and the
    /// matching parameter on it; `let found = curve.edge(0.5)?;`.
    pub fn edge(&self, u: f64) -> Result<CompCurveEdge, Failure> {
        let state = self.state.read().clone();
        let located = state.prepare(u, state.cur_index)?;
        Ok(CompCurveEdge {
            edge: state.curve(located.index)?.edge(),
            parameter: located.parameter,
        })
    }
}
