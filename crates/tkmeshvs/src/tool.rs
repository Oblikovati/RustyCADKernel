//! MeshVS_Tool preserves native aspect factories and polygon-normal formulas.
//!
//! ```
//! use martensite_tkmeshvs::{MeshVSDrawer,MeshVSTool};
//! use rustycad_tkernel::standard::Handle;
//! let drawer=Handle::new(MeshVSDrawer::default());
//! let line=MeshVSTool::create_aspect_line3d(&drawer,true)?;
//! assert!(!line.is_null());
//! # Ok::<(),rustycad_tkernel::standard::Failure>(())
//! ```
mod normals;
mod simple;
mod values;
/// Native mesh presentation helpers; `MeshVSTool::get_normal(&nodes, &mut normal)?`.
pub struct MeshVSTool;

mod fill;
pub(crate) mod text;
