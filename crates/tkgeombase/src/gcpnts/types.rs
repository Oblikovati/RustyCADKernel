// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_AbscissaType.hxx and
// GCPnts_DeflectionType.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve classifications that select a GCPnts algorithm.

/// How an abscissa search treats the curve (`GCPnts_AbscissaType`);
/// `AbscissaType::LengthParametrized as i32 == 0`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum AbscissaType {
    /// The length is proportional to the parameter (`GCPnts_LengthParametrized`).
    LengthParametrized,
    /// A general parametrization (`GCPnts_Parametrized`).
    Parametrized,
    /// Several continuity intervals (`GCPnts_AbsComposite`).
    AbsComposite,
}

/// How a deflection distribution treats the curve (`GCPnts_DeflectionType`);
/// `DeflectionType::Curved as i32 == 2`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum DeflectionType {
    /// A straight curve: its two ends (`GCPnts_Linear`).
    Linear,
    /// A circle: a constant angular step (`GCPnts_Circular`).
    Circular,
    /// A general curve (`GCPnts_Curved`).
    Curved,
    /// Several continuity intervals (`GCPnts_DefComposite`).
    DefComposite,
}
