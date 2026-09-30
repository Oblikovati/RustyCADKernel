// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SectionLaw.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_SectionLaw`: the section curve of a sweep as a function of the sweep
//! parameter, as the poles and weights of one B-spline shape.
use super::sweep_failures::{domain_error, not_implemented};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomCurve};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The section and its first derivatives (`D1` arguments `Poles`, `DPoles`,
/// `Weigths`, `DWeigths`).
pub struct SectionD1<'a, 'b> {
    /// `Poles`.
    pub poles: &'a mut Array1<'b, Point3>,
    /// `DPoles`.
    pub d_poles: &'a mut Array1<'b, Vector3>,
    /// `Weigths`.
    pub weights: &'a mut Array1<'b, f64>,
    /// `DWeigths`.
    pub d_weights: &'a mut Array1<'b, f64>,
}

/// The second derivatives added to [`SectionD1`] (`D2` arguments).
pub struct SectionD2<'a, 'b> {
    /// The value and first derivatives.
    pub d1: SectionD1<'a, 'b>,
    /// `D2Poles`.
    pub d2_poles: &'a mut Array1<'b, Vector3>,
    /// `D2Weigths`.
    pub d2_weights: &'a mut Array1<'b, f64>,
}

/// `GeomFill_SectionLaw`: the abstract section law, reached through
/// `Handle<dyn GeomFillSectionLaw>` (docs/geomalgo-design.md, section 14).
///
/// Every member takes `&self` (the laws keep their state behind locks). Members that
/// write an `Error` argument take it as `&mut f64`, since the native bodies may leave
/// it untouched; `law.d0(0.5, &mut poles, &mut weights)?`.
pub trait GeomFillSectionLaw: Transient {
    /// `D0(Param, Poles, Weigths)`: the section at `param`.
    fn d0(
        &self,
        param: f64,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure>;

    /// `D1(...)`; Standard_NotImplemented by default.
    fn d1(&self, param: f64, values: SectionD1<'_, '_>) -> Result<bool, Failure> {
        let _ = (param, values);
        Err(not_implemented("GeomFill_SectionLaw::D1"))
    }

    /// `D2(...)`; Standard_NotImplemented by default.
    fn d2(&self, param: f64, values: SectionD2<'_, '_>) -> Result<bool, Failure> {
        let _ = (param, values);
        Err(not_implemented("GeomFill_SectionLaw::D2"))
    }

    /// `BSplineSurface()`: the surface of the law when it has one; a null handle by
    /// default.
    fn bspline_surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        Ok(Handle::default())
    }

    /// `SectionShape(NbPoles, NbKnots, Degree)`.
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure>;

    /// `Knots(TKnots)`.
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure>;

    /// `Mults(TMults)`.
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure>;

    /// `IsRational()`.
    fn is_rational(&self) -> Result<bool, Failure>;

    /// `IsUPeriodic()`.
    fn is_u_periodic(&self) -> Result<bool, Failure>;

    /// `IsVPeriodic()`.
    fn is_v_periodic(&self) -> Result<bool, Failure>;

    /// `NbIntervals(S)`.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure>;

    /// `Intervals(T, S)`.
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure>;

    /// `SetInterval(First, Last)`.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure>;

    /// `GetInterval(First, Last)`.
    fn get_interval(&self) -> Result<(f64, f64), Failure>;

    /// `GetDomain(First, Last)`.
    fn get_domain(&self) -> Result<(f64, f64), Failure>;

    /// `GetTolerance(BoundTol, SurfTol, AngleTol, Tol3d)`.
    fn get_tolerance(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        angle_tol: f64,
        tol3d: &mut Array1<'_, f64>,
    ) -> Result<(), Failure>;

    /// `SetTolerance(Tol3d, Tol2d)`: nothing by default.
    fn set_tolerance(&self, tol3d: f64, tol2d: f64) -> Result<(), Failure> {
        let _ = (tol3d, tol2d);
        Ok(())
    }

    /// `BarycentreOfSurf()`; Standard_NotImplemented by default.
    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        Err(not_implemented("GeomFill_SectionLaw::BarycentreOfSurf"))
    }

    /// `MaximalSection()`.
    fn maximal_section(&self) -> Result<f64, Failure>;

    /// `GetMinimalWeight(Weigths)`; Standard_NotImplemented by default.
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let _ = weights;
        Err(not_implemented("GeomFill_SectionLaw::GetMinimalWeight"))
    }

    /// `IsConstant(Error)`: false with `Error = 0` by default.
    fn is_constant(&self, error: &mut f64) -> Result<bool, Failure> {
        *error = 0.0;
        Ok(false)
    }

    /// `ConstantSection()`; Standard_DomainError by default.
    fn constant_section(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Err(domain_error("GeomFill_SectionLaw::ConstantSection"))
    }

    /// `IsConicalLaw(Error)`: false with `Error = 0` by default.
    fn is_conical_law(&self, error: &mut f64) -> Result<bool, Failure> {
        *error = 0.0;
        Ok(false)
    }

    /// `CirclSection(Param)`; Standard_DomainError by default.
    fn circl_section(&self, param: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let _ = param;
        Err(domain_error("GeomFill_SectionLaw::CirclSection"))
    }
}
impl_transient_interface!(
    dyn GeomFillSectionLaw,
    "GeomFill_SectionLaw",
    StandardTransient
);
