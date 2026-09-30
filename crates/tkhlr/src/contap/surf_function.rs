// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_SurfFunction.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_SurfFunction: the contour function of `(u, v)` on a surface.
use super::types::ContapTFunction;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Direction2, Direction3, Point3, Vector3};
/// Contap_SurfFunction: `F(u, v)`: the normal of the surface dotted with the sight
/// direction (`ContourStd`), with the vector from the eye (`ContourPrs`), or minus the
/// draft cosine (`DraftStd`), divided by the mean normal length of the surface samples.
/// A zero of `F` is a contour point. The `DraftPrs` function is set but never
/// evaluated natively: its value and gradient are left unchanged.
///
/// A failure of a surface evaluation inside a solver callback is kept and raised when
/// the solver returns (docs/geomalgo-design.md section 6).
///
/// ```
/// use rustycad_tkhlr::contap::{ContapSurfFunction, ContapTFunction};
/// use rustycad_tkmath::gp::Direction3;
/// let mut f = ContapSurfFunction::new();
/// f.set_direction(Direction3::X);
/// f.set_tolerance(0.0);
/// assert_eq!(f.function_type(), ContapTFunction::ContourStd);
/// assert_eq!(f.tolerance(), 1e-12);
/// ```
pub struct ContapSurfFunction {
    pub(super) my_surf: Adaptor3dSurfaceHandle,
    pub(super) my_mean: f64,
    pub(super) my_type: ContapTFunction,
    pub(super) my_dir: Direction3,
    pub(super) my_eye: Point3,
    pub(super) my_ang: f64,
    pub(super) my_cos_ang: f64,
    pub(super) tol: f64,
    pub(super) solpt: Point3,
    pub(super) valf: f64,
    pub(super) usol: f64,
    pub(super) vsol: f64,
    pub(super) fpu: f64,
    pub(super) fpv: f64,
    pub(super) d2d: Direction2,
    pub(super) d3d: Vector3,
    pub(super) tangent: bool,
    pub(super) computed: bool,
    pub(super) derived: bool,
    pub(super) capture: FailureCapture,
}

impl Default for ContapSurfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ContapSurfFunction {
    fn clone(&self) -> Self {
        Self {
            my_surf: self.my_surf.clone(),
            capture: FailureCapture::default(),
            ..*self
        }
    }
}

impl ContapSurfFunction {
    /// Contap_SurfFunction(): the silhouette along +Z, mean 1, tolerance 1e-6, no
    /// surface; `ContapSurfFunction::new()`.
    pub fn new() -> Self {
        Self {
            my_surf: Adaptor3dSurfaceHandle::default(),
            my_mean: 1.,
            my_type: ContapTFunction::ContourStd,
            my_dir: Direction3::Z,
            my_eye: Point3::default(),
            my_ang: 0.0,
            my_cos_ang: 0.,
            tol: 1.0e-6,
            solpt: Point3::default(),
            valf: 0.0,
            usol: 0.0,
            vsol: 0.0,
            fpu: 0.0,
            fpv: 0.0,
            d2d: Direction2::default(),
            d3d: Vector3::default(),
            tangent: false,
            computed: false,
            derived: false,
            capture: FailureCapture::default(),
        }
    }
}

mod evaluation;
mod settings;
mod tangent;
