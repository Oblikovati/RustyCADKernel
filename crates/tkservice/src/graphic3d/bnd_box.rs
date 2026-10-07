//! `Graphic3d_BndBox3d`, `Graphic3d_BndBox4d` and `Graphic3d_BndBox4f`: the BVH boxes the
//! presentation structures and groups keep their bounds in.
use rustycad_tkmath::bvh::{BvhBox, BvhVec3d, BvhVec4d, BvhVec4f};

/// `Graphic3d_BndBox3d` (`BVH_Box<double, 3>`); `Graphic3dBndBox3d::default().is_valid() == false`.
pub type Graphic3dBndBox3d = BvhBox<BvhVec3d>;
/// `Graphic3d_BndBox4d` (`BVH_Box<double, 4>`); `Graphic3dBndBox4d::default()`.
pub type Graphic3dBndBox4d = BvhBox<BvhVec4d>;
/// `Graphic3d_BndBox4f` (`BVH_Box<float, 4>`); `Graphic3dBndBox4f::default()`.
pub type Graphic3dBndBox4f = BvhBox<BvhVec4f>;
