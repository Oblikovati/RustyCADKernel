// Rust translation of the point-curve templates and aliases of ModelingData/TKGeomBase/
// Extrema (Extrema_GFuncExtPC, Extrema_GGenExtPC, Extrema_GenLocateExtPC,
// Extrema_GGExtPC, Extrema_GLocateExtPC, Extrema_GCurveLocator and their
// Extrema_ExtPC*, Extrema_LocateExtPC*, Extrema_EPCOf*, Extrema_ELPCOf*,
// Extrema_LocEPCOf*, Extrema_PCFOf* and Extrema_PCLocFOf* aliases) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The point-curve extrema: the class templates, generic over [`ExtremaCurveTool`],
//! and the native aliases of their 3D and 2D instantiations.
mod curve_locator;
mod ext_pc;
mod func_ext_pc;
mod gen_ext_pc;
mod gen_locate_ext_pc;
mod locate_ext_pc;

pub use curve_locator::GCurveLocator;
pub use ext_pc::GGExtPC;
pub use func_ext_pc::GFuncExtPC;
pub use gen_ext_pc::GGenExtPC;
pub use gen_locate_ext_pc::GenLocateExtPC;
pub use locate_ext_pc::GLocateExtPC;

use super::curve_tools::{Curve2dTool, CurveTool};

/// `Extrema_ExtPC`: `Extrema_GGExtPC` over Adaptor3d_Curve.
pub type ExtPC<'a> = GGExtPC<'a, CurveTool>;
/// `Extrema_ExtPC2d`: `Extrema_GGExtPC` over Adaptor2d_Curve2d.
pub type ExtPC2d<'a> = GGExtPC<'a, Curve2dTool>;
/// `Extrema_ELPCOfLocateExtPC`: the closed-form search of Extrema_LocateExtPC.
pub type ELPCOfLocateExtPC<'a> = GGExtPC<'a, CurveTool>;
/// `Extrema_ELPCOfLocateExtPC2d`.
pub type ELPCOfLocateExtPC2d<'a> = GGExtPC<'a, Curve2dTool>;
/// `Extrema_EPCOfExtPC`: `Extrema_GGenExtPC` over Adaptor3d_Curve.
pub type EPCOfExtPC<'a> = GGenExtPC<'a, CurveTool>;
/// `Extrema_EPCOfExtPC2d`.
pub type EPCOfExtPC2d<'a> = GGenExtPC<'a, Curve2dTool>;
/// `Extrema_EPCOfELPCOfLocateExtPC`.
pub type EPCOfELPCOfLocateExtPC<'a> = GGenExtPC<'a, CurveTool>;
/// `Extrema_EPCOfELPCOfLocateExtPC2d`.
pub type EPCOfELPCOfLocateExtPC2d<'a> = GGenExtPC<'a, Curve2dTool>;
/// `Extrema_PCFOfEPCOfExtPC`: `Extrema_GFuncExtPC` over Adaptor3d_Curve.
pub type PCFOfEPCOfExtPC<'a> = GFuncExtPC<'a, CurveTool>;
/// `Extrema_PCFOfEPCOfExtPC2d`.
pub type PCFOfEPCOfExtPC2d<'a> = GFuncExtPC<'a, Curve2dTool>;
/// `Extrema_PCFOfEPCOfELPCOfLocateExtPC`.
pub type PCFOfEPCOfELPCOfLocateExtPC<'a> = GFuncExtPC<'a, CurveTool>;
/// `Extrema_PCFOfEPCOfELPCOfLocateExtPC2d`.
pub type PCFOfEPCOfELPCOfLocateExtPC2d<'a> = GFuncExtPC<'a, Curve2dTool>;
/// `Extrema_PCLocFOfLocEPCOfLocateExtPC`.
pub type PCLocFOfLocEPCOfLocateExtPC<'a> = GFuncExtPC<'a, CurveTool>;
/// `Extrema_PCLocFOfLocEPCOfLocateExtPC2d`.
pub type PCLocFOfLocEPCOfLocateExtPC2d<'a> = GFuncExtPC<'a, Curve2dTool>;
/// `Extrema_LocEPCOfLocateExtPC`: `Extrema_GenLocateExtPC` over Adaptor3d_Curve.
pub type LocEPCOfLocateExtPC<'a> = GenLocateExtPC<'a, CurveTool>;
/// `Extrema_LocEPCOfLocateExtPC2d`.
pub type LocEPCOfLocateExtPC2d<'a> = GenLocateExtPC<'a, Curve2dTool>;
/// `Extrema_LocateExtPC`: `Extrema_GLocateExtPC` over Adaptor3d_Curve.
pub type LocateExtPC<'a> = GLocateExtPC<'a, CurveTool>;
/// `Extrema_LocateExtPC2d`.
pub type LocateExtPC2d<'a> = GLocateExtPC<'a, Curve2dTool>;
