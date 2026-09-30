//! Native parameter scaling and persistent projection for non-SameParameter edges.
use crate::imesh_data::{EdgeHandle, FaceHandle};
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorCurve, topods};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, top_abs::Orientation};
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkmath::gp::Point3;
mod collection;
mod runtime;
pub use collection::EdgeParameterCollection;
struct ParameterProjectionState {
    parameters: Option<Box<dyn EdgeParameterCollection>>,
    same_parameter: bool,
    first: f64,
    old_first: f64,
    scale: f64,
    current: f64,
    found: f64,
    projector: LocateExtPC<'static>,
}
impl Default for ParameterProjectionState {
    fn default() -> Self {
        Self {
            parameters: None,
            same_parameter: false,
            first: 0.0,
            old_first: 0.0,
            scale: 0.0,
            current: 0.0,
            found: 0.0,
            projector: LocateExtPC::new(),
        }
    }
}
/// Parameters of a discrete edge on its face; `provider.parameter(index, point)?`.
pub struct EdgeParameterProvider {
    transient: StandardTransient,
    adaptor: Handle<BRepAdaptorCurve>,
    state: SharedState<ParameterProjectionState>,
}
impl EdgeParameterProvider {
    /// Empty native provider; `EdgeParameterProvider::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            adaptor: Handle::new(BRepAdaptorCurve::default()),
            state: SharedState::new(ParameterProjectionState::default()),
        }
    }
    /// Initialize edge, face and shared parameter collection; `EdgeParameterProvider::with_edge(&edge, orientation, &face, parameters)?`.
    pub fn with_edge<P: EdgeParameterCollection + 'static>(
        edge: &EdgeHandle,
        orientation: Orientation,
        face: &FaceHandle,
        parameters: P,
    ) -> Result<Self, Failure> {
        let provider = Self::new();
        provider.init(edge, orientation, face, parameters)?;
        Ok(provider)
    }
    /// Replace inputs while preserving native cache initialization order; `provider.init(&edge, orientation, &face, parameters)?`.
    pub fn init<P: EdgeParameterCollection + 'static>(
        &self,
        edge: &EdgeHandle,
        orientation: Orientation,
        face: &FaceHandle,
        parameters: P,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.parameters = Some(Box::new(parameters));
        state.same_parameter = edge.get_same_param();
        state.scale = 1.0;
        let oriented = topods::edge(&edge.get_edge()?.oriented(orientation))?;
        self.adaptor
            .initialize_on_face(&oriented, &face.get_face()?)?;
        if state.same_parameter {
            return Ok(());
        }
        state.initialize_projection(&self.adaptor)
    }
    /// Stored value for SameParameter, otherwise monotonically guarded projection; `provider.parameter(1, point)?`.
    pub fn parameter(&self, index: i32, point: Point3) -> Result<f64, Failure> {
        let mut state = self.state.write();
        let parameter = state
            .parameters
            .as_ref()
            .ok_or_else(uninitialized_parameters)?
            .value(index)?;
        if state.same_parameter {
            return Ok(parameter);
        }
        state.project_parameter(parameter, point)
    }
    /// Shared pcurve of the face adaptor; `provider.get_pcurve()?`.
    pub fn get_pcurve(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Ok(self
            .adaptor
            .transformed_curve()
            .curve_on_surface()?
            .get_curve())
    }
}
impl ParameterProjectionState {
    fn initialize_projection(&mut self, adaptor: &Handle<BRepAdaptorCurve>) -> Result<(), Failure> {
        self.first = adaptor.first_parameter()?;
        let last = adaptor.last_parameter()?;
        self.current = self.first;
        self.found = self.first;
        let parameters = self
            .parameters
            .as_ref()
            .ok_or_else(uninitialized_parameters)?;
        self.old_first = parameters.value(parameters.lower())?;
        let old_last = parameters.value(parameters.upper())?;
        if (self.old_first != self.first || old_last != last) && self.old_first != old_last {
            self.scale = (last - self.first) / (old_last - self.old_first);
        }
        self.projector.initialize_owned(
            adaptor.cast(),
            (adaptor.first_parameter()?, adaptor.last_parameter()?),
            precision::p_confusion(),
        )
    }
    fn project_parameter(&mut self, parameter: f64, point: Point3) -> Result<f64, Failure> {
        let previous = self.current;
        self.current = self.first + self.scale * (parameter - self.old_first);
        let previous_found = self.found;
        self.found += self.current - previous;
        self.projector.perform(point, self.found)?;
        if self.projector.is_done() {
            let projected = self.projector.point()?.parameter();
            // OCCT 3d097a03 rejects projections crossing the previous result to avoid self-intersections.
            if (previous_found < self.found && previous_found < projected)
                || (previous_found > self.found && previous_found > projected)
            {
                self.found = projected;
            }
        }
        Ok(self.found)
    }
}
fn uninitialized_parameters() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "uninitialized parameters; expected initialized edge parameter provider",
    )
}

impl Default for EdgeParameterProvider {
    fn default() -> Self {
        Self::new()
    }
}
