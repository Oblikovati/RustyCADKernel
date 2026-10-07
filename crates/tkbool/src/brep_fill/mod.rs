//! `BRepFill`: sweeps, pipes, thru-sections and fillings of the topological shapes.
mod generator;
mod thru_section_error_status;
mod transition_style;

pub use generator::{BRepFillGenerator, GeneratedFaces};
pub use thru_section_error_status::ThruSectionErrorStatus;
pub use transition_style::{TransitionStyle, TypeOfContact};
