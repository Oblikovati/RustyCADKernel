// Rust port of DataExchange/TKXSBase/LibCtl/LibCtl_Library.gxx and the node classes it
// generates (Interface_GlobalNodeOfGeneralLib_0.cxx, Interface_NodeOfGeneralLib_0.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LibCtl`: the generic library of (module, protocol) pairs that dispatches an object to
//! the module of the protocol that recognises it (xsbase-design.md section 2).
//!
//! The template is instantiated natively per library (`Interface_GeneralLib`,
//! `Interface_ReaderLib`, and in the format toolkits `StepData_WriterLib`...). Here one
//! generic [`LibCtlLibrary`] serves all of them: a [`LibCtlKind`] names the module and
//! protocol types of an instance and its process-wide [`LibCtlRegistry`] (the native
//! static `THE_GLOBAL`). The `*_in` members take an explicit registry instead.
//!
//! The generated node classes (`*_GlobalNodeOf*Lib`, `*_NodeOf*Lib`) are the generic
//! [`LibCtlGlobalNode`] and [`LibCtlNode`], shared through `Arc` (natively handles).
mod library;
mod nodes;

pub use library::{LibCtlKind, LibCtlLibrary, LibCtlProtocol, LibCtlRegistry};
pub use nodes::{LibCtlGlobalNode, LibCtlNode};
