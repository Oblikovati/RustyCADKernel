//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod options;
mod progress;
pub use options::Options;
pub use progress::{Algo, PISteps};

mod alerts;
pub use alerts::*;
