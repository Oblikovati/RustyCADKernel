//! Boolean algorithm infrastructure from pinned OCCT TKBO.
mod options;
pub use options::Options;

rustycad_tkernel::define_simple_alert!(AlertUserBreak, "BOPAlgo_AlertUserBreak");
