// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_FillingStyle.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_FillingStyle`: the filling scheme of `GeomFill_BezierCurves` and
//! `GeomFill_BSplineCurves`.

/// The filling scheme (`GeomFill_FillingStyle`), with the native values;
/// `GeomFillFillingStyle::CoonsStyle as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GeomFillFillingStyle {
    /// `GeomFill_StretchStyle`: the flattest surface (`GeomFill_Stretch`).
    #[default]
    StretchStyle = 0,
    /// `GeomFill_CoonsStyle`: a rounded surface (`GeomFill_Coons`).
    CoonsStyle = 1,
    /// `GeomFill_CurvedStyle`: the most rounded surface (`GeomFill_Curved`).
    CurvedStyle = 2,
}
