//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod enums;
mod options;
pub use enums::{Glue, Operation};
mod parallel;
mod progress;
mod section_attribute;
pub use options::Options;
pub use parallel::{ParallelAlgo, ParallelAlgoBase};
pub use progress::{Algo, PISteps};
pub use section_attribute::SectionAttribute;

mod alerts;
pub use alerts::*;
mod builder;
mod pave_filler;
pub mod tools;
mod tools_provider;
pub use builder::BOPAlgoBuilder;
pub use pave_filler::{BOPAlgoPaveFiller, EdgeRangeDistance};
pub use tools_provider::BOPAlgoToolsProvider;
mod builder_area;
mod builder_face;
pub use builder_area::BuilderArea;
pub use builder_face::BuilderFace;
mod wire_edge_set;
mod wire_splitter;
pub use wire_edge_set::WireEdgeSet;
pub use wire_splitter::{EdgeInfo, WireSplitter};
