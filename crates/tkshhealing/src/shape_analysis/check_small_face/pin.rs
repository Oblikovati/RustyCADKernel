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
    /// Detects a pin on a Bezier or B-spline support and updates row/sense only for supported surfaces;
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
        let tolerance = self.pin_precision()?;
        let result = check_surface_poles(&surface, tolerance)?;
        let Some(result) = result else {
            return Ok(false);
        };
        *row = result.row;
        *sense = result.sense;
        self.pin_status = ShapeExtend::encode_status(result.status);
        Ok(true)
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

struct PinResult {
    row: i32,
    sense: i32,
    status: ShapeExtendStatus,
}

fn check_surface_poles(
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
) -> Result<Option<PinResult>, Failure> {
    let bspline = surface.cast::<GeomBSplineSurface>();
    if let Some(surface) = bspline.get() {
        return inspect_poles(
            surface.nb_u_poles(),
            surface.nb_v_poles(),
            tolerance,
            |u, v| surface.pole(u, v),
        );
    }
    let bezier = surface.cast::<GeomBezierSurface>();
    let Some(surface) = bezier.get() else {
        return Ok(None);
    };
    inspect_poles(
        surface.nb_u_poles(),
        surface.nb_v_poles(),
        tolerance,
        |u, v| surface.pole(u, v),
    )
}

fn inspect_poles(
    u_count: i32,
    v_count: i32,
    tolerance: f64,
    pole: impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<Option<PinResult>, Failure> {
    if u_count == 0 || v_count == 0 {
        return Ok(None);
    }
    let (sense, row, status) = inspect_bounds(u_count, v_count, tolerance, &pole)?;
    if sense == 0 {
        return Ok(None);
    }
    let mut pin_status = match status {
        1 => ShapeExtendStatus::Done1,
        2 => ShapeExtendStatus::Done2,
        _ => ShapeExtendStatus::Ok,
    };
    if status == 1 && has_identical_boundary_poles(u_count, v_count, &pole)? {
        pin_status = ShapeExtendStatus::Done3;
    }
    Ok(Some(PinResult {
        row,
        sense,
        status: pin_status,
    }))
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
    let count = if direction == 1 { v_count } else { u_count };
    let mut bounds = PoleBounds::default();
    for index in 1..=count {
        let point = if direction == 1 {
            pole(rank, index)?
        } else {
            pole(index, rank)?
        };
        bounds.include(point);
    }
    if bounds.is_small(PIN_TOLERANCE) {
        return Ok(0);
    }
    Ok(if bounds.is_small(tolerance) { 1 } else { 2 })
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
        let (first, second) = if direction == 1 {
            (pole(rank, index)?, pole(rank, index + 1)?)
        } else {
            (pole(index, rank)?, pole(index + 1, rank)?)
        };
        if first.distance(&second) <= POLE_EQUALITY_TOLERANCE {
            return Ok(true);
        }
    }
    Ok(false)
}

fn min_native(first: f64, second: f64) -> f64 {
    if first < second { first } else { second }
}

fn max_native(first: f64, second: f64) -> f64 {
    if first > second { first } else { second }
}
