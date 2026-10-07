//! `BRepAlgo`: the shape images and ascendants/descendants of the Boolean operations,
//! and the package functions on wires, faces and validity.
mod as_des;
mod image;
mod normal_projection;
mod package;
pub use as_des::BRepAlgoAsDes;
pub use image::BRepAlgoImage;
pub use normal_projection::BRepAlgoNormalProjection;
pub use package::{
    concatenate_wire, concatenate_wire_c0, convert_face, convert_wire, is_topologically_valid,
    is_valid, is_valid_result,
};
