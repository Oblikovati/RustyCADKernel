// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Plane.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Plane`: `P(u, v) = Location + u * XDir + v * YDir` on a grid.
use super::surface_grid::{any_empty, constant_surface_grid, empty_grid, invalid_orders, map_rows};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::GeomPlane;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// Batch evaluator of a `Geom_Plane` (`GeomGridEval_Plane`).
///
/// The U terms `Location + u * XDir` are computed once per row; a null plane or an
/// empty parameter array gives an empty grid, and the plane cannot raise;
/// `GeomGridEvalPlane::new(plane).evaluate_grid(&u, &v)`.
pub struct GeomGridEvalPlane {
    geom: Handle<GeomPlane>,
}

// The pre-extracted location and axis coordinates (`aLocX`...`aYZ`).
struct PlaneCoords {
    loc: [f64; 3],
    x: [f64; 3],
    y: [f64; 3],
}

fn coordinates(d: Direction3) -> [f64; 3] {
    [d.x(), d.y(), d.z()]
}

fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
}

impl PlaneCoords {
    // `uX = aLocX + u * aXX` (and Y, Z), once per row.
    fn row(&self, u: f64) -> [f64; 3] {
        let (l, x) = (self.loc, self.x);
        [l[0] + u * x[0], l[1] + u * x[1], l[2] + u * x[2]]
    }
    // `uX + v * aYX` (and Y, Z).
    fn point(&self, row: &[f64; 3], v: f64) -> Point3 {
        let y = self.y;
        Point3::new(row[0] + v * y[0], row[1] + v * y[1], row[2] + v * y[2])
    }
    fn d1u(&self) -> Vector3 {
        Vector3::new(self.x[0], self.x[1], self.x[2])
    }
    fn d1v(&self) -> Vector3 {
        Vector3::new(self.y[0], self.y[1], self.y[2])
    }
    fn d1(&self, row: &[f64; 3], v: f64) -> GeomGridEvalSurfD1 {
        GeomGridEvalSurfD1 {
            point: self.point(row, v),
            d1u: self.d1u(),
            d1v: self.d1v(),
        }
    }
    fn d2(&self, row: &[f64; 3], v: f64) -> GeomGridEvalSurfD2 {
        let first: GeomGridEvalSurfD1 = self.d1(row, v);
        GeomGridEvalSurfD2 {
            point: first.point,
            d1u: first.d1u,
            d1v: first.d1v,
            d2u: zero(),
            d2v: zero(),
            d2uv: zero(),
        }
    }
    fn d3(&self, row: &[f64; 3], v: f64) -> GeomGridEvalSurfD3 {
        let second: GeomGridEvalSurfD2 = self.d2(row, v);
        GeomGridEvalSurfD3 {
            point: second.point,
            d1u: second.d1u,
            d1v: second.d1v,
            d2u: second.d2u,
            d2v: second.d2v,
            d2uv: second.d2uv,
            d3u: zero(),
            d3v: zero(),
            d3uuv: zero(),
            d3uvv: zero(),
        }
    }
}

impl GeomGridEvalPlane {
    /// Evaluator of `plane` (`GeomGridEval_Plane(thePlane)`); a null handle is accepted;
    /// `GeomGridEvalPlane::new(plane)`.
    pub fn new(plane: Handle<GeomPlane>) -> Self {
        Self { geom: plane }
    }
    /// The evaluated plane (`Geometry`); `evaluator.geometry().is_null()`.
    pub fn geometry(&self) -> &Handle<GeomPlane> {
        &self.geom
    }
    fn coords(&self, u: &Array1<'_, f64>, v: &Array1<'_, f64>) -> Option<PlaneCoords> {
        let plane: &GeomPlane = self.geom.get()?;
        if any_empty(u, v) {
            return None;
        }
        let pln = plane.pln();
        let (location, position) = (pln.location(), pln.position());
        Some(PlaneCoords {
            loc: [location.x(), location.y(), location.z()],
            x: coordinates(position.x_direction()),
            y: coordinates(position.y_direction()),
        })
    }
    /// Points at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)`.
    pub fn evaluate_grid(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Array2<'static, Point3> {
        let Some(c) = self.coords(u, v) else {
            return empty_grid();
        };
        map_rows(u, v, |u| c.row(u), |row, v| c.point(row, v))
    }
    /// Points and the constant D1U, D1V (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&u, &v)`.
    pub fn evaluate_grid_d1(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Array2<'static, GeomGridEvalSurfD1> {
        let Some(c) = self.coords(u, v) else {
            return empty_grid();
        };
        map_rows(u, v, |u| c.row(u), |row, v| c.d1(row, v))
    }
    /// Points, D1 and zero D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&u, &v)`.
    pub fn evaluate_grid_d2(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Array2<'static, GeomGridEvalSurfD2> {
        let Some(c) = self.coords(u, v) else {
            return empty_grid();
        };
        map_rows(u, v, |u| c.row(u), |row, v| c.d2(row, v))
    }
    /// Points, D1 and zero D2, D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&u, &v)`.
    pub fn evaluate_grid_d3(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Array2<'static, GeomGridEvalSurfD3> {
        let Some(c) = self.coords(u, v) else {
            return empty_grid();
        };
        map_rows(u, v, |u| c.row(u), |row, v| c.d3(row, v))
    }
    /// Derivative `(nu, nv)`: XDir for `(1, 0)`, YDir for `(0, 1)`, zero otherwise; empty
    /// for negative orders or `nu + nv < 1` (`EvaluateGridDN`);
    /// `evaluator.evaluate_grid_dn(&u, &v, 1, 0)`.
    pub fn evaluate_grid_dn(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
        nu: i32,
        nv: i32,
    ) -> Array2<'static, Vector3> {
        if invalid_orders(nu, nv) {
            return empty_grid();
        }
        let Some(c) = self.coords(u, v) else {
            return empty_grid();
        };
        let derivative: Vector3 = match (nu, nv) {
            (1, 0) => c.d1u(),
            (0, 1) => c.d1v(),
            _ => zero(),
        };
        constant_surface_grid(u.len(), v.len(), derivative)
    }
}
