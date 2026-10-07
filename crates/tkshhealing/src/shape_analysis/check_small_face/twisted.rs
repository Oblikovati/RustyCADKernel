use super::spot::ShapeAnalysisCheckSmallFace;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomElementarySurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::Vector3;

const GRID_INTERVALS: usize = 5;

impl ShapeAnalysisCheckSmallFace {
    /// Finds a 90-degree normal reversal across the native 5-by-5 support samples;
    /// `analyzer.check_twisted(&face, &mut u, &mut v)?`.
    pub fn check_twisted(
        &mut self,
        face: &Face,
        parameter_u: &mut f64,
        parameter_v: &mut f64,
    ) -> Result<bool, Failure> {
        let surface = BRepTool::surface_with_location(face)?.0;
        if is_elementary(&surface) {
            return Ok(false);
        }
        // The native code reads myPrecision into a local it never uses, so an unset
        // tolerance does not change the result.
        let Some(parameters) = locate_twist(&surface)? else {
            return Ok(false);
        };
        *parameter_u = parameters.0;
        *parameter_v = parameters.1;
        self.twisted_status = ShapeExtend::encode_status(ShapeExtendStatus::Done);
        Ok(true)
    }
}

fn locate_twist(
    surface: &rustycad_tkernel::standard::Handle<dyn GeomSurface>,
) -> Result<Option<(f64, f64)>, Failure> {
    let bounds = surface.bounds();
    // The native normals come from GeomAdaptor_Surface::D1, whose B-spline and Bezier
    // evaluation goes through the span cache.
    let adaptor = GeomAdaptorSurface::with_surface(surface)?;
    let grid = sample_normal_grid(&adaptor, bounds)?;
    Ok(first_reversing_pair(&grid).map(|indices| twisted_parameters(bounds, indices)))
}

fn is_elementary(surface: &rustycad_tkernel::standard::Handle<dyn GeomSurface>) -> bool {
    surface.cast::<dyn GeomElementarySurface>().get().is_some()
}

fn sample_normal_grid(
    surface: &GeomAdaptorSurface,
    bounds: (f64, f64, f64, f64),
) -> Result<Vec<Vec<Vector3>>, Failure> {
    let (u_min, u_max, v_min, _) = bounds;
    let u_step = (u_max - u_min) / GRID_INTERVALS as f64;
    let v_step = (u_max - u_min) / GRID_INTERVALS as f64;
    collect_normal_rows(surface, (u_min, v_min), (u_step, v_step))
}

fn collect_normal_rows(
    surface: &GeomAdaptorSurface,
    start: (f64, f64),
    step: (f64, f64),
) -> Result<Vec<Vec<Vector3>>, Failure> {
    let mut rows = Vec::with_capacity(GRID_INTERVALS);
    let mut u = start.0;
    for _ in 0..GRID_INTERVALS {
        rows.push(collect_normal_row(surface, u, start.1, step.1)?);
        u += step.0;
    }
    Ok(rows)
}

fn collect_normal_row(
    surface: &GeomAdaptorSurface,
    u: f64,
    v_start: f64,
    v_step: f64,
) -> Result<Vec<Vector3>, Failure> {
    let mut normals = Vec::with_capacity(GRID_INTERVALS);
    let mut v = v_start;
    for _ in 0..GRID_INTERVALS {
        normals.push(sample_normal(surface, u, v)?);
        v += v_step;
    }
    Ok(normals)
}

fn sample_normal(surface: &GeomAdaptorSurface, u: f64, v: f64) -> Result<Vector3, Failure> {
    let derivatives = surface.d1(u, v)?;
    Ok(derivatives.d1u.crossed(derivatives.d1v))
}

fn first_reversing_pair(grid: &[Vec<Vector3>]) -> Option<(usize, usize)> {
    for u in 0..GRID_INTERVALS - 1 {
        for v in 0..GRID_INTERVALS - 1 {
            if pair_reverses(grid, u, v) {
                return Some((u, v));
            }
        }
    }
    None
}

fn pair_reverses(grid: &[Vec<Vector3>], u: usize, v: usize) -> bool {
    let normal = grid[u][v];
    twisted_norm(normal, grid[u][v + 1]) < 0.0 || twisted_norm(normal, grid[u + 1][v]) < 0.0
}

// TwistedNorm: `(x1 * x2) + (y1 * y2) + (z1 * z2)`.
fn twisted_norm(first: Vector3, second: Vector3) -> f64 {
    (first.x() * second.x()) + (first.y() * second.y()) + (first.z() * second.z())
}

fn twisted_parameters(bounds: (f64, f64, f64, f64), indices: (usize, usize)) -> (f64, f64) {
    let (u_min, u_max, v_min, _) = bounds;
    let u_step = (u_max - u_min) / GRID_INTERVALS as f64;
    let v_step = (u_max - u_min) / GRID_INTERVALS as f64;
    let u = u_min + u_step * (indices.0 + 1) as f64 - u_step / 2.0;
    let v = v_min + v_step * (indices.1 + 1) as f64 - v_step / 2.0;
    (u, v)
}
