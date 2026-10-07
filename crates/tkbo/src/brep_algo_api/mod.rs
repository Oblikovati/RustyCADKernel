//! The BRep API of the Boolean operations: General Fuse, Boolean operations and section.
mod algo;
pub use algo::BRepAlgoAPIAlgo;

mod builder_algo;
pub use builder_algo::BRepAlgoAPIBuilderAlgo;

mod boolean_operation;
pub use boolean_operation::BRepAlgoAPIBooleanOperation;

mod operations;
pub use operations::{BRepAlgoAPICommon, BRepAlgoAPICut, BRepAlgoAPIFuse};

mod section;
pub use section::BRepAlgoAPISection;

mod check;
pub use check::BRepAlgoAPICheck;

mod splitter;
pub use splitter::BRepAlgoAPISplitter;
