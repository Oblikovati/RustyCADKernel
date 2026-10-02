//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod enums;
mod options;
pub use enums::{Glue, Operation};
mod progress;
pub use options::Options;
pub use progress::{Algo, PISteps};

mod alerts;
pub use alerts::*;
