// Rust translation of GeomBndLib_OffsetCurve at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_OffsetCurve`: exact boxes for offsets of lines and of circles offset in
//! their plane, otherwise the basis box enlarged per coordinate (`Box`) or the sampled
//! offset curve (`BoxOptimal`).
use super::circle::Circle;
use super::curve::Curve;
use super::line::Line;
use super::live;
use super::other_curve::OtherCurve;
use super::spline_curve::adaptor_of;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomOffsetCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::std_max;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line3, Vector3};

/// `GeomBndLib_OffsetCurve`: the box of a `Geom_OffsetCurve`.
pub struct OffsetCurve {
    geom: Handle<GeomOffsetCurve>,
}

// `tryOffsetLine`: the offset of a line is the line shifted by `offset * (D ^ V)`.
fn try_offset_line(
    offset: &GeomOffsetCurve,
    range: (f64, f64),
    tolerance: f64,
) -> Result<Option<BoundingBox3>, Failure> {
    let line: Handle<GeomLine> = offset.basis_curve().cast();
    let Some(line) = line.get() else {
        return Ok(None);
    };
    let basis: Line3 = line.lin();
    let mut shift: Vector3 =
        Vector3::from(basis.direction()).crossed(Vector3::from(offset.direction()));
    if shift.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    shift.normalize()?;
    shift *= offset.offset();
    let shifted: Line3 = Line3::new(basis.location().translated(shift), basis.direction());
    let mut bounds: BoundingBox3 = Line::lin_box(&shifted, range.0, range.1, 0.0)?;
    bounds.enlarge(tolerance);
    Ok(Some(bounds))
}

// `tryOffsetCircle`: a circle offset along its axis direction is a concentric circle of
// radius `R + offset * sign`, the sign read from the normal at the mid parameter.
fn try_offset_circle(
    offset: &GeomOffsetCurve,
    (u1, u2): (f64, f64),
    tolerance: f64,
) -> Result<Option<BoundingBox3>, Failure> {
    let circle: Handle<GeomCircle> = offset.basis_curve().cast();
    let Some(circle) = circle.get() else {
        return Ok(None);
    };
    let basis = circle.circ()?;
    let direction = offset.direction();
    let dot_axis: f64 = basis.position().direction().dot(direction).abs();
    if (1.0 - dot_axis).abs() > precision::angular() {
        return Ok(None);
    }
    let (point, d1) = elclib::d1(0.5 * (u1 + u2), &basis);
    if d1.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    let mut normal: Vector3 = d1.crossed(Vector3::from(direction));
    if normal.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    normal.normalize()?;
    let mut radial: Vector3 = Vector3::between(basis.position().location(), point);
    if radial.square_magnitude() <= precision::square_confusion() {
        return Ok(None);
    }
    radial.normalize()?;
    let sign: f64 = normal.dot(radial);
    if (sign.abs() - 1.0).abs() > 1e-9 {
        return Ok(None);
    }
    let radius: f64 = basis.radius() + offset.offset() * sign;
    let mut bounds: BoundingBox3 = BoundingBox3::default();
    if radius > precision::confusion() {
        let mut shifted = basis;
        shifted.set_radius(radius)?;
        bounds = Circle::circ_arc_box(&shifted, u1, u2, 0.0);
    } else if radius.abs() <= precision::confusion() {
        bounds.add_point(basis.position().location());
    } else {
        return Ok(None);
    }
    bounds.enlarge(tolerance);
    Ok(Some(bounds))
}

// The two exact fast paths, line first.
fn try_exact(
    offset: &GeomOffsetCurve,
    range: (f64, f64),
    tolerance: f64,
) -> Result<Option<BoundingBox3>, Failure> {
    if let Some(bounds) = try_offset_line(offset, range, tolerance)? {
        return Ok(Some(bounds));
    }
    try_offset_circle(offset, range, tolerance)
}

// The generic `Box` fallback: the basis box widened per coordinate by
// `|d| sqrt(1 - Vk^2)`, the bound of the offset displacement, keeping the open sides.
fn enlarged_basis_box(
    offset: &GeomOffsetCurve,
    (u1, u2): (f64, f64),
    tolerance: f64,
) -> Result<BoundingBox3, Failure> {
    let distance: f64 = offset.offset().abs();
    let direction = offset.direction();
    let mut bounds: BoundingBox3 =
        Curve::from_curve(&offset.basis_curve())?.bounding_box_range(u1, u2, 0.0)?;
    if bounds.is_void() {
        return Ok(bounds);
    }
    let widen = |v: f64| distance * std_max(0.0, 1.0 - v * v).sqrt();
    let enlarge: [f64; 3] = [
        widen(direction.x()),
        widen(direction.y()),
        widen(direction.z()),
    ];
    let open: [bool; 6] = open_flags(&bounds);
    if bounds.has_finite_part() {
        let l = bounds.finite_part().limits();
        let min: [f64; 3] = [
            l.x_min - enlarge[0],
            l.y_min - enlarge[1],
            l.z_min - enlarge[2],
        ];
        let max: [f64; 3] = [
            l.x_max + enlarge[0],
            l.y_max + enlarge[1],
            l.z_max + enlarge[2],
        ];
        bounds.update_bounds(min, max);
    }
    reopen(&mut bounds, open);
    bounds.enlarge(tolerance);
    Ok(bounds)
}

fn open_flags(bounds: &BoundingBox3) -> [bool; 6] {
    [
        bounds.is_open_xmin(),
        bounds.is_open_xmax(),
        bounds.is_open_ymin(),
        bounds.is_open_ymax(),
        bounds.is_open_zmin(),
        bounds.is_open_zmax(),
    ]
}

// The native re-opening after `Update` (a no-op, since `Update` keeps the flags).
fn reopen(bounds: &mut BoundingBox3, open: [bool; 6]) {
    let openers: [fn(&mut BoundingBox3); 6] = [
        BoundingBox3::open_xmin,
        BoundingBox3::open_xmax,
        BoundingBox3::open_ymin,
        BoundingBox3::open_ymax,
        BoundingBox3::open_zmin,
        BoundingBox3::open_zmax,
    ];
    for (flag, opener) in open.into_iter().zip(openers) {
        if flag {
            opener(bounds);
        }
    }
}

impl OffsetCurve {
    /// The evaluator of `curve`; `OffsetCurve::new(handle)`.
    pub fn new(curve: Handle<GeomOffsetCurve>) -> Self {
        Self { geom: curve }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomOffsetCurve> {
        &self.geom
    }
    fn offset(&self) -> Result<&GeomOffsetCurve, Failure> {
        live(&self.geom, "GeomBndLib_OffsetCurve")
    }
    /// `Box(Tol)`: the curve's own range; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let curve = self.offset()?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `Box(U1, U2, Tol)`: the exact line/circle paths, otherwise the basis box widened
    /// by the offset per coordinate; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let offset = self.offset()?;
        if let Some(bounds) = try_exact(offset, (u1, u2), tolerance)? {
            return Ok(bounds);
        }
        enlarged_basis_box(offset, (u1, u2), tolerance)
    }
    /// `BoxOptimal(Tol)`: the curve's own range; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let curve = self.offset()?;
        self.bounding_box_optimal_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: the exact paths, otherwise `GeomBndLib_OtherCurve`'s
    /// optimal box of the offset curve itself; `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        if let Some(bounds) = try_exact(self.offset()?, (u1, u2), tolerance)? {
            return Ok(bounds);
        }
        let adaptor: GeomAdaptorCurve = adaptor_of(&self.geom)?;
        let adaptor: &dyn Adaptor3dCurve = &adaptor;
        OtherCurve::new(adaptor).bounding_box_optimal_range(u1, u2, tolerance)
    }
}
