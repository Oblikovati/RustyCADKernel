// Rust translation of GeomBndLib_OffsetCurve2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_OffsetCurve2d`: exact boxes for offsets of lines and circles, otherwise
//! the sampled offset curve (both `Box` and `BoxOptimal`, unlike the 3D class).
use super::circle::Circle2d;
use super::line::Line2d;
use super::live;
use super::other_curve::OtherCurve2d;
use super::spline_curve2d::adaptor2d_of;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Vector2};

/// `GeomBndLib_OffsetCurve2d`: the box of a `Geom2d_OffsetCurve`.
pub struct OffsetCurve2d {
    geom: Handle<Geom2dOffsetCurve>,
}

// `tryOffsetLine2d`: the line shifted by `offset` along its right normal `(Dy, -Dx)`.
fn try_offset_line(
    offset: &Geom2dOffsetCurve,
    range: (f64, f64),
    tolerance: f64,
) -> Result<Option<BoundingBox2>, Failure> {
    let line: Handle<Geom2dLine> = offset.basis_curve().cast();
    let Some(line) = line.get() else {
        return Ok(None);
    };
    let basis: Line2 = line.lin2d();
    let direction = basis.direction();
    let mut normal: Vector2 = Vector2::new(direction.y(), -direction.x());
    if normal.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    normal.normalize()?;
    normal *= offset.offset();
    let shifted: Line2 = Line2::new(basis.location().translated(normal), direction);
    let mut bounds: BoundingBox2 = Line2d::lin_box(&shifted, range.0, range.1, 0.0)?;
    bounds.enlarge(tolerance);
    Ok(Some(bounds))
}

// `tryOffsetCircle2d`: a concentric circle of radius `R + offset * sign`, the sign read
// from the normal at the mid parameter.
fn try_offset_circle(
    offset: &Geom2dOffsetCurve,
    (u1, u2): (f64, f64),
    tolerance: f64,
) -> Result<Option<BoundingBox2>, Failure> {
    let circle: Handle<Geom2dCircle> = offset.basis_curve().cast();
    let Some(circle) = circle.get() else {
        return Ok(None);
    };
    let basis = circle.circ2d()?;
    let (point, d1) = elclib::d1(0.5 * (u1 + u2), &basis);
    if d1.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    let mut normal: Vector2 = Vector2::new(d1.y(), -d1.x());
    if normal.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    normal.normalize()?;
    let mut radial: Vector2 = Vector2::between(basis.position().location(), point);
    if radial.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    radial.normalize()?;
    let sign: f64 = normal.dot(radial);
    if (sign.abs() - 1.0).abs() > 1e-9 {
        return Ok(None);
    }
    let radius: f64 = basis.radius() + offset.offset() * sign;
    let mut bounds: BoundingBox2 = BoundingBox2::default();
    if radius > precision::confusion() {
        let mut shifted = basis;
        shifted.set_radius(radius)?;
        bounds = Circle2d::circ_arc_box(&shifted, u1, u2, 0.0);
    } else if radius.abs() <= precision::confusion() {
        bounds.add_point(basis.position().location());
    } else {
        return Ok(None);
    }
    bounds.enlarge(tolerance);
    Ok(Some(bounds))
}

fn try_exact(
    offset: &Geom2dOffsetCurve,
    range: (f64, f64),
    tolerance: f64,
) -> Result<Option<BoundingBox2>, Failure> {
    if let Some(bounds) = try_offset_line(offset, range, tolerance)? {
        return Ok(Some(bounds));
    }
    try_offset_circle(offset, range, tolerance)
}

impl OffsetCurve2d {
    /// The evaluator of `curve`; `OffsetCurve2d::new(handle)`.
    pub fn new(curve: Handle<Geom2dOffsetCurve>) -> Self {
        Self { geom: curve }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<Geom2dOffsetCurve> {
        &self.geom
    }
    fn offset(&self) -> Result<&Geom2dOffsetCurve, Failure> {
        live(&self.geom, "GeomBndLib_OffsetCurve2d")
    }
    /// `Box(Tol)`: the curve's own range; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let curve = self.offset()?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `Box(U1, U2, Tol)`: the exact paths, otherwise the sampled box of the offset
    /// curve (`GeomBndLib_OtherCurve2d::Box`); `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        if let Some(bounds) = try_exact(self.offset()?, (u1, u2), tolerance)? {
            return Ok(bounds);
        }
        let adaptor: Geom2dAdaptorCurve = adaptor2d_of(&self.geom)?;
        let adaptor: &dyn Adaptor2dCurve2d = &adaptor;
        OtherCurve2d::new(adaptor).bounding_box_range(u1, u2, tolerance)
    }
    /// `BoxOptimal(Tol)`: the curve's own range; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let curve = self.offset()?;
        self.bounding_box_optimal_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: the exact paths, otherwise
    /// `GeomBndLib_OtherCurve2d::BoxOptimal`; `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        if let Some(bounds) = try_exact(self.offset()?, (u1, u2), tolerance)? {
            return Ok(bounds);
        }
        let adaptor: Geom2dAdaptorCurve = adaptor2d_of(&self.geom)?;
        let adaptor: &dyn Adaptor2dCurve2d = &adaptor;
        OtherCurve2d::new(adaptor).bounding_box_optimal_range(u1, u2, tolerance)
    }
}
