//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod enums;
mod options;
pub use enums::{Glue, Operation};
mod parallel;
mod progress;
pub use options::Options;
pub use parallel::{ParallelAlgo, ParallelAlgoBase};
pub use progress::{Algo, PISteps};

mod alerts;
pub use alerts::*;
mod builder;
mod tools_provider;
pub use builder::BOPAlgoBuilder;
pub use tools_provider::BOPAlgoToolsProvider;
