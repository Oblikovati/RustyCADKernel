//! `BRepFill`: sweeps, pipes, thru-sections and fillings of the topological shapes.
mod compatible_wires;
mod generator;
mod location_law;
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
mod trim_shell_corner;

pub use compatible_wires::BRepFillCompatibleWires;
pub use generator::{BRepFillGenerator, GeneratedFaces};
pub use location_law::{
    BRepFillACRLaw, BRepFillDraftLaw, BRepFillEdge3DLaw, BRepFillEdgeOnSurfLaw,
    BRepFillLocationLaw, BRepFillLocationLawBase,
};
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
pub use trim_shell_corner::BRepFillTrimShellCorner;
