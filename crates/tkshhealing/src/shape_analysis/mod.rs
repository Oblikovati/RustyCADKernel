// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAnalysis at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeAnalysis: the analysis tools that detect problems of shapes (wires,
//! edges, surfaces, free bounds, contents). docs/shhealing-design.md section 4 fixes how
//! its classes map; docs/tkshhealing-progress.md lists the ported classes.
mod edge;
mod free_bound_data;
mod geom;
mod shape_tolerance;
mod shell;
mod wire_order;

pub use free_bound_data::ShapeAnalysisFreeBoundData;
pub use geom::ShapeAnalysisGeom;
pub use shape_tolerance::ShapeAnalysisShapeTolerance;
pub use shell::ShapeAnalysisShell;
pub use wire_order::ShapeAnalysisWireOrder;

pub use edge::{Curve3dRange, EdgeTangent2d, PCurveRange, ShapeAnalysisEdge};

mod canonical_recognition;
pub use canonical_recognition::ShapeAnalysisCanonicalRecognition;

mod surface;
pub use surface::{ShapeAnalysisSurface, SurfaceSingularity};
mod curve;
pub use curve::{
    CurveProjection, CurveProjectionResult, ShapeAnalysisCurve, adjust_by_period, adjust_to_period,
};
mod wire;
pub use wire::ShapeAnalysisWire;

mod package;
pub use package::ShapeAnalysis;

mod free_bounds;
pub use free_bounds::ShapeAnalysisFreeBounds;

mod transfer_parameters;
pub use transfer_parameters::{
    ShapeAnalysisTransferParameters, TransferParameterData, TransferParameterFields,
    TransferParameters,
};
mod transfer_parameters_proj;
pub use transfer_parameters_proj::ShapeAnalysisTransferParametersProj;

mod wire_vertex;
pub use wire_vertex::ShapeAnalysisWireVertex;
