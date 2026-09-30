// CorrectPlaneBoundaries and CorrectSurfaceBoundaries from IntTools_FaceFace.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Native domain enlargement and clipping against linear periodic seam pcurves.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::{bnd::BoundingBox2, geom_abs::SurfaceType, gp::Direction2};
pub(super) fn correct_plane(bounds: &mut [f64; 4]) {
    for axis in [0, 2] {
        if precision::is_infinite(bounds[axis]) || precision::is_infinite(bounds[axis + 1]) {
            continue;
        }
        let delta = 0.1 * (bounds[axis + 1] - bounds[axis]);
        bounds[axis] -= delta;
        bounds[axis + 1] += delta;
    }
}
pub(super) fn correct_surface(
    face: &Face,
    tolerance: f64,
    bounds: &mut [f64; 4],
) -> Result<(), Failure> {
    let surface = BRepTool::surface(face)?;
    let (u1, u2, v1, v2) = surface.bounds();
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    if nested_basis(&surface) {
        return Ok(());
    }
    let periodic = [adaptor.is_u_periodic()?, adaptor.is_v_periodic()?];
    let enlarge = enlarges(adaptor.get_type()?);
    if enlarge {
        enlarge_bounds(bounds, [u1, u2, v1, v2], periodic, tolerance);
    }
    if periodic[0] || periodic[1] {
        correct_seams(face, periodic, bounds)?;
    }
    Ok(())
}
fn nested_basis(surface: &Handle<dyn GeomSurface>) -> bool {
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    let offset = surface.cast::<GeomOffsetSurface>();
    let basis = if !trimmed.is_null() {
        trimmed.basis_surface()
    } else if !offset.is_null() {
        offset.basis_surface()
    } else {
        return false;
    };
    !basis.cast::<GeomRectangularTrimmedSurface>().is_null()
        || !basis.cast::<GeomOffsetSurface>().is_null()
}
fn enlarge_bounds(bounds: &mut [f64; 4], natural: [f64; 4], periodic: [bool; 2], delta: f64) {
    for axis in [0, 1] {
        if periodic[axis] {
            continue;
        }
        let index = axis * 2;
        bounds[index] =
            if !precision::is_infinite(bounds[index]) && bounds[index] - natural[index] > delta {
                bounds[index] - delta
            } else {
                natural[index]
            };
        bounds[index + 1] = if !precision::is_infinite(bounds[index + 1])
            && natural[index + 1] - bounds[index + 1] > delta
        {
            bounds[index + 1] + delta
        } else {
            natural[index + 1]
        };
    }
}
#[derive(Default)]
struct SeamBounds {
    rectangle: BoundingBox2,
    correct: [bool; 2],
    found: bool,
}
fn correct_seams(face: &Face, periodic: [bool; 2], bounds: &mut [f64; 4]) -> Result<(), Failure> {
    let mut seams = SeamBounds::default();
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&shape)?;
        if BRepTool::is_closed_on_face(&edge, face)? && !seams.add_edge(&edge, face)? {
            return Ok(());
        }
    }
    if !seams.found {
        return Ok(());
    }
    let [umin, vmin, umax, vmax] = seams.rectangle.get()?;
    let values = [umin, umax, vmin, vmax];
    for axis in [0, 1] {
        if periodic[axis] && seams.correct[axis] {
            clamp_axis(bounds, values, axis * 2);
        }
    }
    Ok(())
}
fn clamp_axis(bounds: &mut [f64; 4], values: [f64; 4], index: usize) {
    if bounds[index] < values[index] {
        bounds[index] = values[index];
    }
    if bounds[index + 1] > values[index + 1] {
        bounds[index + 1] = values[index + 1];
    }
}
impl SeamBounds {
    fn add_edge(&mut self, edge: &Edge, face: &Face) -> Result<bool, Failure> {
        self.found = true;
        for orientation in [Orientation::Forward, Orientation::Reversed] {
            let oriented = Edge::unchecked(edge.oriented(orientation));
            if !self.add_pcurve(&oriented, face)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn add_pcurve(&mut self, edge: &Edge, face: &Face) -> Result<bool, Failure> {
        let pcurve = BRepTool::curve_on_face(edge, face)?;
        if pcurve.pcurve.is_null() {
            return Ok(false);
        }
        let line = pcurve.pcurve.cast::<Geom2dLine>();
        if line.is_null() {
            return Ok(false);
        }
        let direction = line.direction();
        self.correct[0] =
            self.correct[0] || direction.is_parallel(Direction2::Y, precision::angular());
        self.correct[1] =
            self.correct[1] || direction.is_parallel(Direction2::X, precision::angular());
        self.rectangle
            .add_point(pcurve.pcurve.eval_d0(pcurve.first)?);
        self.rectangle
            .add_point(pcurve.pcurve.eval_d0(pcurve.last)?);
        Ok(true)
    }
}

fn enlarges(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::BezierSurface
            | SurfaceType::BSplineSurface
            | SurfaceType::SurfaceOfExtrusion
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::Cylinder
    )
}
