// Rust port of the type aliases of DataExchange/TKXSBase/XSAlgo/XSAlgo_ShapeProcessor.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter types of `XSAlgo_ShapeProcessor`, which the actors of the Transfer
//! package keep.
use rustycad_tkernel::{ncollection::DataMap, tcollection::AsciiString};
use rustycad_tkshhealing::shape_process::ShapeProcessOperationsFlags;

/// `XSAlgo_ShapeProcessor::ParameterMap`: parameter names to values.
pub type XSAlgoParameterMap = DataMap<AsciiString, AsciiString>;

/// `XSAlgo_ShapeProcessor::ProcessingData`: the parameters and the operations.
pub type XSAlgoProcessingData = (XSAlgoParameterMap, ShapeProcessOperationsFlags);

/// `XSAlgo_ShapeProcessor::ProcessingFlags`: the operations, and whether they were set.
pub type XSAlgoProcessingFlags = (ShapeProcessOperationsFlags, bool);
