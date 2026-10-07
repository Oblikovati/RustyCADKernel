//! `GetFaceFromSeq`, `RemoveEdgeFromMap`, `ComputeMinEdgeSize`, `FindCoordBounds` and
//! `AddOrdinaryEdges` of ShapeUpgrade_UnifySameDomain.cxx: the edge bookkeeping of the
//! faces to merge and the extent of the merged area along a periodic direction.
use super::geometry::update_boundaries;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The edges of `shape` in explorer order (none for a null shape).
pub(super) fn edges_of(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    if shape.is_null() {
        return Ok(Vec::new());
    }
    Ok(Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?.collect())
}

/// The other face of a two-face list (the first one unless it is `face`).
fn other_face(faces: &[Shape], face: &Shape) -> Option<Shape> {
    let first = faces.first()?;
    if first.is_same(face) {
        return faces.last().cloned();
    }
    Some(first.clone())
}

/// `GetFaceFromSeq(theFaces, theRefFace, theMapEF, theUsedFaces, theIndFace)`: the first
/// face when none is used yet; else a face sharing a non-seam edge with a used face (or
/// a seam edge when there is no other), added to the used faces with its index in the
/// sequence (`index` unchanged when the face is not found from the second one on).
pub(super) fn get_face_from_seq(
    faces: &[Shape],
    ref_face: &Face,
    map_ef: &IndexedDataMapOfShapeListOfShape,
    used: &mut IndexedMapOfShape,
    index: &mut usize,
) -> Result<Shape, Failure> {
    if used.is_empty() {
        let first = faces.first().cloned().ok_or_else(|| {
            Failure::with_type(
                "Standard_OutOfRange",
                "ShapeUpgrade_UnifySameDomain: GetFaceFromSeq on 0 faces; expected at least 1",
                "",
            )
        })?;
        *index = 1;
        used.add(first.clone());
        return Ok(first);
    }
    let mut target = Shape::default();
    for ii in 1..=used.extent() {
        let used_face = used.find_key(ii)?.clone();
        target = bordering_face(&used_face, ref_face, map_ef, used)?;
        if !target.is_null() {
            break;
        }
    }
    used.add(target.clone());
    if let Some(found) = faces.iter().skip(1).position(|f| f.is_same(&target)) {
        *index = found + 2;
    }
    Ok(target)
}

/// The face across a non-seam edge of `used_face` not used yet, else across its last
/// seam edge with two faces, else null.
fn bordering_face(
    used_face: &Shape,
    ref_face: &Face,
    map_ef: &IndexedDataMapOfShapeListOfShape,
    used: &IndexedMapOfShape,
) -> Result<Shape, Failure> {
    let mut seam = Shape::default();
    for shape in edges_of(used_face)? {
        let list: Vec<Shape> = map_ef.find_from_key(&shape)?.iter().cloned().collect();
        if list.len() < 2 {
            continue;
        }
        let face = other_face(&list, used_face).unwrap_or_default();
        if used.contains(&face) {
            continue;
        }
        if BRepTool::is_closed_on_face(&topods::edge(&shape)?, ref_face)? {
            seam = shape;
            continue;
        }
        return Ok(face);
    }
    if seam.is_null() {
        return Ok(Shape::default());
    }
    let list: Vec<Shape> = map_ef.find_from_key(&seam)?.iter().cloned().collect();
    Ok(other_face(&list, used_face).unwrap_or_default())
}

/// `RemoveEdgeFromMap(theEdge, theVertexToEdges)`: the edge removed from the lists of
/// both its vertices; true when it was found.
pub(super) fn remove_edge_from_map(
    edge: &Shape,
    map: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<bool, Failure> {
    let (first, last) = topexp::vertices(&topods::edge(edge)?, false);
    let mut removed = false;
    for vertex in [first.into_shape(), last.into_shape()] {
        let Some(list) = map.change_seek(&vertex) else {
            continue;
        };
        let before = list.extent();
        list.retain(|candidate| !candidate.is_same(edge));
        removed |= list.extent() != before;
    }
    Ok(removed)
}

/// `ComputeMinEdgeSize(theEdges, theRefFace, theEdgesMap)`: the smallest 2D chord (to
/// the middle point for a closed non-degenerated edge) of the edges with a p-curve on
/// the reference face; every edge is added to `edges_map`.
pub(super) fn compute_min_edge_size(
    edges: &[Shape],
    ref_face: &Face,
    edges_map: &mut MapOfShape,
) -> Result<f64, Failure> {
    let mut min_size = f64::MAX;
    for shape in edges {
        edges_map.add(shape.clone());
        let edge = topods::edge(shape)?;
        let (v1, v2) = topexp::vertices(&edge, false);
        let adaptor = BRepAdaptorCurve2d::with_edge_on_face(&edge, ref_face)?;
        if adaptor.geom2d_adaptor().curve().is_null() {
            continue;
        }
        let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let first_point = adaptor.value(first)?;
        let other = if v1.is_same(&v2) && !BRepTool::degenerated(&edge)? {
            adaptor.value((first + last) / 2.)?
        } else {
            adaptor.value(last)?
        };
        min_size = min_size.min(first_point.square_distance(other));
    }
    Ok(min_size.sqrt())
}

/// The output of `FindCoordBounds`.
pub(super) struct CoordBounds {
    pub min: f64,
    pub max: f64,
    pub nb_intervals: usize,
    pub face_max: usize,
}

/// `FindCoordBounds(...)`: the intervals of coordinate `index` covered by the faces (in
/// bordering order) on the reference face, merged; the minimum is that of the first
/// interval (of the second one less the period with two intervals) and the maximum that
/// of the first. None when an edge has no p-curve or no interval is found.
pub(super) fn find_coord_bounds(
    faces: &[Shape],
    ref_face: &Face,
    map_ef: &IndexedDataMapOfShapeListOfShape,
    edges_map: &MapOfShape,
    (index, period): (i32, f64),
) -> Result<Option<CoordBounds>, Failure> {
    let mut intervals: Vec<(f64, f64)> = Vec::new();
    let mut simple_max = f64::MIN;
    let mut face_max = 0;
    let mut used = IndexedMapOfShape::default();
    for _ in 0..faces.len() {
        let mut face_index = 0;
        let face = get_face_from_seq(faces, ref_face, map_ef, &mut used, &mut face_index)?;
        let Some((min, max)) = face_range(&face, ref_face, edges_map, index)? else {
            return Ok(None);
        };
        if precision::is_infinite(min) || precision::is_infinite(max) {
            continue;
        }
        if max > simple_max {
            simple_max = max;
            face_max = face_index;
        }
        insert_interval(&mut intervals, (min, max));
    }
    let Some(&(first_min, first_max)) = intervals.first() else {
        return Ok(None);
    };
    let min = if intervals.len() == 2 {
        intervals[1].0 - period
    } else {
        first_min
    };
    Ok(Some(CoordBounds {
        min,
        max: first_max,
        nb_intervals: intervals.len(),
        face_max,
    }))
}

/// The range of coordinate `index` of the edges of the face in `edges_map`; None when
/// one of them has no p-curve on the reference face.
fn face_range(
    face: &Shape,
    ref_face: &Face,
    edges_map: &MapOfShape,
    index: i32,
) -> Result<Option<(f64, f64)>, Failure> {
    let (mut min, mut max) = (f64::MAX, f64::MIN);
    for shape in edges_of(face)? {
        if !edges_map.contains(&shape) {
            continue;
        }
        let found = BRepTool::curve_on_face(&topods::edge(&shape)?, ref_face)?;
        if found.pcurve.is_null() {
            return Ok(None);
        }
        update_boundaries(
            &found.pcurve,
            (found.first, found.last),
            index,
            (&mut min, &mut max),
        )?;
    }
    Ok(Some((min, max)))
}

/// Inserts `(min, max)` into the intervals: inside one, extending one, or new (prepended
/// when below the first one).
fn insert_interval(intervals: &mut Vec<(f64, f64)>, (min, max): (f64, f64)) {
    for interval in intervals.iter_mut() {
        let (local_min, local_max) = *interval;
        let min_inside = min >= local_min && min <= local_max;
        let max_inside = max >= local_min && max <= local_max;
        if min_inside && max_inside {
            return;
        }
        if min < local_min && max_inside {
            interval.0 = min;
            return;
        }
        if min < local_min && max > local_max {
            *interval = (min, max);
            return;
        }
        if min_inside && max > local_max {
            interval.1 = max;
            return;
        }
    }
    if !intervals.is_empty() && max < intervals[0].0 {
        intervals.insert(0, (min, max));
    } else {
        intervals.push((min, max));
    }
}

/// `AddOrdinaryEdges(edges, aShape, anIndex, theRemovedEdges)`: the edges of `shape`
/// added to `edges`, the edges met twice (seams) and those already in `edges` removed
/// (into `removed`); `index` receives the 1-based position before the first edge removed
/// from `edges` (true is returned then).
pub(super) fn add_ordinary_edges(
    edges: &mut Vec<Shape>,
    shape: &Shape,
    index: &mut usize,
    removed: &mut Vec<Shape>,
) -> Result<bool, Failure> {
    let mut new_edges = IndexedMapOfShape::default();
    for edge in edges_of(shape)? {
        if new_edges.contains(&edge) {
            new_edges.remove_key(&edge);
            removed.push(edge);
        } else {
            new_edges.add(edge);
        }
    }
    let mut dropped = false;
    let mut i = 0;
    while i < edges.len() {
        let current = edges[i].clone();
        if !new_edges.contains(&current) {
            i += 1;
            continue;
        }
        new_edges.remove_key(&current);
        edges.remove(i);
        removed.push(current);
        if !dropped {
            dropped = true;
            // The native 1-based `i` after its decrement.
            *index = i;
        }
    }
    for k in 1..=new_edges.extent() {
        edges.push(new_edges.find_key(k)?.clone());
    }
    Ok(dropped)
}
