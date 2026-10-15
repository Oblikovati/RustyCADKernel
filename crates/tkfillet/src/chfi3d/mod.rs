//! ChFi3d: the 3d fillet and chamfer builders (`ChFi3d_Builder`, `ChFi3d_FilBuilder`,
//! `ChFi3d_ChBuilder`), their helpers, and the face-connection geometry shared by fillet
//! construction and offset analysis (docs/tkfillet-progress.md).
pub(crate) mod builder;
mod builder0;
mod ch_builder;
mod fil_builder;
pub use builder::{
    BlendSide, ChFi3dBuilder, ChFi3dBuilderOps, RstCall, RstRef, SurfCall, SurfOut, WalkSettings,
};
pub use builder0::*;
pub use ch_builder::ChFi3dChBuilder;
pub use fil_builder::ChFi3dFilBuilder;
mod concave_side;
mod fillet_shape;
pub use fillet_shape::FilletShape;
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
