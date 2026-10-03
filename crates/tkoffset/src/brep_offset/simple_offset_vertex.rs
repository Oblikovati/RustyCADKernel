// Translation of BRepOffset_SimpleOffset::FillVertexData at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::OffsetEdgeGeometry;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Point3;

/// One incident source edge and its optional result from the edge-offset map.
#[derive(Clone)]
pub struct OffsetVertexIncidentEdge {
    /// The source edge whose endpoint contributes to this vertex.
    pub edge: Edge,
    /// New edge curve and tolerance, absent when the edge map has no entry.
    pub geometry: Option<OffsetEdgeGeometry>,
}

/// The averaged point and tolerance assigned to a vertex by simple offset.
#[derive(Clone, Copy)]
pub struct OffsetVertexGeometry {
    /// The centroid of the incident offset-curve endpoints.
    pub point: Point3,
    /// The largest incident edge tolerance or centroid deviation, scaled by 1.001.
    pub tolerance: f64,
}

/// Computes the private `FillVertexData` result from incident edge-map entries.
/// Returns `None` when none contributes a non-null 3D offset curve. Native empty-sample
/// behavior is unprobed and remains unsupported by this helper.
///
/// ```
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkoffset::brep_offset::build_simple_offset_vertex;
/// let mut vertex = Vertex::default();
/// BRepBuilder::default()
///     .make_vertex_at(&mut vertex, Point3::new(1.0, 2.0, 3.0), 1.0e-7)
///     .unwrap();
/// assert!(build_simple_offset_vertex(&vertex, &[]).unwrap().is_none());
/// ```
pub fn build_simple_offset_vertex(
    vertex: &Vertex,
    incident_edges: &[OffsetVertexIncidentEdge],
) -> Result<Option<OffsetVertexGeometry>, Failure> {
    let source_point = BRepTool::pnt(vertex)?;
    let mut samples: Vec<Point3> = Vec::new();
    let mut max_edge_tolerance: f64 = 0.0;
    for incident in incident_edges {
        collect_incident_samples(
            incident,
            &source_point,
            &mut samples,
            &mut max_edge_tolerance,
        )?;
    }
    if samples.is_empty() {
        return Ok(None);
    }
    let point = centroid(&samples);
    let deviation = farthest_sample_distance(&point, &samples);
    let tolerance = max_edge_tolerance.max(deviation) * 1.001;
    Ok(Some(OffsetVertexGeometry { point, tolerance }))
}

fn collect_incident_samples(
    incident: &OffsetVertexIncidentEdge,
    source_point: &Point3,
    samples: &mut Vec<Point3>,
    max_edge_tolerance: &mut f64,
) -> Result<(), Failure> {
    let Some(geometry) = incident.geometry.as_ref() else {
        return Ok(());
    };
    let Some(offset_curve) = geometry.curve.get() else {
        return Ok(());
    };
    let (source_curve, first, last) = BRepTool::curve(&incident.edge)?;
    let Some(source_curve) = source_curve.get() else {
        return Ok(());
    };
    append_closest_offset_ends(
        &incident.edge,
        source_point,
        source_curve,
        (first, last),
        offset_curve,
        samples,
    )?;
    *max_edge_tolerance = max_edge_tolerance.max(geometry.tolerance);
    Ok(())
}

fn append_closest_offset_ends(
    edge: &Edge,
    source_point: &Point3,
    source_curve: &dyn GeomCurve,
    (first, last): (f64, f64),
    offset_curve: &dyn GeomCurve,
    samples: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let (source_first, source_last) = (source_curve.value(first)?, source_curve.value(last)?);
    let (near, far) =
        if source_last.square_distance(source_point) < source_first.square_distance(source_point) {
            (last, first)
        } else {
            (first, last)
        };
    samples.push(offset_curve.value(near)?);
    append_closed_edge_other_end(edge, offset_curve, far, samples)?;
    Ok(())
}

fn append_closed_edge_other_end(
    edge: &Edge,
    offset_curve: &dyn GeomCurve,
    far_parameter: f64,
    samples: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let (first_vertex, last_vertex) = topexp::vertices(edge, false);
    if first_vertex.is_same(&last_vertex) {
        samples.push(offset_curve.value(far_parameter)?);
    }
    Ok(())
}

fn centroid(samples: &[Point3]) -> Point3 {
    let count = samples.len() as f64;
    let (x, y, z) = samples.iter().fold((0.0, 0.0, 0.0), |sum, point| {
        (sum.0 + point.x(), sum.1 + point.y(), sum.2 + point.z())
    });
    Point3::new(x / count, y / count, z / count)
}

fn farthest_sample_distance(center: &Point3, samples: &[Point3]) -> f64 {
    samples
        .iter()
        .map(|point| center.square_distance(point).sqrt())
        .fold(0.0, f64::max)
}
