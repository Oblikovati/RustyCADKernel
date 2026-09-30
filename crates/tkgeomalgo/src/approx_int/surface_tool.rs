//! The `ThePSurfaceTool` calls of the ApproxInt templates that the IntImp trait does
//! not hold (docs/geomalgo-design.md section 20).
use crate::int_imp::{HSurfaceTool, ThePSurfaceTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Sphere3};

/// `ThePSurfaceTool` of ApproxInt: the IntImp tool plus the periods, the surface kind and
/// the quadric accessors that `ApproxInt_ImpPrmSvSurfaces` and `ApproxInt_Approx` call.
///
/// ```
/// use rustycad_tkgeomalgo::approx_int::ApproxIntSurfaceTool;
/// use rustycad_tkgeomalgo::int_imp::HSurfaceTool;
/// fn kind<T: ApproxIntSurfaceTool>(s: &T::Surface) -> bool {
///     T::is_u_periodic(s).unwrap_or(false)
/// }
/// # let _ = kind::<HSurfaceTool>;
/// ```
pub trait ApproxIntSurfaceTool: ThePSurfaceTool {
    /// `IsUPeriodic(S)`.
    fn is_u_periodic(surface: &Self::Surface) -> Result<bool, Failure>;
    /// `IsVPeriodic(S)`.
    fn is_v_periodic(surface: &Self::Surface) -> Result<bool, Failure>;
    /// `UPeriod(S)`.
    fn u_period(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `VPeriod(S)`.
    fn v_period(surface: &Self::Surface) -> Result<f64, Failure>;
    /// `GetType(S)`.
    fn get_type(surface: &Self::Surface) -> Result<SurfaceType, Failure>;
    /// `Plane(S)`.
    fn plane(surface: &Self::Surface) -> Result<Plane3, Failure>;
    /// `Cylinder(S)`.
    fn cylinder(surface: &Self::Surface) -> Result<Cylinder3, Failure>;
    /// `Sphere(S)`.
    fn sphere(surface: &Self::Surface) -> Result<Sphere3, Failure>;
    /// `Cone(S)`.
    fn cone(surface: &Self::Surface) -> Result<Cone3, Failure>;
}

impl ApproxIntSurfaceTool for HSurfaceTool {
    fn is_u_periodic(surface: &Self::Surface) -> Result<bool, Failure> {
        h_surface_tool::is_u_periodic(surface)
    }
    fn is_v_periodic(surface: &Self::Surface) -> Result<bool, Failure> {
        h_surface_tool::is_v_periodic(surface)
    }
    fn u_period(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::u_period(surface)
    }
    fn v_period(surface: &Self::Surface) -> Result<f64, Failure> {
        h_surface_tool::v_period(surface)
    }
    fn get_type(surface: &Self::Surface) -> Result<SurfaceType, Failure> {
        h_surface_tool::get_type(surface)
    }
    fn plane(surface: &Self::Surface) -> Result<Plane3, Failure> {
        h_surface_tool::plane(surface)
    }
    fn cylinder(surface: &Self::Surface) -> Result<Cylinder3, Failure> {
        h_surface_tool::cylinder(surface)
    }
    fn sphere(surface: &Self::Surface) -> Result<Sphere3, Failure> {
        h_surface_tool::sphere(surface)
    }
    fn cone(surface: &Self::Surface) -> Result<Cone3, Failure> {
        h_surface_tool::cone(surface)
    }
}
