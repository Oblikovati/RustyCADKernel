// Rust translation of GeomBndLib_Line.hxx and GeomBndLib_Line2d.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Line` and `GeomBndLib_Line2d`: boxes of line segments, opened along the
//! direction for infinite parameters.
use super::live;
use super::ranges::add_line_range;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Line3};

/// `GeomBndLib_Line`: the box of a `Geom_Line` segment; `Line::lin_box` takes a `gp_Lin`.
pub struct Line {
    geom: Handle<GeomLine>,
}

impl Line {
    /// The evaluator of `line`; `Line::new(handle)`.
    pub fn new(line: Handle<GeomLine>) -> Self {
        Self { geom: line }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomLine> {
        &self.geom
    }
    /// `Box(U1, U2, Tol)`; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        Self::lin_box(
            &live(&self.geom, "GeomBndLib_Line")?.lin(),
            u1,
            u2,
            tolerance,
        )
    }
    /// `Box(Tol)`: the whole (infinite) line, which opens its box;
    /// `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let line = live(&self.geom, "GeomBndLib_Line")?;
        self.bounding_box_range(line.first_parameter(), line.last_parameter(), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Lin, U1, U2, Tol)`: the segment ends; an infinite end opens the sides the
    /// direction reaches. Two infinite ends of the same sign raise Standard_Failure;
    /// `Line::lin_box(&line, 0.0, 10.0, 0.0)?`.
    pub fn lin_box(
        line: &Line3,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        let value = |u: f64| elclib::value(u, line);
        let message = "GeomBndLib_Line::Box - bad parameter";
        add_line_range(&mut bounds, line.direction(), (u1, u2), &value, message)?;
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
}

/// `GeomBndLib_Line2d`: the box of a `Geom2d_Line` segment; `Line2d::lin_box` takes a
/// `gp_Lin2d`.
pub struct Line2d {
    geom: Handle<Geom2dLine>,
}

impl Line2d {
    /// The evaluator of `line`; `Line2d::new(handle)`.
    pub fn new(line: Handle<Geom2dLine>) -> Self {
        Self { geom: line }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<Geom2dLine> {
        &self.geom
    }
    /// `Box(U1, U2, Tol)`; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        Self::lin_box(
            &live(&self.geom, "GeomBndLib_Line2d")?.lin2d(),
            u1,
            u2,
            tolerance,
        )
    }
    /// `Box(Tol)`: the whole (infinite) line; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let line = live(&self.geom, "GeomBndLib_Line2d")?;
        self.bounding_box_range(line.first_parameter(), line.last_parameter(), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Lin2d, U1, U2, Tol)`: as [`Line::lin_box`] in 2D;
    /// `Line2d::lin_box(&line, 0.0, 10.0, 0.0)?`.
    pub fn lin_box(
        line: &Line2,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        let mut bounds: BoundingBox2 = BoundingBox2::default();
        let value = |u: f64| elclib::value(u, line);
        let message = "GeomBndLib_Line2d::Box - bad parameter";
        add_line_range(&mut bounds, line.direction(), (u1, u2), &value, message)?;
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
}
