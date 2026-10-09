//! The `BRepFill` package class: ruled faces and shells, the frame of a profile on a
//! planar spine, the origin of a closed wire and its reduced curvilinear abscissas.
mod acr;
mod axe;
mod face_shell;
mod origin;
mod ruled;

pub use acr::{compute_acr, insert_acr};
pub use axe::axe;
pub use face_shell::{ruled_face, ruled_shell};
pub use origin::search_origin;
