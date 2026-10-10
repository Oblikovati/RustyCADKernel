//! Shared polyline base state, including sensitive curves and user-derived polylines.
use super::{Select3DSensitivePolyCore, Select3DSensitiveSet};
/// The native SensitivePoly base of a sensitive set.
/// Example: `entity.cast::<dyn Select3DSensitivePolySource>().sensitive_poly_core()`.
pub trait Select3DSensitivePolySource: Select3DSensitiveSet {
    /// Borrow the native polyline base state; `poly.sensitive_poly_core().array_bounds()`.
    fn sensitive_poly_core(&self) -> &Select3DSensitivePolyCore;
}

rustycad_tkernel::impl_transient_interface!(
    dyn Select3DSensitivePolySource,
    "Select3D_SensitivePoly",
    dyn Select3DSensitiveSet
);
