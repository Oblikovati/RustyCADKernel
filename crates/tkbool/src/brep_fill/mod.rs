//! `BRepFill`: sweeps, pipes, thru-sections and fillings of the topological shapes.
mod advanced_evolved;
mod approx_seewing;
mod compatible_wires;
mod draft;
mod filling;
mod generator;
mod location_law;
mod multi_line;
mod offset_ancestors;
mod offset_wire;
mod package;
mod pipe;
mod pipe_shell;
mod section;
mod section_law;
mod section_placement;
mod shape_table;
mod sweep;
mod thru_section_error_status;
mod transition_style;
mod trim_edge_tool;
mod trim_shell_corner;
mod trim_surface_tool;

pub use advanced_evolved::{AdvancedEvolvedStatus, BRepFillAdvancedEvolved};
pub use approx_seewing::BRepFillApproxSeewing;
pub use compatible_wires::BRepFillCompatibleWires;
pub use draft::BRepFillDraft;
pub use filling::{
    BRepFillCurveConstraint, BRepFillEdgeFaceAndOrder, BRepFillFaceAndOrder, BRepFillFilling,
};
pub use generator::{BRepFillGenerator, GeneratedFaces};
pub use location_law::{
    BRepFillACRLaw, BRepFillDraftLaw, BRepFillEdge3DLaw, BRepFillEdgeOnSurfLaw,
    BRepFillLocationLaw, BRepFillLocationLawBase,
};
pub use multi_line::{BRepFillMultiLine, MultiLineCurves};
pub use offset_ancestors::BRepFillOffsetAncestors;
pub use offset_wire::BRepFillOffsetWire;
pub use package::{axe, compute_acr, insert_acr, ruled_face, ruled_shell, search_origin};
pub use pipe::BRepFillPipe;
pub use pipe_shell::BRepFillPipeShell;
pub use section::BRepFillSection;
pub use section_law::{
    BRepFillNSections, BRepFillSectionLaw, BRepFillSectionLawBase, BRepFillShapeLaw,
};
pub use section_placement::BRepFillSectionPlacement;
pub use sweep::{BRepFillSweep, SweepBuildOptions, SweepHistory, SweepTablesMap};
pub use thru_section_error_status::ThruSectionErrorStatus;
pub use transition_style::{TransitionStyle, TypeOfContact};
pub use trim_edge_tool::BRepFillTrimEdgeTool;
pub use trim_shell_corner::BRepFillTrimShellCorner;
pub use trim_surface_tool::{BRepFillTrimSurfaceTool, ProjectedCurves};

/// `BRepFill_ComputeCLine`: the Approx_ComputeCLine instantiation over
/// AppCont_Function, fitting a [`BRepFillMultiLine`];
/// `BRepFillComputeCLine::with_line(&multi_line, ComputeCLineOptions::default())?`.
pub type BRepFillComputeCLine = rustycad_tkgeombase::approx::ComputeCLine;

mod evolved;
pub use evolved::BRepFillEvolved;
