// Rust translation of ModelingAlgorithms/TKFillet/BRepBlend at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepBlend: the walking of the blend functions on B-rep faces (the instances of the
//! Blend generics on `Adaptor3d_Surface`, `Adaptor2d_Curve2d` restrictions and
//! `Adaptor3d_HVertex` vertices), the blend lines, the functions between a surface and a
//! restriction or two restrictions, and the approximation of the blend surfaces.
mod blend_tool;
mod curv_point_rad_inv;
mod extremity;
mod h_curve2d_tool;
mod h_curve_tool;
mod line;
mod point_on_rst;
mod rad_inv_macro;
mod rad_inv_terms;
mod rst_rst;
mod surf_curv_inv;
mod surf_point_inv;
pub(crate) mod surf_rst;

pub use blend_tool::BRepBlendBlendTool;
pub use curv_point_rad_inv::BRepBlendCurvPointRadInv;
pub use extremity::BRepBlendExtremity;
pub use h_curve_tool::BRepBlendHCurveTool;
pub use h_curve2d_tool::BRepBlendHCurve2dTool;
pub use line::{BRepBlendLine, BRepBlendLineHandle};
pub use point_on_rst::BRepBlendPointOnRst;
pub use rst_rst::{BRepBlendRstRstConstRad, BRepBlendRstRstEvolRad, RstRstInputs};
pub use surf_curv_inv::{BRepBlendSurfCurvConstRadInv, BRepBlendSurfCurvEvolRadInv};
pub use surf_point_inv::{BRepBlendSurfPointConstRadInv, BRepBlendSurfPointEvolRadInv};
pub use surf_rst::{BRepBlendSurfRstConstRad, BRepBlendSurfRstEvolRad, SurfRstInputs};

/// `BRepBlend_ChAsym`: the typedef of `BlendFunc_ChAsym`.
pub type BRepBlendChAsym = crate::blend_func::ChAsym;
/// `BRepBlend_ChAsymInv`: the typedef of `BlendFunc_ChAsymInv`.
pub type BRepBlendChAsymInv = crate::blend_func::ChAsymInv;
/// `BRepBlend_ChamfInv`: the typedef of `BlendFunc_ChamfInv`.
pub type BRepBlendChamfInv = crate::blend_func::ChamfInv;
/// `BRepBlend_Chamfer`: the typedef of `BlendFunc_Chamfer`.
pub type BRepBlendChamfer = crate::blend_func::Chamfer;
/// `BRepBlend_ConstRad`: the typedef of `BlendFunc_ConstRad`.
pub type BRepBlendConstRad = crate::blend_func::ConstRad;
/// `BRepBlend_ConstRadInv`: the typedef of `BlendFunc_ConstRadInv`.
pub type BRepBlendConstRadInv = crate::blend_func::ConstRadInv;
/// `BRepBlend_ConstThroat`: the typedef of `BlendFunc_ConstThroat`.
pub type BRepBlendConstThroat = crate::blend_func::ConstThroat;
/// `BRepBlend_ConstThroatInv`: the typedef of `BlendFunc_ConstThroatInv`.
pub type BRepBlendConstThroatInv = crate::blend_func::ConstThroatInv;
/// `BRepBlend_ConstThroatWithPenetration`: the typedef of
/// `BlendFunc_ConstThroatWithPenetration`.
pub type BRepBlendConstThroatWithPenetration = crate::blend_func::ConstThroatWithPenetration;
/// `BRepBlend_ConstThroatWithPenetrationInv`: the typedef of
/// `BlendFunc_ConstThroatWithPenetrationInv`.
pub type BRepBlendConstThroatWithPenetrationInv = crate::blend_func::ConstThroatWithPenetrationInv;
/// `BRepBlend_EvolRad`: the typedef of `BlendFunc_EvolRad`.
pub type BRepBlendEvolRad = crate::blend_func::EvolRad;
/// `BRepBlend_EvolRadInv`: the typedef of `BlendFunc_EvolRadInv`.
pub type BRepBlendEvolRadInv = crate::blend_func::EvolRadInv;

use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;

/// The curve behind a handle, or Standard_NullObject naming `class` (a null dereference
/// natively).
fn live_curve_of<'a>(
    curve: &'a Handle<dyn Adaptor3dCurve>,
    class: &str,
) -> Result<&'a dyn Adaptor3dCurve, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("{class}: null curve; expected an Adaptor3d_Curve"),
        )
    })
}

/// The 2d curve behind a handle, or Standard_NullObject naming `class`.
fn live_curve2d_of<'a>(
    curve: &'a Handle<dyn Adaptor2dCurve2d>,
    class: &str,
) -> Result<&'a dyn Adaptor2dCurve2d, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("{class}: null curve; expected an Adaptor2d_Curve2d"),
        )
    })
}
