// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntWalk at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntWalk: the marching algorithms of the surface intersections.
//!
//! `IntWalk_PWalking` marches along the intersection of two parametric surfaces from
//! a start point; the template `IntWalk_IWalking.gxx` marches along the zero set of an
//! implicit function on a parametric surface. The template is the generic
//! [`IntWalkIWalking`] (docs/geomalgo-design.md, "IntWalk templates"), and the two
//! IntImp instances of the package are aliases of the IntImp generics.
mod iwalking;
mod pwalking;

pub use iwalking::{IWalkingTypes, IntWalkIWalking, TheIWFunction, TheIWLine};
pub use pwalking::IntWalkPWalking;

use crate::int_imp::{HSurfaceTool, IntImpInt2S, IntImpZerParFunc};

/// `IntWalk_TheInt2S`: `IntImp_Int2S.gxx` on `Adaptor3d_HSurfaceTool`.
///
/// `IntWalkTheInt2S::new(&s1, &s2, 1e-6)?`.
pub type IntWalkTheInt2S = IntImpInt2S<HSurfaceTool>;

/// `IntWalk_TheFunctionOfTheInt2S`: `IntImp_ZerParFunc.gxx` on `Adaptor3d_HSurfaceTool`.
///
/// `IntWalkTheFunctionOfTheInt2S::new(&s1, &s2)?`.
pub type IntWalkTheFunctionOfTheInt2S = IntImpZerParFunc<HSurfaceTool>;

/// `IntWalk_StatusDeflection`: the verdict of a deflection test on a new point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IntWalkStatusDeflection {
    /// The step is too great: divide it.
    PasTropGrand,
    /// The step is too small: increase it.
    StepTooSmall,
    /// The point is confused with the previous one.
    PointConfondu,
    /// Stop on the previous point.
    ArretSurPointPrecedent,
    /// Stop on the point.
    ArretSurPoint,
    /// The point is accepted.
    #[default]
    OK,
}

/// `IntWalk_WalkingData`: a start point of the implicit walking and its state
/// (`etat`, negative once processed).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IntWalkWalkingData {
    /// The start U parameter.
    pub ustart: f64,
    /// The start V parameter.
    pub vstart: f64,
    /// The state code of the point (see IntWalk_IWalking.gxx).
    pub etat: i32,
}

/// `IntWalk_VectorOfWalkingData` (`NCollection_LinearVector<IntWalk_WalkingData>`),
/// 0-based as natively.
pub type IntWalkVectorOfWalkingData = Vec<IntWalkWalkingData>;

/// `IntWalk_VectorOfInteger` (`NCollection_LinearVector<int>`), 0-based as natively.
pub type IntWalkVectorOfInteger = Vec<i32>;
