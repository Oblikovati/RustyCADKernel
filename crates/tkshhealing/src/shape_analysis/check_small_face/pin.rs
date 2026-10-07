use super::spot::{ShapeAnalysisCheckSmallFace, axis_is_small};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurface};
use rustycad_tkmath::gp::Point3;

const PIN_TOLERANCE: f64 = 1.0e-9;
const DEFAULT_PRECISION: f64 = 1.0e-4;
const POLE_EQUALITY_TOLERANCE: f64 = 1.0e-15;

impl ShapeAnalysisCheckSmallFace {
    /// Detects a pin on a Bezier or B-spline support; once the support has poles the native
    /// code resets `row` and `sense` to 0 before it scans the four natural bounds.
    /// `analyzer.check_pin(&face, &mut row, &mut sense)?`.
    pub fn check_pin(
        &mut self,
        face: &Face,
        row: &mut i32,
        sense: &mut i32,
    ) -> Result<bool, Failure> {
        let surface = BRepTool::surface_with_location(face)?.0;
        if is_elementary(&surface) {
            return Ok(false);
        }
        let poles = SurfacePoles::of(&surface);
        if poles.u_count == 0 || poles.v_count == 0 {
            return Ok(false);
        }
        // The native precision is read only on this path, where its value matters.
        let tolerance = self.pin_precision()?;
        let pole = |u: i32, v: i32| poles.pole(u, v);
        let scan = inspect_bounds(poles.u_count, poles.v_count, tolerance, &pole)?;
        (*sense, *row) = (scan.0, scan.1);
        if scan.0 == 0 {
            return Ok(false);
        }
        self.record_pin_status(scan.2, &poles)?;
        Ok(true)
    }

    // The switch on the last bound's status leaves the pin status as it was for 0.
    fn record_pin_status(&mut self, status: i32, poles: &SurfacePoles) -> Result<(), Failure> {
        match status {
            1 => self.pin_status = ShapeExtend::encode_status(ShapeExtendStatus::Done1),
            2 => self.pin_status = ShapeExtend::encode_status(ShapeExtendStatus::Done2),
            _ => {}
        }
        let pole = |u: i32, v: i32| poles.pole(u, v);
        if status == 1 && has_identical_boundary_poles(poles.u_count, poles.v_count, &pole)? {
            self.pin_status = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
        }
        Ok(())
    }

    fn pin_precision(&self) -> Result<f64, Failure> {
        let Some(tolerance) = self.tolerance else {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "ShapeAnalysis_CheckSmallFace::CheckPin requires SetTolerance before reading native myPrecision",
            ));
        };
        Ok(if tolerance < 0.0 {
            DEFAULT_PRECISION
        } else {
            tolerance
        })
    }
}

fn is_elementary(surface: &Handle<dyn GeomSurface>) -> bool {
    surface
        .cast::<dyn rustycad_tkg3d::geom::GeomElementarySurface>()
        .get()
        .is_some()
}

/// The pole net of a B-spline or Bezier support; both counts are 0 for another surface.
struct SurfacePoles {
    bspline: Handle<GeomBSplineSurface>,
    bezier: Handle<GeomBezierSurface>,
    u_count: i32,
    v_count: i32,
}

impl SurfacePoles {
    fn of(surface: &Handle<dyn GeomSurface>) -> Self {
        let bspline = surface.cast::<GeomBSplineSurface>();
        let bezier = surface.cast::<GeomBezierSurface>();
        let (u_count, v_count) = match (bspline.get(), bezier.get()) {
            (Some(b), _) => (b.nb_u_poles(), b.nb_v_poles()),
            (_, Some(b)) => (b.nb_u_poles(), b.nb_v_poles()),
            _ => (0, 0),
        };
        Self {
            bspline,
            bezier,
            u_count,
            v_count,
        }
    }

    fn pole(&self, u: i32, v: i32) -> Result<Point3, Failure> {
        if let Some(surface) = self.bspline.get() {
            return surface.pole(u, v);
        }
        let surface = self.bezier.get().ok_or_else(|| {
            Failure::new("CheckPin: support without poles; expected a B-spline or Bezier surface")
        })?;
        surface.pole(u, v)
    }
}

fn inspect_bounds(
    u_count: i32,
    v_count: i32,
    tolerance: f64,
    pole: &impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<(i32, i32, i32), Failure> {
    let mut result = (0, 0, 0);
    for bound in [
        (1, 1, 1, u_count),
        (1, u_count, 1, u_count),
        (2, 1, 2, 1),
        (2, v_count, 2, v_count),
    ] {
        result = inspect_one_bound(result, bound, (u_count, v_count), tolerance, pole)?;
    }
    Ok(result)
}

fn inspect_one_bound(
    previous: (i32, i32, i32),
    bound: (i32, i32, i32, i32),
    counts: (i32, i32),
    tolerance: f64,
    pole: &impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<(i32, i32, i32), Failure> {
    let (direction, rank, sense, row) = bound;
    let status = iso_status(direction, rank, counts.0, counts.1, tolerance, pole)?;
    Ok(if status == 0 {
        (previous.0, previous.1, status)
    } else {
        (sense, row, status)
    })
}

fn iso_status(
    direction: i32,
    rank: i32,
    u_count: i32,
    v_count: i32,
    tolerance: f64,
    pole: &impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<i32, Failure> {
    let bounds = iso_bounds(direction, rank, (u_count, v_count), pole)?;
    if bounds.is_small(PIN_TOLERANCE) {
        return Ok(0);
    }
    Ok(if bounds.is_small(tolerance) { 1 } else { 2 })
}

fn iso_bounds(
    direction: i32,
    rank: i32,
    counts: (i32, i32),
    pole: &impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<PoleBounds, Failure> {
    let mut bounds = PoleBounds::default();
    let count = if direction == 1 { counts.1 } else { counts.0 };
    for index in 1..=count {
        bounds.include(boundary_pole(direction, rank, index, pole)?);
    }
    Ok(bounds)
}

fn boundary_pole(
    direction: i32,
    rank: i32,
    index: i32,
    pole: &impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<Point3, Failure> {
    if direction == 1 {
        pole(rank, index)
    } else {
        pole(index, rank)
    }
}

#[derive(Default)]
struct PoleBounds {
    minimum: Point3,
    maximum: Point3,
    initialized: bool,
}

impl PoleBounds {
    fn include(&mut self, point: Point3) {
        if !self.initialized {
            self.minimum = point;
            self.maximum = point;
            self.initialized = true;
            return;
        }
        self.minimum.set_coordinates(
            min_native(self.minimum.x(), point.x()),
            min_native(self.minimum.y(), point.y()),
            min_native(self.minimum.z(), point.z()),
        );
        self.maximum.set_coordinates(
            max_native(self.maximum.x(), point.x()),
            max_native(self.maximum.y(), point.y()),
            max_native(self.maximum.z(), point.z()),
        );
    }

    fn is_small(&self, tolerance: f64) -> bool {
        axis_is_small(self.maximum.x() - self.minimum.x(), tolerance)
            && axis_is_small(self.maximum.y() - self.minimum.y(), tolerance)
            && axis_is_small(self.maximum.z() - self.minimum.z(), tolerance)
    }
}

fn has_identical_boundary_poles(
    u_count: i32,
    v_count: i32,
    pole: &impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<bool, Failure> {
    for (direction, rank) in [(2, v_count), (2, 1), (1, u_count), (1, 1)] {
        if boundary_has_equal_neighbors(direction, rank, u_count, v_count, pole)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn boundary_has_equal_neighbors(
    direction: i32,
    rank: i32,
    u_count: i32,
    v_count: i32,
    pole: &impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<bool, Failure> {
    let count = if direction == 1 { v_count } else { u_count };
    for index in 1..count {
        let first = boundary_pole(direction, rank, index, pole)?;
        let second = boundary_pole(direction, rank, index + 1, pole)?;
        if first.distance(&second) <= POLE_EQUALITY_TOLERANCE {
            return Ok(true);
        }
    }
    Ok(false)
}

// MinMaxPnt: `if (minx > x) minx = x;` and `if (maxx < x) maxx = x;`.
fn min_native(current: f64, point: f64) -> f64 {
    if current > point { point } else { current }
}

fn max_native(current: f64, point: f64) -> f64 {
    if current < point { point } else { current }
}
