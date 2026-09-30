// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Cylinder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Cylinder`: `P(u, v) = C + R * (cos(u) * X + sin(u) * Y) + v * Z` on a grid.
use super::quadric::{
    QuadricFrame, QuadricTerms, UContext, quadric_grid, quadric_grid_members, scaled, zero,
};
use super::surface_grid::{any_empty, empty_grid, invalid_orders};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::GeomCylindricalSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Cylinder3, Point3, Vector3};

/// Batch evaluator of a `Geom_CylindricalSurface` (`GeomGridEval_Cylinder`).
///
/// The cylinder is read once per call (`Cylinder()`), the U terms once per row; a
/// null surface or empty parameters give an empty grid;
/// `GeomGridEvalCylinder::new(cylinder).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalCylinder {
    geom: Handle<GeomCylindricalSurface>,
}

// The cylinder `Data`: the frame and the radius.
struct CylinderData {
    frame: QuadricFrame,
    radius: f64,
}

impl CylinderData {
    // `C + R * DirU + v * Z`, as `CX + Radius * dirUX + v * ZX`.
    fn point(&self, ctx: &UContext, v: f64) -> Point3 {
        let (f, r) = (&self.frame, self.radius);
        let at = |k: usize| f.c[k] + r * ctx.dir[k] + v * f.z[k];
        Point3::new(at(0), at(1), at(2))
    }
}

impl QuadricTerms for CylinderData {
    fn frame(&self) -> &QuadricFrame {
        &self.frame
    }
    fn d0(&self, ctx: &UContext, v: f64) -> Point3 {
        self.point(ctx, v)
    }
    // D1U = R * dDirU, D1V = Z.
    fn d1(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD1 {
        GeomGridEvalSurfD1 {
            point: self.point(ctx, v),
            d1u: scaled(self.radius, ctx.ddir),
            d1v: self.frame.z_vector(),
        }
    }
    // D2U = -R * DirU, D2V = D2UV = 0.
    fn d2(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD2 {
        let first: GeomGridEvalSurfD1 = self.d1(ctx, v);
        GeomGridEvalSurfD2 {
            point: first.point,
            d1u: first.d1u,
            d1v: first.d1v,
            d2u: scaled(-self.radius, ctx.dir),
            d2v: zero(),
            d2uv: zero(),
        }
    }
    // D3U = -R * dDirU, every other third derivative is zero.
    fn d3(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD3 {
        let second: GeomGridEvalSurfD2 = self.d2(ctx, v);
        GeomGridEvalSurfD3 {
            point: second.point,
            d1u: second.d1u,
            d1v: second.d1v,
            d2u: second.d2u,
            d2v: second.d2v,
            d2uv: second.d2uv,
            d3u: scaled(-self.radius, ctx.ddir),
            d3v: zero(),
            d3uuv: zero(),
            d3uvv: zero(),
        }
    }
    // V is linear: only D(0,1) = Z survives in V; pure U derivatives cycle.
    fn dn(&self, ctx: &UContext, _v: f64, nu: i32, nv: i32) -> Vector3 {
        match (nu, nv) {
            (_, 2..) => zero(),
            (0, 1) => self.frame.z_vector(),
            (_, 1) => zero(),
            _ => scaled(self.radius, ctx.dir_derivative(nu)),
        }
    }
}

impl GeomGridEvalCylinder {
    /// Evaluator of `cylinder` (`GeomGridEval_Cylinder(theCylinder)`); a null handle is
    /// accepted; `GeomGridEvalCylinder::new(cylinder)`.
    pub fn new(cylinder: Handle<GeomCylindricalSurface>) -> Self {
        Self { geom: cylinder }
    }
    // `extractData` when there is something to evaluate.
    fn terms(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Option<CylinderData>, Failure> {
        let Some(surface) = self.geom.get() else {
            return Ok(None);
        };
        if any_empty(u, v) {
            return Ok(None);
        }
        let cylinder: Cylinder3 = surface.cylinder()?;
        Ok(Some(CylinderData {
            frame: QuadricFrame::new(cylinder.location(), cylinder.position()),
            radius: cylinder.radius(),
        }))
    }
    quadric_grid_members!(GeomCylindricalSurface);
}
