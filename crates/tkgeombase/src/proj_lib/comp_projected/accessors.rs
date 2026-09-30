// Rust translation of the setters and accessors of ProjLib_CompProjectedCurve
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The options, the parts and the per-part results of `ProjLib_CompProjectedCurve`.
use super::parts::{Parts, SequenceOfSequenceOfPnt};
use super::{CompProjectedCurve, CompState};
use crate::native_failure::failure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};

// Standard_NoSuchObject unless `index` names a part.
fn check_part(state: &CompState, index: i32) -> Result<(), Failure> {
    if index < 1 || index > state.nb_curves {
        return Err(failure(
            "Standard_NoSuchObject",
            &format!(
                "ProjLib_CompProjectedCurve: part {index}; expected 1..{}",
                state.nb_curves
            ),
        ));
    }
    Ok(())
}

// The value of a per-part array: Standard_NullObject before it is computed,
// Standard_OutOfRange outside 1..N.
fn array_value<T: Clone>(array: &Option<Vec<T>>, index: i32, name: &str) -> Result<T, Failure> {
    let values: &Vec<T> = array.as_ref().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            &format!("ProjLib_CompProjectedCurve::{name}: not computed"),
        )
    })?;
    if index < 1 || index as usize > values.len() {
        return Err(failure(
            "Standard_OutOfRange",
            &format!(
                "ProjLib_CompProjectedCurve::{name}: index {index}; expected 1..{}",
                values.len()
            ),
        ));
    }
    Ok(values[(index - 1) as usize].clone())
}

impl CompProjectedCurve {
    /// `SetTol3d(theTol3d)`: the 3D tolerance of the approximation;
    /// `projection.set_tol3d(1e-6)`.
    pub fn set_tol3d(&self, tol3d: f64) {
        self.state.write().tol3d = tol3d;
    }
    /// `SetContinuity(theContinuity)` (C2 by default); `projection.set_continuity(Shape::C1)`.
    pub fn set_continuity(&self, continuity: Shape) {
        self.state.write().continuity = continuity;
    }
    /// `SetMaxDegree(theMaxDegree)`: ignored below 1; `projection.set_max_degree(8)`.
    pub fn set_max_degree(&self, max_degree: i32) {
        if max_degree >= 1 {
            self.state.write().max_degree = max_degree;
        }
    }
    /// `SetMaxSeg(theMaxSeg)`: ignored below 1; `projection.set_max_seg(16)`.
    pub fn set_max_seg(&self, max_seg: i32) {
        if max_seg >= 1 {
            self.state.write().max_seg = max_seg;
        }
    }
    /// `SetProj2d(theProj2d)`; `projection.set_proj2d(true)`.
    pub fn set_proj2d(&self, proj2d: bool) {
        self.state.write().proj2d = proj2d;
    }
    /// `SetProj3d(theProj3d)`; `projection.set_proj3d(true)`.
    pub fn set_proj3d(&self, proj3d: bool) {
        self.state.write().proj3d = proj3d;
    }
    /// `GetProj2d()`; `projection.get_proj2d()`.
    pub fn get_proj2d(&self) -> bool {
        self.state.read().proj2d
    }
    /// `GetProj3d()`; `projection.get_proj3d()`.
    pub fn get_proj3d(&self) -> bool {
        self.state.read().proj3d
    }
    /// `Load(const occ::handle<Adaptor3d_Surface>&)`; `projection.load_surface(&surface)`.
    pub fn load_surface(&self, surface: &Handle<dyn Adaptor3dSurface>) {
        self.state.write().surface = surface.clone();
    }
    /// `Load(const occ::handle<Adaptor3d_Curve>&)`; `projection.load_curve(&curve)`.
    pub fn load_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) {
        self.state.write().curve = curve.clone();
    }
    /// `GetSurface()`; `projection.get_surface()`.
    pub fn get_surface(&self) -> Handle<dyn Adaptor3dSurface> {
        self.state.read().surface.clone()
    }
    /// `GetCurve()`; `projection.get_curve()`.
    pub fn get_curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.state.read().curve.clone()
    }
    /// `GetTolerance(TolU, TolV)`; `let (tol_u, tol_v) = projection.get_tolerance();`.
    pub fn get_tolerance(&self) -> (f64, f64) {
        let state = self.state.read();
        (state.tol_u, state.tol_v)
    }
    /// `NbCurves()`: the number of continuous parts; `projection.nb_curves()`.
    pub fn nb_curves(&self) -> i32 {
        self.state.read().nb_curves
    }
    /// `GetSequence()`: the shared sequence of `(t, u, v)` triples per part;
    /// `projection.get_sequence()`.
    pub fn get_sequence(&self) -> Handle<SequenceOfSequenceOfPnt> {
        self.state.read().sequence.clone()
    }
    /// `Bounds(Index, Udeb, Ufin)`: the curve range of a part;
    /// `let (first, last) = projection.bounds(1)?;`.
    pub fn bounds(&self, index: i32) -> Result<(f64, f64), Failure> {
        let state: CompState = self.snapshot();
        check_part(&state, index)?;
        let points: Vec<Point3> = state.parts()?.points(index)?;
        Ok((points[0].x(), points[points.len() - 1].x()))
    }
    // The first triple of a part and the part check.
    fn first_triple(&self, index: i32) -> Result<(CompState, Point3), Failure> {
        let state: CompState = self.snapshot();
        check_part(&state, index)?;
        let first: Point3 = state.parts()?.points(index)?[0];
        Ok((state, first))
    }
    /// `IsSinglePnt(Index, P)`: whether the part is a single point, and its first
    /// (u, v); `let (single, uv) = projection.is_single_pnt(1)?;`.
    pub fn is_single_pnt(&self, index: i32) -> Result<(bool, Point2), Failure> {
        let (state, first) = self.first_triple(index)?;
        let point: Point2 = Point2::new(first.y(), first.z());
        Ok((array_value(&state.sngl_pnts, index, "IsSinglePnt")?, point))
    }
    /// `IsUIso(Index, U)`: whether the part is a U iso line, and its first U;
    /// `let (iso, u) = projection.is_u_iso(1)?;`.
    pub fn is_u_iso(&self, index: i32) -> Result<(bool, f64), Failure> {
        let (state, first) = self.first_triple(index)?;
        Ok((array_value(&state.u_iso, index, "IsUIso")?, first.y()))
    }
    /// `IsVIso(Index, V)`: whether the part is a V iso line, and its first V;
    /// `let (iso, v) = projection.is_v_iso(1)?;`.
    pub fn is_v_iso(&self, index: i32) -> Result<(bool, f64), Failure> {
        let (state, first) = self.first_triple(index)?;
        Ok((array_value(&state.v_iso, index, "IsVIso")?, first.z()))
    }
    /// `MaxDistance(Index)`: the largest distance of the part from the curve;
    /// `projection.max_distance(1)?`.
    pub fn max_distance(&self, index: i32) -> Result<f64, Failure> {
        let state: CompState = self.snapshot();
        check_part(&state, index)?;
        array_value(&state.max_distance, index, "MaxDistance")
    }
}

impl CompProjectedCurve {
    /// `ResultIsPoint(theIndex)`; `projection.result_is_point(1)?`.
    pub fn result_is_point(&self, index: i32) -> Result<bool, Failure> {
        array_value(&self.state.read().results.is_point, index, "ResultIsPoint")
    }
    /// `GetResult2dUApproxError(theIndex)`; `projection.get_result_2d_u_approx_error(1)?`.
    pub fn get_result_2d_u_approx_error(&self, index: i32) -> Result<f64, Failure> {
        array_value(
            &self.state.read().results.error_2d_u,
            index,
            "GetResult2dUApproxError",
        )
    }
    /// `GetResult2dVApproxError(theIndex)`; `projection.get_result_2d_v_approx_error(1)?`.
    pub fn get_result_2d_v_approx_error(&self, index: i32) -> Result<f64, Failure> {
        array_value(
            &self.state.read().results.error_2d_v,
            index,
            "GetResult2dVApproxError",
        )
    }
    /// `GetResult3dApproxError(theIndex)`; `projection.get_result_3d_approx_error(1)?`.
    pub fn get_result_3d_approx_error(&self, index: i32) -> Result<f64, Failure> {
        array_value(
            &self.state.read().results.error_3d,
            index,
            "GetResult3dApproxError",
        )
    }
    /// `GetResult2dC(theIndex)`: the 2D curve of the part (null for a point);
    /// `projection.get_result_2d_c(1)?`.
    pub fn get_result_2d_c(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        array_value(&self.state.read().results.curve_2d, index, "GetResult2dC")
    }
    /// `GetResult3dC(theIndex)`: the 3D curve of the part (null for a point);
    /// `projection.get_result_3d_c(1)?`.
    pub fn get_result_3d_c(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        array_value(&self.state.read().results.curve_3d, index, "GetResult3dC")
    }
    /// `GetResult2dP(theIndex)`: Standard_TypeMismatch unless the part is a point;
    /// `projection.get_result_2d_p(1)?`.
    pub fn get_result_2d_p(&self, index: i32) -> Result<Point2, Failure> {
        let results = self.state.read().results.clone();
        if !array_value(&results.is_point, index, "GetResult2dP")? {
            return Err(failure(
                "Standard_TypeMismatch",
                "ProjLib_CompProjectedCurve : result is not a point 2d",
            ));
        }
        array_value(&results.point_2d, index, "GetResult2dP")
    }
    /// `GetResult3dP(theIndex)`: Standard_TypeMismatch unless the part is a point;
    /// `projection.get_result_3d_p(1)?`.
    pub fn get_result_3d_p(&self, index: i32) -> Result<Point3, Failure> {
        let results = self.state.read().results.clone();
        if !array_value(&results.is_point, index, "GetResult3dP")? {
            return Err(failure(
                "Standard_TypeMismatch",
                "ProjLib_CompProjectedCurve : result is not a point 3d",
            ));
        }
        array_value(&results.point_3d, index, "GetResult3dP")
    }
}
