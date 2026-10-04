//! Face-connection geometry shared by fillet construction and offset analysis.
mod connection;
mod point_correction;
pub use connection::define_connect_type;
mod orientation;
pub use orientation::{next_side, next_side_on_face, same_side};
