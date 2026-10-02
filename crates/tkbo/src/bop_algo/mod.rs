//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod enums;
mod options;

pub use enums::{Glue, Operation};
pub use options::Options;

rustycad_tkernel::define_simple_alert!(AlertUserBreak, "BOPAlgo_AlertUserBreak");
