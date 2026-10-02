//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod options;
mod enums;

pub use options::Options;
pub use enums::{Glue, Operation};

rustycad_tkernel::define_simple_alert!(AlertUserBreak, "BOPAlgo_AlertUserBreak");
