//! `BRepFeat`: the form features on a basis shape: prisms, drafted prisms, revolutions,
//! pipes, linear and revolution forms (ribs and slots), cylindrical holes, gluing and
//! splitting of shapes.
mod builder;
mod common;
mod form;
mod gluer;
mod make_dprism;
mod make_pipe;
mod make_prism;
mod make_revol;
mod package;
mod split_shape;
pub use builder::BRepFeatBuilder;
pub use form::{DescendantMap, Form, FormFeature};
pub use gluer::BRepFeatGluer;
pub use make_dprism::BRepFeatMakeDPrism;
pub use make_pipe::BRepFeatMakePipe;
pub use make_prism::BRepFeatMakePrism;
pub use make_revol::BRepFeatMakeRevol;
pub use package::{
    ParametricMinMax, PerfSelection, Status, StatusError, barycenter, face_until, is_inside,
    parametric_barycenter, parametric_min_max, print, sample_edges, status_error_message, tool,
};
pub use split_shape::BRepFeatSplitShape;
