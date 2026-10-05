//! Operating-system facilities exposed through Rust-owned interfaces.
//!
//! One module per OSD class family; each module re-exports its public items here so the
//! package reads as the flat native `OSD_*` namespace (`osd::Protection`, `osd::Timer`).
//!
//! Host-specific behavior follows the native build of the running platform (the Linux
//! branch is the verified profile); modules document every Windows/macOS difference.

// Foundation: enumerations, error reporting, protection and exception types.
mod enums;
pub use enums::*;
mod error;
pub use error::*;
mod exceptions;
pub use exceptions::*;
pub(crate) use exceptions::{OSD_EXCEPTION_HIERARCHY, OSD_OSD_ERROR_PARENT};
mod protection;
pub use protection::*;

// Paths.
mod path;
pub use path::*;

// File-system nodes: OSD_FileNode, OSD_File, OSD_Directory and their iterators.
mod file;
pub use file::*;

// Stream file systems: OSD_FileSystem and its local, cached and selector variants.
mod file_system;
pub use file_system::*;

// Process and host queries: OSD_Process, OSD_Host, OSD_Environment, OSD_SharedLibrary,
// OSD_MemInfo.
mod process;
pub use process::*;

// Timing, signals and the OSD package statics.
mod perf_meter;
pub use perf_meter::*;
mod timing;
pub use timing::*;

// Threads, the thread pool and parallel loops.
mod parallel;
pub use parallel::*;
