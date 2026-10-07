//! The planes and tangents of edges and wires used by `BOPAlgo_Tools::EdgesToWires` and
//! `WiresToFaces` (the static `FindPlane` and `FindEdgeTangent` of BOPAlgo_Tools.cxx).
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction3, Plane3, Vector3};

/// The tangent direction of each edge (`BOPAlgo_IndexedDataMapOfShapeDir`).
pub(super) type EdgeTangents = IndexedDataMap<Shape, Direction3, ShapeMapHasher>;

/// The number of samples of the curve searches.
const SAMPLES: i32 = 11;

/// The tangent of a 3D curve: the line direction, or the first first-derivative longer than
/// the confusion at 10 regular steps (`FindEdgeTangent(Curve, Tangent)`).
pub(super) fn curve_tangent(curve: &BRepAdaptorCurve) -> Result<Option<Vector3>, Failure> {
    if !curve.transformed_curve().is_3d_curve() {
        return Ok(None);
    }
    if curve.get_type()? == CurveType::Line {
        return Ok(Some(Vector3::from(curve.line()?.position().direction())));
    }
    let (first, last) = (curve.first_parameter()?, curve.last_parameter()?);
    let step = (last - first) / f64::from(SAMPLES);
    let mut parameter = first + step;
    while parameter <= last {
        let (_, tangent) = curve.d1(parameter)?;
        if tangent.magnitude() > precision::confusion() {
            return Ok(Some(tangent));
        }
        parameter += step;
    }
    Ok(None)
}

/// The tangent of an edge, computed once and kept in `tangents` (`FindEdgeTangent(Edge, Map,
/// Dir)`).
pub(super) fn edge_tangent(
    edge: &Edge,
    tangents: &mut EdgeTangents,
) -> Result<Option<Direction3>, Failure> {
    if let Some(tangent) = tangents.seek(edge) {
        return Ok(Some(*tangent));
    }
    let Some(tangent) = curve_tangent(&BRepAdaptorCurve::with_edge(edge)?)? else {
        return Ok(None);
    };
    let direction = Direction3::from_vector(tangent)?;
    tangents.add(edge.as_ref().clone(), direction);
    Ok(Some(direction))
}

/// The plane of a 3D curve: none for a line, the axis of a conic, or the cross product of
/// the first derivative with the first non-parallel later one (`FindPlane(Curve, Plane)`).
pub(super) fn curve_plane(curve: &BRepAdaptorCurve) -> Result<Option<Plane3>, Failure> {
    if !curve.transformed_curve().is_3d_curve() {
        return Ok(None);
    }
    let normal = match curve.get_type()? {
        CurveType::Line => return Ok(None),
        CurveType::Circle => Some(Vector3::from(curve.circle()?.position().direction())),
        CurveType::Ellipse => Some(Vector3::from(curve.ellipse()?.position().direction())),
        CurveType::Hyperbola => Some(Vector3::from(curve.hyperbola()?.position().direction())),
        CurveType::Parabola => Some(Vector3::from(curve.parabola()?.position().direction())),
        _ => sampled_normal(curve)?,
    };
    let Some(normal) = normal else {
        return Ok(None);
    };
    let origin = curve.value(curve.first_parameter()?)?;
    Ok(Some(Plane3::new(origin, Direction3::from_vector(normal)?)?))
}

/// The cross product of the first derivative with the first later derivative giving a
/// vector longer than the confusion.
fn sampled_normal(curve: &BRepAdaptorCurve) -> Result<Option<Vector3>, Failure> {
    let (first, last) = (curve.first_parameter()?, curve.last_parameter()?);
    let step = (last - first) / f64::from(SAMPLES);
    let (_, first_tangent) = curve.d1(first)?;
    let mut parameter = first + step;
    while parameter <= last {
        let (_, tangent) = curve.d1(parameter)?;
        let normal = first_tangent.crossed(tangent);
        if normal.magnitude() > precision::confusion() {
            return Ok(Some(normal));
        }
        parameter += step;
    }
    Ok(None)
}

/// The plane of a wire: from two non-parallel edge tangents at a vertex of the first edge,
/// else from a single edge whose plane is defined; the edges without one are noted in
/// `no_unique_plane` (`FindPlane(Wire, Plane, Map, Map)`).
pub(super) fn wire_plane(
    wire: &Shape,
    tangents: &mut EdgeTangents,
    no_unique_plane: &mut MapOfShape,
) -> Result<Option<Plane3>, Failure> {
    let edges = wire_edges(wire)?;
    if edges.is_empty() {
        return Ok(None);
    }
    if let Some(plane) = plane_of_two_edges(&edges, tangents)? {
        return Ok(Some(plane));
    }
    for edge in &edges {
        if no_unique_plane.contains(edge.as_ref()) {
            continue;
        }
        if let Some(plane) = curve_plane(&BRepAdaptorCurve::with_edge(edge)?)? {
            return Ok(Some(plane));
        }
        no_unique_plane.add(edge.as_ref().clone());
    }
    Ok(None)
}

/// The plane through the first vertex of an edge, normal to its tangent and the tangent of
/// another, non-parallel edge.
fn plane_of_two_edges(
    edges: &[Edge],
    tangents: &mut EdgeTangents,
) -> Result<Option<Plane3>, Failure> {
    for first in edges {
        let Some(first_tangent) = edge_tangent(first, tangents)? else {
            continue;
        };
        for second in edges {
            if first.is_same(second) {
                continue;
            }
            let Some(second_tangent) = edge_tangent(second, tangents)? else {
                continue;
            };
            if first_tangent.is_parallel(second_tangent, precision::angular()) {
                continue;
            }
            let normal = first_tangent.crossed(second_tangent)?;
            let vertex = Iterator::next(&mut ShapeIterator::new(first, true, true))
                .ok_or_else(|| no_vertex(first))?;
            return Ok(Some(Plane3::new(
                BRepTool::pnt(&topods::vertex(&vertex)?)?,
                normal,
            )?));
        }
    }
    Ok(None)
}

/// The edges of a wire or a compound of edges in explorer order.
pub(super) fn wire_edges(wire: &Shape) -> Result<Vec<Edge>, Failure> {
    let mut explorer = Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let mut edges = Vec::new();
    while explorer.more() {
        edges.push(topods::edge(explorer.current())?);
        explorer.next()?;
    }
    Ok(edges)
}

/// The failure of an edge without vertex, which the native code would dereference.
fn no_vertex(edge: &Edge) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!(
            "BOPAlgo_Tools::FindPlane: edge {:?} has no vertex; expected an edge with vertices",
            edge.as_ref().shape_type()
        ),
        "",
    )
}
