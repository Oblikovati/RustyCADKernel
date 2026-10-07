//! `FindCommonFace`, `FindClosestPoints` and `ReconstructMissedSeam` of
//! ShapeUpgrade_UnifySameDomain.cxx.
use super::face_bounds::edges_of;
use super::geometry::null_pcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

/// `FindCommonFace(theEdge1, theEdge2, theVFmap, ...)`: a face (taken forward) at the
/// common vertex containing both edges, with their orientations in it.
pub(super) fn find_common_face(
    edge1: &Edge,
    edge2: &Edge,
    vf_map: &IndexedDataMapOfShapeListOfShape,
) -> Result<Option<(Face, Orientation, Orientation)>, Failure> {
    let vertex = topexp::common_vertex(edge1, edge2).unwrap_or_default();
    for shape in vf_map.find_from_key(vertex.as_ref())?.iter() {
        let face = topods::face(&shape.oriented(Orientation::Forward))?;
        let (mut or1, mut or2) = (None, None);
        for edge in edges_of(face.as_ref())? {
            if edge.is_same(edge1) {
                or1 = Some(edge.orientation());
            }
            if edge.is_same(edge2) {
                or2 = Some(edge.orientation());
            }
            if let (Some(o1), Some(o2)) = (or1, or2) {
                return Ok(Some((face, o1, o2)));
            }
        }
    }
    Ok(None)
}

/// The output of `FindClosestPoints`.
pub(super) struct ClosestPoints {
    pub face: Face,
    pub min_sq_dist: f64,
    pub or1: Orientation,
    pub ind1: usize,
    pub ind2: usize,
    pub points1: [Point2; 2],
    pub points2: [Point2; 2],
}

/// The end points of the p-curve of the edge on the face.
fn pcurve_ends(edge: &Edge, face: &Face) -> Result<Option<[Point2; 2]>, Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    let Some(curve) = found.pcurve.get() else {
        return Ok(None);
    };
    Ok(Some([curve.value(found.first)?, curve.value(found.last)?]))
}

/// `FindClosestPoints(...)`: on the common face, the closest pair of 2D end points of
/// the two edges (the first minimum in the order 00, 01, 10, 11). The orientation of the
/// second edge on the face is an unused native output.
pub(super) fn find_closest_points(
    edge1: &Edge,
    edge2: &Edge,
    vf_map: &IndexedDataMapOfShapeListOfShape,
) -> Result<Option<ClosestPoints>, Failure> {
    let Some((face, or1, _)) = find_common_face(edge1, edge2, vf_map)? else {
        return Ok(None);
    };
    let first = pcurve_ends(edge1, &face)?;
    let second = pcurve_ends(edge2, &face)?;
    let (Some(points1), Some(points2)) = (first, second) else {
        return Ok(None);
    };
    let mut result = ClosestPoints {
        face,
        min_sq_dist: f64::MAX,
        or1,
        ind1: usize::MAX,
        ind2: usize::MAX,
        points1,
        points2,
    };
    for (ind1, p1) in points1.iter().enumerate() {
        for (ind2, p2) in points2.iter().enumerate() {
            let sq_dist = p1.square_distance(*p2);
            if sq_dist < result.min_sq_dist {
                (result.min_sq_dist, result.ind1, result.ind2) = (sq_dist, ind1, ind2);
            }
        }
    }
    Ok(Some(result))
}

/// The 2D distances between `point` and the start of the p-curve of the edge.
fn start_offsets(edge: &Edge, face: &Face, point: Point2) -> Result<Option<(f64, f64)>, Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    let Some(curve) = found.pcurve.get() else {
        return Ok(None);
    };
    let parameter = if edge.orientation() == Orientation::Forward {
        found.first
    } else {
        found.last
    };
    let start = curve.value(parameter)?;
    Ok(Some((
        (start.x() - point.x()).abs(),
        (start.y() - point.y()).abs(),
    )))
}

/// The periods of the reference face (0 when not closed in that direction).
#[derive(Clone, Copy)]
pub(super) struct Periods {
    pub u: f64,
    pub v: f64,
}

impl Periods {
    pub(super) fn is_far(self, (du, dv): (f64, f64)) -> bool {
        (self.u != 0. && du > self.u / 2.) || (self.v != 0. && dv > self.v / 2.)
    }

    pub(super) fn is_near(self, (du, dv): (f64, f64)) -> bool {
        (self.u == 0. || du < self.u / 2.) && (self.v == 0. || dv < self.v / 2.)
    }
}

/// The two p-curves of a seam exchanged on the reference surface (the edge modified in
/// place, as natively).
fn swap_seam_pcurves(edge: &mut Edge, face: &Face) -> Result<(), Failure> {
    let orientation = edge.orientation();
    edge.set_orientation(Orientation::Forward);
    let first = BRepTool::curve_on_face(edge, face)?.pcurve;
    edge.reverse();
    let second = BRepTool::curve_on_face(edge, face)?.pcurve;
    edge.reverse();
    let tolerance = BRepTool::edge_tolerance(edge)?;
    let (surface, location) = BRepTool::surface_with_location(face)?;
    BRepBuilder::default().update_edge_pcurves(
        edge,
        (&second, &first),
        &surface,
        &location,
        tolerance,
        None,
    )?;
    edge.set_orientation(orientation);
    Ok(())
}

/// The seam candidate `edge` oriented (or its p-curves exchanged) to start at the
/// current point; None when it does not.
fn oriented_seam(
    mut edge: Edge,
    (face, vertex, point): (&Face, &Vertex, Point2),
    periods: Periods,
    offsets: (f64, f64),
) -> Result<Option<Edge>, Failure> {
    let mut offsets = offsets;
    if periods.is_far(offsets) {
        let (_, last) = topexp::vertices(&edge, true);
        if last.is_same(vertex) || (periods.u != 0. && periods.v != 0.) {
            edge.reverse();
        } else {
            swap_seam_pcurves(&mut edge, face)?;
        }
        offsets = start_offsets(&edge, face, point)?.ok_or_else(null_pcurve)?;
    }
    if periods.is_near(offsets) && topexp::first_vertex(&edge, true).is_same(vertex) {
        return Ok(Some(edge));
    }
    Ok(None)
}

/// `ReconstructMissedSeam(theRemovedEdges, theFrefFace, theCurEdge, theCurVertex,
/// theCurPoint, theUperiod, theVperiod, theSeamEdge, theNextPoint)`: a removed seam edge
/// of the reference face at the current vertex, oriented to start at the current point,
/// with the 2D point at its end.
pub(super) fn reconstruct_missed_seam(
    removed: &[Shape],
    ref_face: &Face,
    (cur_edge, cur_vertex, cur_point): (&Edge, &Vertex, Point2),
    periods: Periods,
) -> Result<Option<(Edge, Point2)>, Failure> {
    for shape in removed {
        let edge = topods::edge(shape)?;
        if edge.is_same(cur_edge) {
            continue;
        }
        if BRepTool::curve_on_face(&edge, ref_face)?.pcurve.is_null() {
            continue;
        }
        let (first, last) = topexp::vertices(&edge, true);
        let at_vertex = first.is_same(cur_vertex) || last.is_same(cur_vertex);
        if !(at_vertex && BRepTool::is_closed_on_face(&edge, ref_face)?) {
            continue;
        }
        let offsets = start_offsets(&edge, ref_face, cur_point)?.ok_or_else(null_pcurve)?;
        if let Some(seam) =
            oriented_seam(edge, (ref_face, cur_vertex, cur_point), periods, offsets)?
        {
            return Ok(Some(seam_end(seam, ref_face)?));
        }
    }
    Ok(None)
}

fn seam_end(seam: Edge, face: &Face) -> Result<(Edge, Point2), Failure> {
    let found = BRepTool::curve_on_face(&seam, face)?;
    let parameter = if seam.orientation() == Orientation::Forward {
        found.last
    } else {
        found.first
    };
    let point = found
        .pcurve
        .get()
        .ok_or_else(null_pcurve)?
        .value(parameter)?;
    Ok((seam, point))
}
