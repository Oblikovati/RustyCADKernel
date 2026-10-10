//! The Prs3d package: the aspects and the drawer of presentations, the presentation tools
//! (arrows, texts, boxes, quadrics) and the package helpers.
//!
//! `Prs3d_Presentation` is a typedef of `Graphic3d_Structure`: the presentations are
//! `Handle<dyn Graphic3dStructureSource>` (v3d-design.md section 1).
mod arrow;
mod arrow_aspect;
mod basic_aspect;
mod bnd_box;
mod datum_aspect;
mod dimension_aspect;
mod drawer;
mod drawer_aspects;
mod drawer_dump;
mod drawer_setup;
mod drawer_values;
mod enums;
mod line_aspect;
mod package;
mod plane_aspect;
mod point_aspect;
mod presentation_shadow;
mod root;
mod shading_aspect;
mod text;
mod text_aspect;
mod tool_quadric;
mod tools;

pub use arrow::Prs3dArrow;
pub use arrow_aspect::Prs3dArrowAspect;
pub use basic_aspect::{Prs3dBasicAspect, prs3d_copy_aspects};
pub use bnd_box::Prs3dBndBox;
pub use datum_aspect::Prs3dDatumAspect;
pub use dimension_aspect::{Prs3dDimensionAspect, Prs3dDimensionUnits};
pub use drawer::{Prs3dDrawer, Prs3dDrawerFields};
pub use enums::*;
pub use line_aspect::{Prs3dIsoAspect, Prs3dLineAspect};
pub use package::Prs3d;
pub use plane_aspect::{Prs3dPlaneAspect, Prs3dPlaneSettings};
pub use point_aspect::Prs3dPointAspect;
pub use presentation_shadow::Prs3dPresentationShadow;
pub use root::{Prs3dPoint, Prs3dPointTool, Prs3dRoot};
pub use shading_aspect::Prs3dShadingAspect;
pub use text::Prs3dText;
pub use text_aspect::Prs3dTextAspect;
pub use tool_quadric::{Prs3dToolQuadric, prs3d_quadric_triangles_nb, prs3d_quadric_vertices_nb};
pub use tools::{
    Prs3dToolCylinder, Prs3dToolDisk, Prs3dToolSector, Prs3dToolSphere, Prs3dToolTorus,
};

use rustycad_tkernel::standard::Failure;

/// `Prs3d_InvalidAngle` (a `Standard_RangeError`), raised by `Prs3d_ArrowAspect::SetAngle`.
/// Example: `prs3d_invalid_angle("angle 2 out of range")`.
pub fn prs3d_invalid_angle(message: &str) -> Failure {
    Failure::with_type("Prs3d_InvalidAngle", message, "")
}
