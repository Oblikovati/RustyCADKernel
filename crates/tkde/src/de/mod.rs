//! The `DE` package: the classes every exchange format (STEP, IGES, STL, OBJ, glTF, VRML,
//! PLY) registers with to be found by file extension or content.
mod configuration_context;
mod configuration_line;

pub use configuration_context::{DeConfigurationContext, DeResourceMap};
