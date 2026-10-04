// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Surface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Surface`: the grid evaluator that picks a specialized one per surface
//! kind and applies the transformation of a transformed adaptor.
use super::surface_adaptor::{AdaptorResolution, resolve_adaptor};
use super::surface_grid::empty_grid;
use super::surface_transform::{TransformedRecord, apply_transformation};
use super::{
    GeomGridEvalBSplineSurface, GeomGridEvalBezierSurface, GeomGridEvalCone, GeomGridEvalCylinder,
    GeomGridEvalOffsetSurface, GeomGridEvalOtherSurface, GeomGridEvalPlane, GeomGridEvalSphere,
    GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3, GeomGridEvalSurfaceOfExtrusion,
    GeomGridEvalSurfaceOfRevolution, GeomGridEvalTorus,
};
use crate::adaptor3d::Adaptor3dSurface;
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

/// The specialized evaluator held by `GeomGridEval_Surface` (`EvaluatorVariant`);
/// `Uninitialized` is the native `std::monostate`, whose grids are empty.
pub enum GeomGridEvalSurfaceEvaluator<'a> {
    /// No evaluator: a null surface.
    Uninitialized,
    /// `GeomGridEval_Plane`.
    Plane(GeomGridEvalPlane),
    /// `GeomGridEval_Cylinder`.
    Cylinder(GeomGridEvalCylinder),
    /// `GeomGridEval_Sphere`.
    Sphere(GeomGridEvalSphere),
    /// `GeomGridEval_Cone`.
    Cone(GeomGridEvalCone),
    /// `GeomGridEval_Torus`.
    Torus(GeomGridEvalTorus),
    /// `GeomGridEval_BezierSurface`.
    Bezier(GeomGridEvalBezierSurface),
    /// `GeomGridEval_BSplineSurface`.
    BSpline(GeomGridEvalBSplineSurface),
    /// `GeomGridEval_OffsetSurface`.
    Offset(GeomGridEvalOffsetSurface),
    /// `GeomGridEval_SurfaceOfRevolution`.
    Revolution(GeomGridEvalSurfaceOfRevolution),
    /// `GeomGridEval_SurfaceOfExtrusion`.
    Extrusion(GeomGridEvalSurfaceOfExtrusion),
    /// `GeomGridEval_OtherSurface`, the fallback on an adaptor or a Geom surface.
    Other(GeomGridEvalOtherSurface<'a>),
}

// Calls the same member on whichever evaluator is held; the Plane evaluator cannot fail.
macro_rules! dispatch {
    ($self:ident, $eval:ident => $call:expr) => {
        match &$self.evaluator {
            GeomGridEvalSurfaceEvaluator::Uninitialized => Ok(empty_grid()),
            GeomGridEvalSurfaceEvaluator::Plane($eval) => Ok($call),
            GeomGridEvalSurfaceEvaluator::Cylinder($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Sphere($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Cone($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Torus($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Bezier($eval) => $call,
            GeomGridEvalSurfaceEvaluator::BSpline($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Offset($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Revolution($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Extrusion($eval) => $call,
            GeomGridEvalSurfaceEvaluator::Other($eval) => $call,
        }
    };
}

/// Grid evaluator of any 3D surface (`GeomGridEval_Surface`).
///
/// From a surface handle, rectangular trimmed wrappers are stripped and the basis class
/// selects the evaluator in the order plane, cylinder, sphere, cone, torus, Bezier,
/// B-spline, offset, revolution, extrusion; any other class (the GeomEval surfaces)
/// goes to `GeomGridEval_OtherSurface` on the handle with the type `OtherSurface`, and
/// a null handle leaves the evaluator uninitialized. Trimming bounds are not applied.
/// From an adaptor, see `surface_adaptor`: a `GeomAdaptor_TransformedSurface` keeps a
/// non-identity transformation, applied to every grid; `GetType` failures of the
/// adaptor are returned; `GeomGridEvalSurface::from_surface(&surface).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalSurface<'a> {
    evaluator: GeomGridEvalSurfaceEvaluator<'a>,
    surface_type: SurfaceType,
    trsf: Option<Transform3>,
}

impl<'a> GeomGridEvalSurface<'a> {
    /// Evaluator of an adaptor (`GeomGridEval_Surface(const Adaptor3d_Surface&)`); the
    /// fallback borrows `surface`; `GeomGridEvalSurface::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(surface: &'a dyn Adaptor3dSurface) -> Result<Self, Failure> {
        let mut result = Self::uninitialized();
        result.initialization_adaptor(surface)?;
        Ok(result)
    }
    /// Evaluator of a surface handle (`GeomGridEval_Surface(const
    /// occ::handle<Geom_Surface>&)`); `GeomGridEvalSurface::from_surface(&surface)`.
    pub fn from_surface(surface: &Handle<dyn GeomSurface>) -> Self {
        let mut result = Self::uninitialized();
        result.initialization_surface(surface);
        result
    }
    fn uninitialized() -> Self {
        Self {
            evaluator: GeomGridEvalSurfaceEvaluator::Uninitialized,
            surface_type: SurfaceType::OtherSurface,
            trsf: None,
        }
    }
    /// Reinitialize through the native protected adaptor hook.
    /// Clears the previous transformation before querying the adaptor, including
    /// when that query fails. Example: `eval.initialization_adaptor(&adaptor)?`.
    pub fn initialization_adaptor(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
    ) -> Result<(), Failure> {
        // Native GeomGridEval_Surface.cxx resets myTrsf before any failing query.
        self.trsf = None;
        match resolve_adaptor(surface)? {
            AdaptorResolution::Geometry(geom, trsf) => {
                self.trsf = trsf;
                self.initialization_surface(&geom);
            }
            AdaptorResolution::Fallback(surface_type, adaptor) => {
                self.surface_type = surface_type;
                let other = GeomGridEvalOtherSurface::from_adaptor(adaptor);
                self.evaluator = GeomGridEvalSurfaceEvaluator::Other(other);
            }
        }
        Ok(())
    }
    /// Reinitialize through the native protected geometry hook, retaining any
    /// adaptor transformation. A null handle clears the held surface only.
    /// Example: `eval.initialization_surface(&Handle::default())`.
    pub fn initialization_surface(&mut self, surface: &Handle<dyn GeomSurface>) {
        if surface.is_null() {
            self.evaluator = GeomGridEvalSurfaceEvaluator::Uninitialized;
            self.surface_type = SurfaceType::OtherSurface;
            return;
        }
        let (surface_type, evaluator) = classify(&extract_basis_surface(surface));
        self.surface_type = surface_type;
        self.evaluator = evaluator;
    }
    /// The held evaluator (the private `myEvaluator`, read-only); `evaluator.evaluator()`.
    pub fn evaluator(&self) -> &GeomGridEvalSurfaceEvaluator<'a> {
        &self.evaluator
    }
    /// The detected kind (`GetType`); `evaluator.get_type() == SurfaceType::Plane`.
    pub fn get_type(&self) -> SurfaceType {
        self.surface_type
    }
    /// Whether a transformation is applied (`HasTransformation`);
    /// `evaluator.has_transformation()`.
    pub fn has_transformation(&self) -> bool {
        self.trsf.is_some()
    }
    /// The applied transformation, if any (`GetTransformation`);
    /// `evaluator.get_transformation().is_none()`.
    pub fn get_transformation(&self) -> Option<&Transform3> {
        self.trsf.as_ref()
    }
    // The dispatched grid, transformed when a transformation is held.
    fn transformed<T: TransformedRecord>(
        &self,
        grid: Result<Array2<'static, T>, Failure>,
    ) -> Result<Array2<'static, T>, Failure> {
        let mut grid: Array2<'static, T> = grid?;
        apply_transformation(&mut grid, self.trsf);
        Ok(grid)
    }
    /// Points at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, Point3>, Failure> {
        self.transformed(dispatch!(self, eval => eval.evaluate_grid(u, v)))
    }
    /// Points and first derivatives (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&u, &v)?`.
    pub fn evaluate_grid_d1(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD1>, Failure> {
        self.transformed(dispatch!(self, eval => eval.evaluate_grid_d1(u, v)))
    }
    /// Points and derivatives up to order 2 (`EvaluateGridD2`);
    /// `evaluator.evaluate_grid_d2(&u, &v)?`.
    pub fn evaluate_grid_d2(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD2>, Failure> {
        self.transformed(dispatch!(self, eval => eval.evaluate_grid_d2(u, v)))
    }
    /// Points and derivatives up to order 3 (`EvaluateGridD3`);
    /// `evaluator.evaluate_grid_d3(&u, &v)?`.
    pub fn evaluate_grid_d3(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD3>, Failure> {
        self.transformed(dispatch!(self, eval => eval.evaluate_grid_d3(u, v)))
    }
    /// Derivative `(nu, nv)` at every `(u, v)` (`EvaluateGridDN`);
    /// `evaluator.evaluate_grid_dn(&u, &v, 1, 1)?`.
    pub fn evaluate_grid_dn(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
        nu: i32,
        nv: i32,
    ) -> Result<Array2<'static, Vector3>, Failure> {
        self.transformed(dispatch!(self, eval => eval.evaluate_grid_dn(u, v, nu, nv)))
    }
}

/// `ExtractBasisSurface`: the basis under any number of rectangular trimmed wrappers.
fn extract_basis_surface(surface: &Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    let mut result: Handle<dyn GeomSurface> = surface.clone();
    loop {
        let trimmed: Handle<GeomRectangularTrimmedSurface> = result.cast();
        let Some(wrapper) = trimmed.get() else {
            return result;
        };
        result = wrapper.basis_surface();
    }
}

// The `occ::down_cast` chain of `initialization`: the first class that matches wins.
fn classify<'a>(
    basis: &Handle<dyn GeomSurface>,
) -> (SurfaceType, GeomGridEvalSurfaceEvaluator<'a>) {
    use GeomGridEvalSurfaceEvaluator as E;
    if let Some(plane) = kind::<GeomPlane>(basis) {
        return (SurfaceType::Plane, E::Plane(GeomGridEvalPlane::new(plane)));
    }
    if let Some(cylinder) = kind::<GeomCylindricalSurface>(basis) {
        return (
            SurfaceType::Cylinder,
            E::Cylinder(GeomGridEvalCylinder::new(cylinder)),
        );
    }
    if let Some(sphere) = kind::<GeomSphericalSurface>(basis) {
        return (
            SurfaceType::Sphere,
            E::Sphere(GeomGridEvalSphere::new(sphere)),
        );
    }
    if let Some(cone) = kind::<GeomConicalSurface>(basis) {
        return (SurfaceType::Cone, E::Cone(GeomGridEvalCone::new(cone)));
    }
    if let Some(torus) = kind::<GeomToroidalSurface>(basis) {
        return (SurfaceType::Torus, E::Torus(GeomGridEvalTorus::new(torus)));
    }
    classify_other(basis)
}

// The spline, offset and swept surfaces, then the fallback on the handle.
fn classify_other<'a>(
    basis: &Handle<dyn GeomSurface>,
) -> (SurfaceType, GeomGridEvalSurfaceEvaluator<'a>) {
    use GeomGridEvalSurfaceEvaluator as E;
    if let Some(bezier) = kind::<GeomBezierSurface>(basis) {
        let evaluator = GeomGridEvalBezierSurface::new(bezier);
        return (SurfaceType::BezierSurface, E::Bezier(evaluator));
    }
    if let Some(bspline) = kind::<GeomBSplineSurface>(basis) {
        let evaluator = GeomGridEvalBSplineSurface::new(bspline);
        return (SurfaceType::BSplineSurface, E::BSpline(evaluator));
    }
    if let Some(offset) = kind::<GeomOffsetSurface>(basis) {
        let evaluator = GeomGridEvalOffsetSurface::new(offset);
        return (SurfaceType::OffsetSurface, E::Offset(evaluator));
    }
    if let Some(revolution) = kind::<GeomSurfaceOfRevolution>(basis) {
        let evaluator = GeomGridEvalSurfaceOfRevolution::new(revolution);
        return (SurfaceType::SurfaceOfRevolution, E::Revolution(evaluator));
    }
    if let Some(extrusion) = kind::<GeomSurfaceOfLinearExtrusion>(basis) {
        let evaluator = GeomGridEvalSurfaceOfExtrusion::new(extrusion);
        return (SurfaceType::SurfaceOfExtrusion, E::Extrusion(evaluator));
    }
    let evaluator = GeomGridEvalOtherSurface::from_surface(basis.clone());
    (SurfaceType::OtherSurface, E::Other(evaluator))
}

// `occ::down_cast<T>(basis)` when it succeeds.
fn kind<T: RuntimeTyped>(basis: &Handle<dyn GeomSurface>) -> Option<Handle<T>> {
    let typed: Handle<T> = basis.cast::<T>();
    (!typed.is_null()).then_some(typed)
}
