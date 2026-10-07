//! Rust port of the OCCT TKCDF toolkit: the document and application framework of OCAF.
//!
//! - [`ldom`]: the light DOM that reads and writes XML.
//! - [`pcdm`]: persistent-document interfaces (readers, writers, drivers, reader filter).
//!
//! The design rules are in `docs/ocaf-design.md`.
pub mod cdf;
pub mod cdm;
pub mod json_dump;
pub mod ldom;
mod live;
pub mod pcdm;
pub use live::{SharedResourceManager, live};
