//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod enums;
mod options;
mod progress;

pub use enums::{Glue, Operation};
pub use options::Options;
pub use progress::{Algo, PISteps};

rustycad_tkernel::define_simple_alert!(AlertUserBreak, "BOPAlgo_AlertUserBreak");
