// IntTools_Tools.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Geometric operations shared by intersection algorithms.
/// Native intersection utilities; `Tools::intermediate_point(0.0, 1.0)`.
pub struct Tools;
impl Tools {
    /// Picks the native asymmetric interior parameter; `Tools::intermediate_point(0.0, 1.0)`.
    pub fn intermediate_point(first: f64, last: f64) -> f64 {
        // The rounded native constant, not a fresh evaluation of 10 * exp(-pi).
        const PAR_T: f64 = 0.43213918;
        (1.0 - PAR_T) * first + PAR_T * last
    }
}
mod curves;
mod ranges;
mod segment_plane;
mod splitting;
mod surface_tolerance;
mod tolerance;
mod topology;
