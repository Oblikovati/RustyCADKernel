//! Rust translation of ShapeFix_EdgeProjAux at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod init_2d;
mod init_3d;
mod parameter_update;
mod projection;

use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{SharedState, StandardTransient};

/// Project edge endpoints onto the p-curve to determine its usable parameter range.
///
/// ```
/// use rustycad_tkshhealing::shape_fix::ShapeFixEdgeProjAux;
/// let projection = ShapeFixEdgeProjAux::new();
/// assert!(!projection.is_first_done());
/// ```
pub struct ShapeFixEdgeProjAux {
    parent: StandardTransient,
    state: SharedState<EdgeProjAuxState>,
}
impl_transient!(
    ShapeFixEdgeProjAux,
    "ShapeFix_EdgeProjAux",
    StandardTransient,
    parent
);

#[derive(Clone, Default)]
struct EdgeProjAuxState {
    pub(super) face: Face,
    pub(super) edge: Edge,
    pub(super) first_param: f64,
    pub(super) last_param: f64,
    pub(super) first_done: bool,
    pub(super) last_done: bool,
}

impl Default for ShapeFixEdgeProjAux {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(EdgeProjAuxState::default()),
        }
    }
}

impl ShapeFixEdgeProjAux {
    /// Construct without an edge; `ShapeFixEdgeProjAux::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct and initialize the edge projection; `ShapeFixEdgeProjAux::with_face_edge(&face, &edge)`.
    pub fn with_face_edge(face: &Face, edge: &Edge) -> Self {
        let projection = Self::new();
        projection.init(face, edge);
        projection
    }

    /// Replace the current face and edge and reset computed endpoints; `tool.init(&face, &edge)`.
    pub fn init(&self, face: &Face, edge: &Edge) {
        *self.state.write() = EdgeProjAuxState {
            face: face.clone(),
            edge: edge.clone(),
            ..EdgeProjAuxState::default()
        };
    }

    /// Recompute the p-curve range from 3D endpoints; `tool.compute(1e-7)?`.
    pub fn compute(&self, precision: f64) -> Result<(), rustycad_tkernel::standard::Failure> {
        let mut state = self.state.write();
        reset_projection(&mut state);
        init_2d::initialize(&mut state, precision)?;
        finish_computation(&mut state);
        Ok(())
    }

    /// Initialize projected endpoints from the edge's 3D vertices and face p-curve.
    ///
    /// This exposes OCCT's protected `Init2d` operation for derived repair tools.
    ///
    /// ```
    /// use rustycad_tkshhealing::shape_fix::ShapeFixEdgeProjAux;
    /// let projection = ShapeFixEdgeProjAux::new();
    /// let _ = projection.init_2d(1.0e-7);
    /// ```
    pub fn init_2d(&self, precision: f64) -> Result<(), rustycad_tkernel::standard::Failure> {
        let mut state = self.state.write();
        reset_projection(&mut state);
        init_2d::initialize(&mut state, precision)
    }

    /// Initialize the range from a bounded p-curve or nearest endpoint extrema.
    ///
    /// This exposes OCCT's protected `Init3d` operation for derived repair tools.
    ///
    /// ```
    /// use rustycad_tkshhealing::shape_fix::ShapeFixEdgeProjAux;
    /// let projection = ShapeFixEdgeProjAux::new();
    /// let _ = projection.init_3d(1.0e-7);
    /// ```
    pub fn init_3d(&self, precision: f64) -> Result<(), rustycad_tkernel::standard::Failure> {
        init_3d::initialize(&mut self.state.write(), precision)
    }

    /// Adjust an endpoint parameter pair to the supplied p-curve's orientation and period.
    ///
    /// This exposes OCCT's protected `UpdateParam2d` operation for derived repair tools.
    ///
    /// ```
    /// use rustycad_tkshhealing::shape_fix::ShapeFixEdgeProjAux;
    /// use rustycad_tkernel::standard::Handle;
    /// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    /// use rustycad_tkmath::gp::{Direction2, Point2};
    /// let projection = ShapeFixEdgeProjAux::new();
    /// let line: Handle<dyn Geom2dCurve> =
    ///     Handle::new(Geom2dLine::from_point(Point2::default(), Direction2::new(1.0, 0.0).unwrap())).cast();
    /// let _ = projection.update_param_2d(&line);
    /// ```
    pub fn update_param_2d(
        &self,
        curve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        parameter_update::update_parameters(&mut self.state.write(), curve)
    }

    /// Whether the first endpoint was projected; `tool.is_first_done()`.
    pub fn is_first_done(&self) -> bool {
        self.state.read().first_done
    }

    /// Whether the last endpoint was projected; `tool.is_last_done()`.
    pub fn is_last_done(&self) -> bool {
        self.state.read().last_done
    }

    /// The usable first p-curve parameter; `tool.first_param()`.
    pub fn first_param(&self) -> f64 {
        self.state.read().first_param
    }

    /// The usable last p-curve parameter; `tool.last_param()`.
    pub fn last_param(&self) -> f64 {
        self.state.read().last_param
    }

    /// Report whether `curve` is an ISO; the current OCCT implementation always returns false.
    pub fn is_iso(
        &self,
        _curve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    ) -> bool {
        false
    }
}

fn reset_projection(state: &mut EdgeProjAuxState) {
    state.first_done = false;
    state.last_done = false;
    state.first_param = 0.0;
    state.last_param = 0.0;
}

fn finish_computation(state: &mut EdgeProjAuxState) {
    if !(state.first_done && state.last_done) {
        return;
    }
    if state.first_param >= state.last_param {
        std::mem::swap(&mut state.first_param, &mut state.last_param);
    }
    state.first_done = true;
    state.last_done = true;
}
