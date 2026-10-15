//! `BRepFeat`: the form features on a basis shape: prisms, drafted prisms, revolutions,
//! pipes, linear and revolution forms (ribs and slots), cylindrical holes, gluing and
//! splitting of shapes.
mod builder;
mod common;
mod form;
mod make_prism;
mod package;
pub use builder::BRepFeatBuilder;
pub use form::{DescendantMap, Form, FormFeature};
pub use make_prism::BRepFeatMakePrism;
pub use package::{
    ParametricMinMax, PerfSelection, Status, StatusError, barycenter, face_until, is_inside,
    parametric_barycenter, parametric_min_max, print, sample_edges, status_error_message, tool,
};
