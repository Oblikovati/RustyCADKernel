//! `RelocatePCurvesToNewUorigin` of ShapeUpgrade_UnifySameDomain.cxx: the p-curves of the
//! boundary edges walked contour by contour from the face holding the maximal coordinate,
//! each shifted by the period so that the contour is continuous in 2D.
use super::EdgePCurveMap;
use super::face_bounds::remove_edge_from_map;
use super::geometry::{get_curve_params, null_pcurve, true_value_of_offset};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::gp::{Point2, Vector2};

/// The walk parameters: the reference face, the coordinate index and period, the 2D
/// tolerance.
#[derive(Clone, Copy)]
pub(super) struct Relocation<'a> {
    pub ref_face: &'a Face,
    pub coord_tol: f64,
    pub index: i32,
    pub period: f64,
}

/// The maps the walk updates.
pub(super) struct RelocationMaps<'a> {
    pub ve_map: &'a mut IndexedDataMapOfShapeListOfShape,
    pub edge_new_pcurve: &'a mut EdgePCurveMap,
    pub used_edges: &'a mut MapOfShape,
}

/// A trimmed p-curve with its start and end parameters in the edge direction.
type TrimmedPCurve = (Handle<dyn Geom2dCurve>, f64, f64);

/// The p-curve of the edge on the reference face trimmed to its range, with its start
/// and end parameters in the edge direction; None without p-curve.
fn trimmed_pcurve(edge: &Edge, ref_face: &Face) -> Result<Option<TrimmedPCurve>, Failure> {
    let found = BRepTool::curve_on_face(edge, ref_face)?;
    if found.pcurve.is_null() {
        return Ok(None);
    }
    let trimmed = Geom2dTrimmedCurve::new(&found.pcurve, found.first, found.last, true, true)?;
    let (mut start, mut end) = (found.first, found.last);
    if edge.orientation() == Orientation::Reversed {
        std::mem::swap(&mut start, &mut end);
    }
    Ok(Some((Handle::new(trimmed).cast(), start, end)))
}

/// The coordinate `index` of a 2D point.
fn coord(point: Point2, index: i32) -> Result<f64, Failure> {
    point.coord(index)
}

/// The start edge of the next contour: an unused edge of the first face whose start (or
/// end) point has the minimal coordinate, with the walk orientation.
fn start_edge(
    edges: &[Shape],
    first_face_shapes: &MapOfShape,
    used: &MapOfShape,
    r: Relocation<'_>,
) -> Result<Option<(Edge, Orientation)>, Failure> {
    let mut start: Option<(Edge, Orientation)> = None;
    let mut coord_min = f64::MAX;
    for shape in edges {
        if used.contains(shape) || !first_face_shapes.contains(shape) {
            continue;
        }
        let edge = topods::edge(shape)?;
        let (first, last) = get_curve_params(&edge, r.ref_face)?;
        let (first, last) = (coord(first, r.index)?, coord(last, r.index)?);
        if start.is_none() {
            let forward = first < last;
            coord_min = if forward { first } else { last };
            let orientation = if forward {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            start = Some((edge, orientation));
            continue;
        }
        if first < coord_min {
            coord_min = first;
            start = Some((edge.clone(), Orientation::Forward));
        }
        if last < coord_min {
            coord_min = last;
            start = Some((edge, Orientation::Reversed));
        }
    }
    Ok(start)
}

/// `RelocatePCurvesToNewUorigin(...)`: false when an edge of a contour has no p-curve.
pub(super) fn relocate_pcurves_to_new_uorigin(
    edges: &[Shape],
    first_face: &Shape,
    r: Relocation<'_>,
    maps: RelocationMaps<'_>,
) -> Result<bool, Failure> {
    let mut first_face_shapes = MapOfShape::default();
    topexp::map_shapes_into_map(first_face, &mut first_face_shapes, true, true);
    loop {
        let Some((start, orientation)) = start_edge(edges, &first_face_shapes, maps.used_edges, r)?
        else {
            return Ok(true);
        };
        if !walk_contour(
            start,
            orientation,
            r,
            &mut *maps.ve_map,
            &mut *maps.edge_new_pcurve,
            &mut *maps.used_edges,
        )? {
            return Ok(false);
        }
    }
}

/// The current point of the walk.
struct WalkPoint {
    edge: Edge,
    point: Point2,
}

/// Collects the p-curves of one contour from `start`; false when one is missing.
fn walk_contour(
    start: Edge,
    orientation: Orientation,
    r: Relocation<'_>,
    ve_map: &mut IndexedDataMapOfShapeListOfShape,
    edge_new_pcurve: &mut EdgePCurveMap,
    used: &mut MapOfShape,
) -> Result<bool, Failure> {
    let Some((pcurve, first, last)) = trimmed_pcurve(&start, r.ref_face)? else {
        return Ok(false);
    };
    edge_new_pcurve.bind(start.clone().into_shape(), pcurve.clone());
    let parameter = if orientation == Orientation::Forward {
        last
    } else {
        first
    };
    let point = pcurve.get().ok_or_else(null_pcurve)?.value(parameter)?;
    let mut current = WalkPoint { edge: start, point };
    loop {
        if !remove_edge_from_map(current.edge.as_ref(), ve_map)? {
            return Ok(true); // "end of contour in 2d"
        }
        used.add(current.edge.clone().into_shape());
        let vertex = if orientation == Orientation::Forward {
            topexp::last_vertex(&current.edge, true)
        } else {
            topexp::first_vertex(&current.edge, true)
        };
        let list: Vec<Shape> = ve_map
            .find_from_key(vertex.as_ref())?
            .iter()
            .cloned()
            .collect();
        if list.is_empty() {
            return Ok(true); // "end of contour in 3d"
        }
        match next_edge(
            &list,
            &current,
            &vertex.into_shape(),
            orientation,
            r,
            edge_new_pcurve,
        )? {
            NextEdge::Missing => return Ok(false),
            NextEdge::Found(next) => current = next,
            NextEdge::None => {}
        }
    }
}

enum NextEdge {
    Found(WalkPoint),
    None,
    Missing,
}

/// The edge of `list` continuing the contour at `vertex`, its p-curve shifted by the
/// period when it is on the other side and bound in the map.
fn next_edge(
    list: &[Shape],
    current: &WalkPoint,
    vertex: &Shape,
    orientation: Orientation,
    r: Relocation<'_>,
    edge_new_pcurve: &mut EdgePCurveMap,
) -> Result<NextEdge, Failure> {
    for shape in list {
        let edge = topods::edge(shape)?;
        if edge.is_same(&current.edge) {
            continue;
        }
        let first_vertex = if orientation == Orientation::Forward {
            topexp::first_vertex(&edge, true)
        } else {
            topexp::last_vertex(&edge, true)
        };
        if !first_vertex.is_same(vertex) {
            continue; // "may be if CurVertex is deg.vertex"
        }
        let Some((pcurve, start, end)) = trimmed_pcurve(&edge, r.ref_face)? else {
            return Ok(NextEdge::Missing);
        };
        let parameter = if orientation == Orientation::Forward {
            start
        } else {
            end
        };
        let point = pcurve.get().ok_or_else(null_pcurve)?.value(parameter)?;
        let mut offset = coord(current.point, r.index)? - coord(point, r.index)?;
        if offset.abs() >= r.coord_tol && (offset.abs() - r.period).abs() >= r.coord_tol {
            continue; // "may be if CurVertex is deg.vertex"
        }
        let mut pcurve = pcurve;
        if offset.abs() > r.period / 2. {
            offset = true_value_of_offset(offset, r.period);
            pcurve = shifted(&pcurve, offset, r.index)?;
        }
        edge_new_pcurve.bind(edge.clone().into_shape(), pcurve.clone());
        let walk_orientation = top_abs::compose(orientation, edge.orientation());
        let curve = pcurve.get().ok_or_else(null_pcurve)?;
        let parameter = if walk_orientation == Orientation::Forward {
            curve.last_parameter()
        } else {
            curve.first_parameter()
        };
        let point = curve.value(parameter)?;
        return Ok(NextEdge::Found(WalkPoint { edge, point }));
    }
    Ok(NextEdge::None)
}

/// A copy of the p-curve translated by `offset` along coordinate `index`.
pub(super) fn shifted(
    pcurve: &Handle<dyn Geom2dCurve>,
    offset: f64,
    index: i32,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let copy = pcurve
        .get()
        .ok_or_else(null_pcurve)?
        .copy()?
        .cast::<dyn Geom2dCurve>();
    let vector = if index == 1 {
        Vector2::new(offset, 0.)
    } else {
        Vector2::new(0., offset)
    };
    copy.get().ok_or_else(null_pcurve)?.translate(vector)?;
    Ok(copy)
}
