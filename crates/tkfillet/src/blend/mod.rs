// Rust translation of ModelingAlgorithms/TKFillet/Blend at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Blend: the abstract blend functions walked and approximated by BRepBlend, the points
//! of a blend line and the walking statuses.
//!
//! The abstract classes are traits over [`rustycad_tkmath::math::FunctionSetWithDerivatives`].
//! A member body that an abstract class defines over a pure member of its base (for
//! example `Blend_Function::Pnt1`) is a free function the implementations call.
mod app_function;
pub mod debug;
mod enums;
mod func_inv;
mod function;
mod point;

pub use app_function::{BlendAppFunction, SectionPoles};
pub use enums::{DecrochStatus, Status};
pub use func_inv::{
    BlendCurvPointFuncInv, BlendFuncInv, BlendInverseBounds, BlendSurfCurvFuncInv,
    BlendSurfPointFuncInv, curv_point_func_inv_nb_variables, func_inv_nb_variables,
    surf_curv_func_inv_nb_variables, surf_point_func_inv_nb_variables,
};
pub use function::{
    BlendFunction, BlendRstRstFunction, BlendSurfRstFunction, SectionEndFrames,
    function_nb_variables, function_pnt1, function_pnt2, function_section_d2,
    rst_rst_minimal_distance, rst_rst_pnt1, rst_rst_pnt2, surf_rst_minimal_distance, surf_rst_pnt1,
    surf_rst_pnt2,
};
pub use point::{BlendPoint, CurveSectionTangents, SectionTangents};
