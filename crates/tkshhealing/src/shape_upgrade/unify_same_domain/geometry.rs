//! The geometric helpers of ShapeUpgrade_UnifySameDomain.cxx: `IsOnSingularity`,
//! `IsUiso`, `IsLinear`, `TrueValueOfOffset`, `UpdateBoundaries`, `TryMakeLine`,
//! `getCylinder`, `ClearRts`, `GetNormalToSurface` and `getCurveParams`.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConic, GeomCurve, GeomCylindricalSurface, GeomLine,
    GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, GeomSweptSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis3, Cylinder3, Direction2, Direction3, Line2, Point2, Vector2, Vector3,
};

/// `IsOnSingularity`: one of the edges is degenerated.
pub(super) fn is_on_singularity(edges: &[Shape]) -> Result<bool, Failure> {
    for edge in edges {
        if BRepTool::degenerated(&topods::edge(edge)?)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `IsUiso` (only for seam edges, assumed U- or V-isolines): the p-curve runs along V.
pub(super) fn is_uiso(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    let adaptor = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
    let (_, tangent) = adaptor.d1(adaptor.first_parameter()?)?;
    Ok(tangent.y().abs() > tangent.x().abs())
}

/// `IsLinear`: the direction of a line, or of the chord of a Bezier or B-spline curve
/// with 2 poles; `None` for other curves.
pub(super) fn is_linear(curve: &BRepAdaptorCurve) -> Result<Option<Direction3>, Failure> {
    let kind = curve.get_type()?;
    if kind == CurveType::Line {
        return Ok(Some(curve.line()?.position().direction()));
    }
    let spline = matches!(kind, CurveType::BezierCurve | CurveType::BSplineCurve);
    if !(spline && curve.nb_poles()? == 2) {
        return Ok(None);
    }
    let first = curve.value(curve.first_parameter()?)?;
    let last = curve.value(curve.last_parameter()?)?;
    Ok(Some(Direction3::from_vector(Vector3::between(
        first, last,
    ))?))
}

/// `TrueValueOfOffset`: the period with the sign of `value` (negative for 0).
pub(super) fn true_value_of_offset(value: f64, period: f64) -> f64 {
    if value > 0. {
        return period;
    }
    -period
}

/// `UpdateBoundaries`: the range of coordinate `index` (1 for U, 2 for V) widened by
/// the p-curve values at 5 equidistant parameters.
pub(super) fn update_boundaries(
    pcurve: &Handle<dyn Geom2dCurve>,
    (first, last): (f64, f64),
    index: i32,
    (min, max): (&mut f64, &mut f64),
) -> Result<(), Failure> {
    const NB_SAMPLES: i32 = 4;
    let curve = pcurve.get().ok_or_else(null_pcurve)?;
    let delta = (last - first) / f64::from(NB_SAMPLES);
    for i in 0..=NB_SAMPLES {
        let coord = curve.value(first + f64::from(i) * delta)?.coord(index)?;
        if coord < *min {
            *min = coord;
        }
        if coord > *max {
            *max = coord;
        }
    }
    Ok(())
}

/// `TryMakeLine`: the 2D line the p-curve is on `[first, last]` (parametrized by arc
/// length), checked at 9 inner points; `None` otherwise.
pub(super) fn try_make_line(
    pcurve: &dyn Geom2dCurve,
    first: f64,
    last: f64,
) -> Result<Option<Handle<Geom2dLine>>, Failure> {
    let (first_point, last_point) = (pcurve.value(first)?, pcurve.value(last)?);
    let chord = Vector2::between(first_point, last_point);
    let parameter_length = (last - first) * (last - first);
    if (chord.square_magnitude() - parameter_length).abs() > precision::confusion() {
        return Ok(None);
    }
    let direction = Direction2::from_vector(chord)?;
    let offset = Vector2::new(-direction.x() * first, -direction.y() * first);
    let line = Line2::new(first_point.translated(offset), direction);
    const NB_SAMPLES: i32 = 10;
    let delta = (last - first) / f64::from(NB_SAMPLES);
    for i in 1..NB_SAMPLES {
        let point = pcurve.value(first + f64::from(i) * delta)?;
        if line.distance(point) > precision::confusion() {
            return Ok(None);
        }
    }
    Ok(Some(Handle::new(Geom2dLine::from_lin2d(line))))
}

/// The basis of a chain of trimmed curves.
fn untrimmed(mut basis: Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    while let Some(trimmed) = basis.cast::<GeomTrimmedCurve>().get() {
        basis = trimmed.basis_curve();
    }
    basis
}

/// `getCylinder`: the cylinder of a cylindrical surface, of a revolution of a line
/// parallel to the axis, or of an extrusion of a circle normal to the direction.
pub(super) fn get_cylinder(
    surface: &Handle<dyn GeomSurface>,
) -> Result<Option<Cylinder3>, Failure> {
    if let Some(cylindrical) = surface.cast::<GeomCylindricalSurface>().get() {
        return Ok(Some(cylindrical.cylinder()?));
    }
    if let Some(revolution) = surface.cast::<GeomSurfaceOfRevolution>().get() {
        return revolution_cylinder(revolution);
    }
    if let Some(extrusion) = surface.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        return extrusion_cylinder(extrusion);
    }
    Ok(None)
}

fn revolution_cylinder(revolution: &GeomSurfaceOfRevolution) -> Result<Option<Cylinder3>, Failure> {
    let basis = untrimmed(revolution.basis_curve());
    let line = basis.cast::<GeomLine>();
    let Some(line) = line.get() else {
        return Ok(None);
    };
    let direction = revolution.direction();
    if !line
        .position()
        .direction()
        .is_parallel(direction, precision::angular())
    {
        return Ok(None);
    }
    // "basis line is parallel to the revolution axis: it is a cylinder".
    let location = revolution.location();
    let radius = line.lin().distance(location);
    Ok(Some(Cylinder3::new(
        Axis3::from_normal(location, direction)?,
        radius,
    )?))
}

fn extrusion_cylinder(
    extrusion: &GeomSurfaceOfLinearExtrusion,
) -> Result<Option<Cylinder3>, Failure> {
    let basis = untrimmed(extrusion.basis_curve());
    let circle = basis.cast::<GeomCircle>();
    let Some(circle) = circle.get() else {
        return Ok(None);
    };
    let direction = extrusion.direction();
    if !circle
        .position()
        .direction()
        .is_parallel(direction, precision::angular())
    {
        return Ok(None);
    }
    // "basis circle is normal to the extrusion axis: it is a cylinder".
    let frame = Axis3::from_normal(circle.location(), direction)?;
    Ok(Some(Cylinder3::new(frame, circle.radius())?))
}

/// `ClearRts`: the basis of a rectangular trimmed surface, else the surface.
pub(super) fn clear_rts(surface: &Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    match surface.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(trimmed) => trimmed.basis_surface(),
        None => surface.clone(),
    }
}

/// `GetNormalToSurface`: the normal of the face at parameter `t` of the edge (through
/// the p-curve of the edge as it is in the forward face when the edge is closed on it);
/// `None` without p-curve or for a null normal.
pub(super) fn get_normal_to_surface(
    face: &Face,
    edge: &Edge,
    t: f64,
) -> Result<Option<Direction3>, Failure> {
    let Some(pcurve) = normal_pcurve(face, edge)? else {
        return Ok(None);
    };
    let Some(curve) = pcurve.get() else {
        return Ok(None);
    };
    let uv = curve.d0(t)?;
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let surface = surface.get().ok_or_else(null_surface)?;
    let d1 = surface.d1(uv.x(), uv.y())?;
    let mut normal = d1.d1u.crossed(d1.d1v);
    if normal.magnitude() < precision::confusion() {
        return Ok(None);
    }
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    normal.transform(location.transformation());
    Ok(Some(Direction3::from_vector(normal)?))
}

/// The p-curve used by `GetNormalToSurface`; `None` when a closed edge is not found in
/// the face.
fn normal_pcurve(face: &Face, edge: &Edge) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    if !BRepTool::is_closed_on_face(edge, face)? {
        return Ok(Some(BRepTool::curve_on_face(edge, face)?.pcurve));
    }
    let forward = topods::face(&face.oriented(Orientation::Forward))?;
    for shape in Explorer::new(&forward, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if shape.is_same(edge) {
            let in_face = topods::edge(&shape)?;
            return Ok(Some(BRepTool::curve_on_face(&in_face, &forward)?.pcurve));
        }
    }
    Ok(None)
}

/// `getCurveParams`: the start and end points of the p-curve of the edge on the face,
/// swapped unless the edge is forward.
pub(super) fn get_curve_params(edge: &Edge, face: &Face) -> Result<(Point2, Point2), Failure> {
    let adaptor = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
    let (mut first, mut last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
    if edge.orientation() != Orientation::Forward {
        std::mem::swap(&mut first, &mut last);
    }
    Ok((adaptor.value(first)?, adaptor.value(last)?))
}

pub(super) fn null_pcurve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_UnifySameDomain: null p-curve; expected a 2D curve of the edge on the face",
        "",
    )
}

pub(super) fn null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_UnifySameDomain: null surface; expected the surface of the face",
        "",
    )
}
