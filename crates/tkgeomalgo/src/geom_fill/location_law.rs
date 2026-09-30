// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_LocationLaw.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_LocationLaw`: the placement `M(t), V(t)` of a sweep section along its
//! path, with optional 2d curves (the traces and restrictions of the sweep).
use super::pipe_error::GeomFillPipeError;
use super::sweep_failures::not_implemented;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Matrix3, Point2, Point3, Vector2, Vector3};

/// The placement a location law evaluates (`D0` outputs `M`, `V`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocationD0 {
    /// `M`: the rotation part.
    pub m: Matrix3,
    /// `V`: the translation part.
    pub v: Vector3,
}

/// The placement and its first derivatives (`D1` outputs `M`, `V`, `DM`, `DV`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocationD1 {
    /// `M`.
    pub m: Matrix3,
    /// `V`.
    pub v: Vector3,
    /// `DM`.
    pub dm: Matrix3,
    /// `DV`.
    pub dv: Vector3,
}

/// The placement and its first two derivatives (`D2` outputs).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocationD2 {
    /// `M`.
    pub m: Matrix3,
    /// `V`.
    pub v: Vector3,
    /// `DM`.
    pub dm: Matrix3,
    /// `DV`.
    pub dv: Vector3,
    /// `D2M`.
    pub d2m: Matrix3,
    /// `D2V`.
    pub d2v: Vector3,
}

/// The 2d poles of a location law and their two derivatives (`Poles2d`,
/// `DPoles2d`, `D2Poles2d` of `D2`).
pub struct LocationPoles2d<'a, 'b> {
    /// `Poles2d`.
    pub poles2d: &'a mut Array1<'b, Point2>,
    /// `DPoles2d`.
    pub d_poles2d: &'a mut Array1<'b, Vector2>,
    /// `D2Poles2d`.
    pub d2_poles2d: &'a mut Array1<'b, Vector2>,
}

/// `GeomFill_LocationLaw`: the abstract location law, reached through
/// `Handle<dyn GeomFillLocationLaw>` (docs/geomalgo-design.md, section 14).
///
/// Every member takes `&self` (the laws keep their state behind locks); the members
/// the base class implements are provided methods with the native bodies.
/// `law.d0(0.5, &mut placement)?`.
pub trait GeomFillLocationLaw: Transient {
    /// `SetCurve(C)`: loads the path; false when the law cannot use it.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure>;

    /// `GetCurve()`: the path.
    fn get_curve(&self) -> Handle<dyn Adaptor3dCurve>;

    /// `SetTrsf(Transfo)`: a rotation applied after the law.
    fn set_trsf(&self, transfo: &Matrix3) -> Result<(), Failure>;

    /// `Copy()`: an independent law of the same kind.
    fn copy(&self) -> Result<Handle<dyn GeomFillLocationLaw>, Failure>;

    /// `D0(Param, M, V)`: the placement at `param`.
    fn d0(&self, param: f64, placement: &mut LocationD0) -> Result<bool, Failure>;

    /// `D0(Param, M, V, Poles2d)`: the placement and the 2d poles.
    fn d0_poles2d(
        &self,
        param: f64,
        placement: &mut LocationD0,
        poles2d: &mut Array1<'_, Point2>,
    ) -> Result<bool, Failure>;

    /// `D1(Param, M, V, DM, DV, Poles2d, DPoles2d)`; Standard_NotImplemented by
    /// default.
    fn d1(
        &self,
        param: f64,
        placement: &mut LocationD1,
        poles2d: &mut Array1<'_, Point2>,
        d_poles2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let _ = (param, placement, poles2d, d_poles2d);
        Err(not_implemented("GeomFill_LocationLaw::D1"))
    }

    /// `D2(...)`; Standard_NotImplemented by default.
    fn d2(
        &self,
        param: f64,
        placement: &mut LocationD2,
        poles: LocationPoles2d<'_, '_>,
    ) -> Result<bool, Failure> {
        let _ = (param, placement, poles);
        Err(not_implemented("GeomFill_LocationLaw::D2"))
    }

    /// `Nb2dCurves()` (non-virtual): the traces plus the restrictions.
    fn nb_2d_curves(&self) -> i32 {
        let mut n = self.trace_number();
        if self.has_first_restriction() {
            n += 1;
        }
        if self.has_last_restriction() {
            n += 1;
        }
        n
    }

    /// `HasFirstRestriction()`: false by default.
    fn has_first_restriction(&self) -> bool {
        false
    }

    /// `HasLastRestriction()`: false by default.
    fn has_last_restriction(&self) -> bool {
        false
    }

    /// `TraceNumber()`: 0 by default.
    fn trace_number(&self) -> i32 {
        0
    }

    /// `ErrorStatus()`: PipeOk by default.
    fn error_status(&self) -> GeomFillPipeError {
        GeomFillPipeError::PipeOk
    }

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

    /// `Resolution(Index, Tol, TolU, TolV)`; Standard_NotImplemented by default.
    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        let _ = (index, tol);
        Err(not_implemented("GeomFill_LocationLaw::Resolution"))
    }

    /// `SetTolerance(Tol3d, Tol2d)`: nothing by default.
    fn set_tolerance(&self, tol3d: f64, tol2d: f64) -> Result<(), Failure> {
        let _ = (tol3d, tol2d);
        Ok(())
    }

    /// `GetMaximalNorm()`.
    fn get_maximal_norm(&self) -> Result<f64, Failure>;

    /// `GetAverageLaw(AM, AV)`.
    fn get_average_law(&self) -> Result<LocationD0, Failure>;

    /// `IsTranslation(Error)`: false by default, `error` untouched.
    fn is_translation(&self, error: &mut f64) -> Result<bool, Failure> {
        let _ = error;
        Ok(false)
    }

    /// `IsRotation(Error)`: false by default, `error` untouched.
    fn is_rotation(&self, error: &mut f64) -> Result<bool, Failure> {
        let _ = error;
        Ok(false)
    }

    /// `Rotation(Center)`; Standard_NotImplemented by default, with the native
    /// message "GeomFill_SectionLaw::Rotation".
    fn rotation(&self, center: &mut Point3) -> Result<(), Failure> {
        let _ = center;
        Err(not_implemented("GeomFill_SectionLaw::Rotation"))
    }
}
impl_transient_interface!(
    dyn GeomFillLocationLaw,
    "GeomFill_LocationLaw",
    StandardTransient
);
