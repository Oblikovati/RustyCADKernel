// SPDX-License-Identifier: GPL-2.0-only
//! Martensite mesh appearance storage; rendering and the rest of TKMeshVS remain separate.
//!
//! ```
//! use martensite_tkmeshvs::{MeshVSDrawer, MeshVSDrawerSource};
//! use rustycad_tkernel::standard::Handle;
//! let source = Handle::new(MeshVSDrawer::default());
//! source.set_integer(7, 42);
//! let destination = MeshVSDrawer::default();
//! destination.assign(&source);
//! let mut output = 9;
//! assert!(!destination.get_integer(8, &mut output));
//! assert_eq!(output, 9);
//! assert!(destination.get_integer(7, &mut output));
//! assert_eq!(output, 42);
//! ```
pub mod drawer;
pub mod flags;
pub mod two_colors;
pub use drawer::{MeshVSDrawer, MeshVSDrawerSource, meshvs_drawer_assign};
pub use flags::{MeshVSDisplayModeFlags, MeshVSDrawerAttribute, MeshVSEntityType};
pub use two_colors::{
    MeshVSColorOutputs, MeshVSTwoColors, bind_two_colors, extract_color, extract_colors,
    extract_colors_into,
};
