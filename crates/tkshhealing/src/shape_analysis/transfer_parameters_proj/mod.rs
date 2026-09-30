//! Projection-based parameter transfer, preserving OCCT 3d097a0 selection and mutation order.
use super::{
    ShapeAnalysisCurve, ShapeAnalysisEdge, ShapeAnalysisTransferParameters, TransferParameterData,
    TransferParameters,
};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face},
};
use rustycad_tkernel::precision;
use rustycad_tkernel::{
    impl_transient,
    ncollection::HSequence,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg2d::{geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface},
    geom::GeomCurve,
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkmath::{gp::Point3, toploc::Location};
mod initialize;
mod projection;
mod range;
mod sequence;
mod vertex;
#[derive(Clone)]
struct ProjectionTransferState {
    spatial: Handle<dyn GeomCurve>,
    planar: Handle<dyn Geom2dCurve>,
    adaptor: Handle<Adaptor3dCurveOnSurface>,
    location: Location,
    precision: f64,
    force: bool,
    initialized: bool,
}
impl Default for ProjectionTransferState {
    fn default() -> Self {
        Self {
            spatial: Handle::default(),
            planar: Handle::default(),
            adaptor: Handle::new(Adaptor3dCurveOnSurface::default()),
            location: Location::default(),
            precision: 0.,
            force: false,
            initialized: false,
        }
    }
}
/// Project between edge parameterizations; `ShapeAnalysisTransferParametersProj::new()`.
pub struct ShapeAnalysisTransferParametersProj {
    parent: ShapeAnalysisTransferParameters,
    state: SharedState<ProjectionTransferState>,
}
impl Default for ShapeAnalysisTransferParametersProj {
    fn default() -> Self {
        let parent = ShapeAnalysisTransferParameters::new();
        parent.set_max_tolerance(1.);
        Self {
            parent,
            state: SharedState::new(ProjectionTransferState::default()),
        }
    }
}
impl_transient!(ShapeAnalysisTransferParametersProj,"ShapeAnalysis_TransferParametersProj",ShapeAnalysisTransferParameters,parent,interfaces:[dyn TransferParameters]);
impl std::ops::Deref for ShapeAnalysisTransferParametersProj {
    type Target = ShapeAnalysisTransferParameters;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl TransferParameters for ShapeAnalysisTransferParametersProj {
    fn transfer_parameter_data(&self) -> &TransferParameterData {
        self.parent.transfer_parameter_data()
    }
    fn init(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        initialize::initialize(self, edge, face)
    }
    fn perform_parameter(&self, parameter: f64, to_2d: bool) -> Result<f64, Failure> {
        let state = self.state.snapshot();
        if !self.requires_projection(&state)? {
            return self.parent.perform_parameter(parameter, to_2d);
        }
        let bounds = self.target_bounds(&state, to_2d)?;
        let projected = projection::segment(self, &state, parameter, to_2d, bounds)?;
        Ok(projection::clamp_parameter(projected, bounds))
    }
    fn perform(
        &self,
        parameters: &Handle<HSequence<f64>>,
        to_2d: bool,
    ) -> Result<Handle<HSequence<f64>>, Failure> {
        let state = self.state.snapshot();
        if !self.requires_projection(&state)? {
            return self.parent.perform(parameters, to_2d);
        }
        sequence::transfer(self, &state, parameters, to_2d)
    }
    fn transfer_range(
        &self,
        edge: &Edge,
        previous: f64,
        current: f64,
        is_2d: bool,
    ) -> Result<(), Failure> {
        let state = self.state.snapshot();
        if !self.requires_projection(&state)? {
            return self.parent.transfer_range(edge, previous, current, is_2d);
        }
        range::transfer(self, &state, edge, previous, current, is_2d)
    }
    fn is_same_range(&self) -> bool {
        let state = self.state.snapshot();
        // Successful initialization retains a valid edge; failed initialization clears the guard.
        !self
            .requires_projection(&state)
            .expect("initialized transfer requires a valid BRep edge")
            && self.parent.is_same_range()
    }
}
impl ShapeAnalysisTransferParametersProj {
    /// Create an inactive projection tool with tolerance one; `ShapeAnalysisTransferParametersProj::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Initialize both projection adaptors; `ShapeAnalysisTransferParametersProj::from_edge(&edge,&face)?`.
    pub fn from_edge(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let tool = Self::new();
        tool.init(edge, face)?;
        Ok(tool)
    }
    /// Inspect the native force flag; `tool.force_projection()`.
    pub fn force_projection(&self) -> bool {
        self.state.read().force
    }
    /// Change the native force flag without rebuilding adaptors; `tool.set_force_projection(true)`.
    pub fn set_force_projection(&self, force: bool) {
        self.state.write().force = force;
    }
    fn requires_projection(&self, state: &ProjectionTransferState) -> Result<bool, Failure> {
        if !state.initialized {
            return Ok(false);
        }
        let (edge, maximum) = self
            .transfer_parameter_data()
            .with(|fields| (fields.edge.clone(), fields.max_tolerance));
        Ok(!(!state.force && state.precision < maximum && BRepTool::same_parameter(&edge)?))
    }
    fn target_bounds(
        &self,
        state: &ProjectionTransferState,
        to_2d: bool,
    ) -> Result<(f64, f64), Failure> {
        if to_2d {
            return Ok((
                state.adaptor.first_parameter()?,
                state.adaptor.last_parameter()?,
            ));
        }
        Ok(self
            .transfer_parameter_data()
            .with(|fields| (fields.first, fields.last)))
    }
}
