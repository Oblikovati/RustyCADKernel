//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod options;
mod progress;
pub use options::Options;
pub use progress::{Algo, PISteps};

rustycad_tkernel::define_simple_alert!(AlertUserBreak, "BOPAlgo_AlertUserBreak");
