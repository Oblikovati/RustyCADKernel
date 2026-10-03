//! Linear edge parameter transfer at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::HSequence,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
};
use rustycad_tkmath::gp::resolution;
mod operations;
/// Protected transfer state for projection-based subclasses; `TransferParameterFields::default()`.
#[derive(Default)]
pub struct TransferParameterFields {
    /// Source spatial range start.
    pub first: f64,
    /// Source spatial range end.
    pub last: f64,
    /// Source edge retained for copying all curve ranges.
    pub edge: Edge,
    /// Tolerance retained for derived projection policies.
    pub max_tolerance: f64,
    shift: f64,
    scale: f64,
    first_2d: f64,
    last_2d: f64,
    face: Face,
}
/// Shared transfer state preserves native subclass identity; `TransferParameterData::default()`.
pub struct TransferParameterData {
    fields: SharedState<TransferParameterFields>,
}
impl Default for TransferParameterData {
    fn default() -> Self {
        Self {
            fields: SharedState::new(TransferParameterFields {
                scale: 1.,
                ..TransferParameterFields::default()
            }),
        }
    }
}
impl TransferParameterData {
    /// Inspect protected state; `state.with(|fields| fields.first)`.
    pub fn with<R>(&self, read: impl FnOnce(&TransferParameterFields) -> R) -> R {
        read(&self.fields.read())
    }
    /// Update subclass projection state; `state.with_mut(|fields| fields.max_tolerance=1e-6)`.
    pub fn with_mut<R>(&self, write: impl FnOnce(&mut TransferParameterFields) -> R) -> R {
        write(&mut self.fields.write())
    }
}
/// Virtual transfer operations retain derived scalar dispatch; `tool.perform_parameter(0.5,true)?`.
pub trait TransferParameters: Transient {
    /// Shared protected storage; `tool.transfer_parameter_data()`.
    fn transfer_parameter_data(&self) -> &TransferParameterData;
    /// Stores the upper bound used by projection-based implementations; `tool.set_max_tolerance(1e-6)`.
    fn set_max_tolerance(&self, tolerance: f64) {
        self.transfer_parameter_data()
            .with_mut(|fields| fields.max_tolerance = tolerance);
    }
    /// Discover unoriented curve ranges; `tool.init(&edge,&face)?`.
    fn init(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        operations::initialize(self.transfer_parameter_data(), edge, face)
    }
    /// Transform an independent sequence through virtual scalar transfer; `tool.perform(&parameters,true)?`.
    fn perform(
        &self,
        parameters: &Handle<HSequence<f64>>,
        to_2d: bool,
    ) -> Result<Handle<HSequence<f64>>, Failure> {
        operations::transfer_sequence(self, parameters, to_2d)
    }
    /// Transform one parameter, preserving native arithmetic order; `tool.perform_parameter(0.5,true)?`.
    fn perform_parameter(&self, parameter: f64, to_2d: bool) -> Result<f64, Failure> {
        Ok(self.transfer_parameter_data().with(|fields| {
            if to_2d {
                return fields.shift + parameter * fields.scale;
            }
            -fields.shift / fields.scale + parameter * 1. / fields.scale
        }))
    }
    /// Copy every matching representation range; `tool.transfer_range(&edge,0.,1.,false)?`.
    fn transfer_range(
        &self,
        edge: &Edge,
        previous: f64,
        current: f64,
        is_2d: bool,
    ) -> Result<(), Failure> {
        let (source, alpha, beta) =
            operations::range_fractions(self.transfer_parameter_data(), previous, current, is_2d);
        ShapeBuildEdge.copy_ranges(edge, &source, alpha, beta)
    }
    /// Compare exact linear coefficients; `tool.is_same_range()`.
    fn is_same_range(&self) -> bool {
        self.transfer_parameter_data()
            .with(|fields| fields.shift == 0. && fields.scale == 1.)
    }
}
impl_transient_interface!(
    dyn TransferParameters,
    "ShapeAnalysis_TransferParameters",
    StandardTransient
);
/// Base linear parameter transfer; `ShapeAnalysisTransferParameters::new()`.
#[derive(Default)]
pub struct ShapeAnalysisTransferParameters {
    parent: StandardTransient,
    state: TransferParameterData,
}
impl_transient!(ShapeAnalysisTransferParameters,"ShapeAnalysis_TransferParameters",StandardTransient,parent,interfaces:[dyn TransferParameters]);
impl TransferParameters for ShapeAnalysisTransferParameters {
    fn transfer_parameter_data(&self) -> &TransferParameterData {
        &self.state
    }
}
impl ShapeAnalysisTransferParameters {
    /// Create the identity transform; `ShapeAnalysisTransferParameters::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Initialize a transform from an edge and face; `ShapeAnalysisTransferParameters::from_edge(&edge,&face)?`.
    pub fn from_edge(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let tool = Self::new();
        tool.init(edge, face)?;
        Ok(tool)
    }
    /// Store the tolerance used by subclasses; `tool.set_max_tolerance(1e-6)`.
    pub fn set_max_tolerance(&self, tolerance: f64) {
        self.state
            .with_mut(|fields| fields.max_tolerance = tolerance);
    }
}
