//! `BRepFeat`: the form features on a basis shape: prisms, drafted prisms, revolutions,
//! pipes, linear and revolution forms (ribs and slots), cylindrical holes, gluing and
//! splitting of shapes.
mod builder;
mod common;
mod cylindrical_hole;
mod form;
mod gluer;
mod make_dprism;
mod make_linear_form;
mod make_pipe;
mod make_prism;
mod make_revol;
mod make_revolution_form;
mod package;
mod package_faces;
mod rib_slot;
mod split_shape;
pub use builder::BRepFeatBuilder;
pub use cylindrical_hole::BRepFeatMakeCylindricalHole;
pub use form::{DescendantMap, Form, FormFeature};
pub use gluer::BRepFeatGluer;
pub use make_dprism::BRepFeatMakeDPrism;
pub use make_linear_form::BRepFeatMakeLinearForm;
pub use make_pipe::BRepFeatMakePipe;
pub use make_prism::BRepFeatMakePrism;
pub use make_revol::BRepFeatMakeRevol;
pub use make_revolution_form::BRepFeatMakeRevolutionForm;
pub use package::{
    ParametricMinMax, PerfSelection, Status, StatusError, barycenter, parametric_barycenter,
    parametric_min_max, print, sample_edges, status_error_message,
};
pub use package_faces::{face_until, is_inside, tool};
pub use rib_slot::{Extremes, ProfileInput, RibSlot, choice_of_faces, int_par};
pub use split_shape::BRepFeatSplitShape;
