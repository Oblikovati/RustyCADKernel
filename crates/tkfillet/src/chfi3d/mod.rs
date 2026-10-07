//! Face-connection geometry shared by fillet construction and offset analysis.
mod concave_side;
mod connection;
mod hatching;
mod search_sing;
pub use concave_side::{coefficient, concave_side};
pub use hatching::compute_hatching_domains;
pub use search_sing::ChFi3dSearchSing;
mod point_correction;
pub use connection::define_connect_type;
mod orientation;
pub use orientation::{next_side, next_side_on_face, same_side};

mod tangency;
pub use tangency::is_tangent_faces;
