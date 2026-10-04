use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Line2, Point2, Vector2};

// The native `UnionPCurves` also joins seam, multi-support, circle, and general curves.
// Keep this helper limited to its proven one-face non-seam line branch; the other branches
// remain open until their sequence and support-location semantics are translated.

/// Mirrors the native same-support line branch of `UnionPCurves` for one non-seam face.
pub(super) fn union_single_face_line(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    merged: &Edge,
) -> Result<(), Failure> {
    let Some(face) = single_nonplanar_support(shape, first, second)? else {
        return Ok(());
    };
    union_line_on_face(&face, first, second, merged)
}

fn single_nonplanar_support(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
) -> Result<Option<Face>, Failure> {
    let mut supports = Vec::new();
    for candidate in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = rustycad_tkbrep::topods::face(&candidate)?;
        if !contains_edge(&face, first)? || !contains_edge(&face, second)? {
            continue;
        }
        if BRepAdaptorSurface::with_face(&face, false)?.get_type()? == SurfaceType::Plane {
            continue;
        }
        supports.push(face);
    }
    Ok((supports.len() == 1).then(|| supports.remove(0)))
}

fn contains_edge(face: &Face, edge: &Edge) -> Result<bool, Failure> {
    for candidate in Explorer::new(&face.clone().into_shape(), ShapeEnum::Edge, ShapeEnum::Face)? {
        if rustycad_tkbrep::topods::edge(&candidate)?.is_same(edge) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn union_line_on_face(
    face: &Face,
    first: &Edge,
    second: &Edge,
    merged: &Edge,
) -> Result<(), Failure> {
    if is_seam_pair(first, second, face)? {
        return Ok(());
    }
    let first = stored_oriented_pcurve(first, face)?;
    let second = stored_oriented_pcurve(second, face)?;
    let Some((first_line, _second_line)) = line_pair(&first, &second)? else {
        return Ok(());
    };
    let Some(extension) = joined_line_range(&second, first_line)? else {
        return Ok(());
    };
    let range = extend_native_range(&first, extension, first.edge_orientation);
    store_reparameterized_line(face, merged, first_line, range)
}

fn is_seam_pair(first: &Edge, second: &Edge, face: &Face) -> Result<bool, Failure> {
    Ok(BRepTool::is_closed_on_face(first, face)? || BRepTool::is_closed_on_face(second, face)?)
}

struct OrientedPCurve {
    edge_orientation: Orientation,
    pcurve: rustycad_tkbrep::brep::EdgePCurve,
}

fn stored_oriented_pcurve(edge: &Edge, face: &Face) -> Result<OrientedPCurve, Failure> {
    let mut support = face.clone();
    support.set_orientation(Orientation::Forward);
    Ok(OrientedPCurve {
        edge_orientation: edge.orientation(),
        pcurve: BRepTool::curve_on_face(edge, &support)?,
    })
}

fn line_pair(
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<Option<(Line2, Line2)>, Failure> {
    if !first.pcurve.is_stored
        || !second.pcurve.is_stored
        || first.pcurve.pcurve.is_null()
        || second.pcurve.pcurve.is_null()
    {
        return Ok(None);
    }
    let first_adaptor = Geom2dAdaptorCurve::with_curve(&first.pcurve.pcurve)?;
    let second_adaptor = Geom2dAdaptorCurve::with_curve(&second.pcurve.pcurve)?;
    if first_adaptor.get_type()? != rustycad_tkmath::geom_abs::CurveType::Line
        || second_adaptor.get_type()? != rustycad_tkmath::geom_abs::CurveType::Line
    {
        return Ok(None);
    }
    Ok(Some((first_adaptor.line()?, second_adaptor.line()?)))
}

fn joined_line_range(second: &OrientedPCurve, line: Line2) -> Result<Option<(f64, f64)>, Failure> {
    let endpoints = line_endpoints(second)?;
    if endpoints
        .iter()
        .any(|point| !line.contains(*point, precision::confusion()))
    {
        return Ok(None);
    }
    let parameters = endpoints.map(|point| line_parameter(line, point));
    let extension = (
        parameters[0].min(parameters[1]),
        parameters[0].max(parameters[1]),
    );
    Ok(Some(extension))
}

fn line_endpoints(curve: &OrientedPCurve) -> Result<[Point2; 2], Failure> {
    Ok([
        curve.pcurve.pcurve.eval_d0(curve.pcurve.first)?,
        curve.pcurve.pcurve.eval_d0(curve.pcurve.last)?,
    ])
}

fn extend_native_range(
    first: &OrientedPCurve,
    extension: (f64, f64),
    orientation: Orientation,
) -> (f64, f64) {
    if orientation != Orientation::Reversed {
        return (first.pcurve.first, first.pcurve.last.max(extension.1));
    }
    (first.pcurve.first.min(extension.0), first.pcurve.last)
}

fn line_parameter(line: Line2, point: Point2) -> f64 {
    Vector2::between(line.location(), point)
        .dot(Vector2::new(line.direction().x(), line.direction().y()))
}

fn store_reparameterized_line(
    face: &Face,
    merged: &Edge,
    line: Line2,
    range: (f64, f64),
) -> Result<(), Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let (first_3d, last_3d) = BRepTool::range(merged)?;
    let pcurve = reparameterized_line(line, range, (first_3d, last_3d));
    let curve: Handle<dyn Geom2dCurve> = Handle::new(pcurve).cast();
    let tolerance = BRepTool::edge_tolerance(merged)?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve(merged, &curve, &surface, &location, tolerance, None)?;
    builder.range_on_surface(merged, &surface, &location, first_3d, last_3d)
}

fn reparameterized_line(line: Line2, range: (f64, f64), edge_range: (f64, f64)) -> Geom2dLine {
    let start = line.location().translated(Vector2::new(
        line.direction().x() * (range.0 - edge_range.0),
        line.direction().y() * (range.0 - edge_range.0),
    ));
    Geom2dLine::from_lin2d(Line2::new(start, line.direction()))
}
