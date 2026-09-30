// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_OtherSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_OtherSurface`: the fallback grid evaluator through the `D0`..`DN`
//! members of an adaptor or of a Geom surface.
use super::surface_grid::{any_empty, empty_grid, invalid_orders, try_map_rows};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::adaptor3d::Adaptor3dSurface;
use crate::geom::GeomSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors::null_object;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The surface source of the fallback (`std::variant<const Adaptor3d_Surface*,
/// occ::handle<Geom_Surface>>`).
pub enum GeomGridEvalSurfaceSource<'a> {
    /// A borrowed adaptor, which must outlive the evaluator.
    Adaptor(&'a dyn Adaptor3dSurface),
    /// A Geom surface handle.
    Geometry(Handle<dyn GeomSurface>),
}

/// Grid evaluation through the non-virtual `D0`..`DN` members of any adaptor or Geom
/// surface (`GeomGridEval_OtherSurface`).
///
/// A failure aborts the grid. A null Geom handle is dereferenced natively (undefined
/// behaviour); the port raises Standard_NullObject instead;
/// `GeomGridEvalOtherSurface::from_adaptor(&adaptor).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalOtherSurface<'a> {
    surface: GeomGridEvalSurfaceSource<'a>,
}

impl<'a> GeomGridEvalOtherSurface<'a> {
    /// Evaluator over a borrowed adaptor (`GeomGridEval_OtherSurface(const
    /// Adaptor3d_Surface*)`); `GeomGridEvalOtherSurface::from_adaptor(&adaptor)`.
    pub fn from_adaptor(surface: &'a dyn Adaptor3dSurface) -> Self {
        Self {
            surface: GeomGridEvalSurfaceSource::Adaptor(surface),
        }
    }
    /// Evaluator over a Geom surface (`GeomGridEval_OtherSurface(const
    /// occ::handle<Geom_Surface>&)`); `GeomGridEvalOtherSurface::from_surface(surface)`.
    pub fn from_surface(surface: Handle<dyn GeomSurface>) -> Self {
        Self {
            surface: GeomGridEvalSurfaceSource::Geometry(surface),
        }
    }
    /// The evaluated source (the private `mySurface`, read-only); `evaluator.source()`.
    pub fn source(&self) -> &GeomGridEvalSurfaceSource<'a> {
        &self.surface
    }
    // The Geom surface, or the NullObject failure of a null handle.
    fn geometry(surface: &Handle<dyn GeomSurface>) -> Result<&dyn GeomSurface, Failure> {
        surface.get().ok_or_else(|| {
            null_object("GeomGridEval_OtherSurface: null Geom_Surface handle; expected a surface")
        })
    }
    // `evaluateD0`.
    fn d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        match &self.surface {
            GeomGridEvalSurfaceSource::Adaptor(a) => a.d0(u, v),
            GeomGridEvalSurfaceSource::Geometry(s) => Self::geometry(s)?.d0(u, v),
        }
    }
    // `evaluateD1`.
    fn d1(&self, u: f64, v: f64) -> Result<GeomGridEvalSurfD1, Failure> {
        match &self.surface {
            GeomGridEvalSurfaceSource::Adaptor(a) => a.d1(u, v),
            GeomGridEvalSurfaceSource::Geometry(s) => Self::geometry(s)?.d1(u, v),
        }
    }
    // `evaluateD2`.
    fn d2(&self, u: f64, v: f64) -> Result<GeomGridEvalSurfD2, Failure> {
        match &self.surface {
            GeomGridEvalSurfaceSource::Adaptor(a) => a.d2(u, v),
            GeomGridEvalSurfaceSource::Geometry(s) => Self::geometry(s)?.d2(u, v),
        }
    }
    // `evaluateD3`.
    fn d3(&self, u: f64, v: f64) -> Result<GeomGridEvalSurfD3, Failure> {
        match &self.surface {
            GeomGridEvalSurfaceSource::Adaptor(a) => a.d3(u, v),
            GeomGridEvalSurfaceSource::Geometry(s) => Self::geometry(s)?.d3(u, v),
        }
    }
    // `evaluateDN`.
    fn dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        match &self.surface {
            GeomGridEvalSurfaceSource::Adaptor(a) => a.dn(u, v, nu, nv),
            GeomGridEvalSurfaceSource::Geometry(s) => Self::geometry(s)?.dn(u, v, nu, nv),
        }
    }
    // The cell loop of every member; empty when either array is empty.
    fn cells<T: Default>(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
        eval: impl Fn(&Self, f64, f64) -> Result<T, Failure>,
    ) -> Result<Array2<'static, T>, Failure> {
        if any_empty(u, v) {
            return Ok(empty_grid());
        }
        try_map_rows(u, v, Ok, |&u, v| eval(self, u, v))
    }
    /// `D0` at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, Point3>, Failure> {
        self.cells(u, v, Self::d0)
    }
    /// `D1` at every `(u, v)` (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&u, &v)?`.
    pub fn evaluate_grid_d1(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD1>, Failure> {
        self.cells(u, v, Self::d1)
    }
    /// `D2` at every `(u, v)` (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&u, &v)?`.
    pub fn evaluate_grid_d2(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD2>, Failure> {
        self.cells(u, v, Self::d2)
    }
    /// `D3` at every `(u, v)` (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&u, &v)?`.
    pub fn evaluate_grid_d3(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Array2<'static, GeomGridEvalSurfD3>, Failure> {
        self.cells(u, v, Self::d3)
    }
    /// `DN(u, v, nu, nv)` at every `(u, v)`; empty for negative orders or `nu + nv < 1`
    /// (`EvaluateGridDN`); `evaluator.evaluate_grid_dn(&u, &v, 1, 1)?`.
    pub fn evaluate_grid_dn(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
        nu: i32,
        nv: i32,
    ) -> Result<Array2<'static, Vector3>, Failure> {
        if invalid_orders(nu, nv) {
            return Ok(empty_grid());
        }
        self.cells(u, v, |e, u, v| e.dn(u, v, nu, nv))
    }
}
