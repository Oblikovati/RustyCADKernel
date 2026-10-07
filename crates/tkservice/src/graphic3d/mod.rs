//! Graphic3d: the 3D presentation data model and the graphic driver interfaces (OCCT package
//! Graphic3d).
//!
//! Each class keeps its package prefix (`Graphic3d_Camera` is `Graphic3dCamera`); abstract
//! driver classes are traits over embedded bases (visualization-design.md section 1).
mod hatch_style;
mod material_enums;
mod render_enums;
mod small_values;
mod stats_enums;
mod texture_enums;

pub use hatch_style::Graphic3dHatchStyle;
pub use material_enums::*;
pub use render_enums::*;
pub use small_values::{Graphic3dBufferRange, Graphic3dPolygonOffset, Graphic3dVertex};
pub use stats_enums::*;
pub use texture_enums::*;
