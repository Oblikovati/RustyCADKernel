// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_IType.hxx and Contap_TFunction.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two enumerations of Contap: the kinds of contour lines and of contour
//! functions.

/// Contap_IType: the geometry of a contour line; the discriminants are the native
/// enumerator values.
///
/// ```
/// use rustycad_tkhlr::contap::ContapIType;
/// assert_eq!(ContapIType::Restriction as i32, 3);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ContapIType {
    /// A straight line (`gp_Lin`).
    Lin = 0,
    /// A circle (`gp_Circ`).
    Circle = 1,
    /// A walking line (points on the surface).
    Walking = 2,
    /// A restriction arc of the domain that is solution.
    Restriction = 3,
}

/// Contap_TFunction: the contour function: the silhouette seen along a direction
/// (`ContourStd`) or from an eye (`ContourPrs`), and the draft line of a direction
/// (`DraftStd`) or of an eye (`DraftPrs`, set but not evaluated natively).
///
/// ```
/// use rustycad_tkhlr::contap::ContapTFunction;
/// assert_eq!(ContapTFunction::DraftPrs as i32, 3);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ContapTFunction {
    /// The silhouette along a direction.
    ContourStd = 0,
    /// The silhouette from an eye.
    ContourPrs = 1,
    /// The draft line along a direction.
    DraftStd = 2,
    /// The draft line from an eye.
    DraftPrs = 3,
}
