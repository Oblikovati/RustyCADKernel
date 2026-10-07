//! The wire building of `IntUnifyFaces` (ShapeUpgrade_UnifySameDomain.cxx): the boundary
//! edges chained in 2D on the reference face into new wires, the seams removed between
//! the merged faces rebuilt where the walk crosses a period, and each wire made a new
//! face (when it touches the surface bound) or kept as a wire, split at its branchings.
use super::face_bounds::remove_edge_from_map;
use super::face_groups::FaceGroup;
use super::geometry::{is_on_singularity, null_pcurve};
use super::periodic_faces::MergeFrame;
use super::seams::{Periods, reconstruct_missed_seam};
use super::split_wire::split_wire;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The new faces and the wires (holes) built from the boundary.
#[derive(Default)]
pub(super) struct NewBoundary {
    pub faces: Vec<Shape>,
    pub wires: Vec<Shape>,
}

/// The state of the walk along one new wire.
struct Walk {
    wire: Wire,
    splitting: IndexedMapOfShape,
    start_vertex: Vertex,
    start_point: Point2,
    cur_edge: Edge,
    cur_vertex: Vertex,
    cur_point: Point2,
}

/// What the walk does at the current vertex.
enum Step {
    Next(Edge, Point2),
    EndOfWire,
    Failed,
}

/// The read-only inputs of the walk.
#[derive(Clone, Copy)]
struct WalkContext<'a> {
    ref_face: &'a Face,
    removed: &'a [Shape],
    periods: Periods,
    coord_tol: f64,
}

/// The parameter at the start (or end) of the edge in its direction.
fn oriented_parameter(edge: &Edge, (first, last): (f64, f64), at_end: bool) -> f64 {
    let forward = edge.orientation() == Orientation::Forward;
    if forward == at_end { last } else { first }
}

/// The unit tangent of the p-curve at the start (or end) of the edge, in the edge
/// direction; also the point at the end for the current edge.
fn tangent(edge: &Edge, face: &Face, at_end: bool) -> Result<Option<(Point2, Vector2)>, Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    let Some(curve) = found.pcurve.get() else {
        return Ok(None);
    };
    let d1 = curve.eval_d1(oriented_parameter(edge, (found.first, found.last), at_end))?;
    let mut direction = d1.d1;
    direction.normalize()?;
    if edge.orientation() == Orientation::Reversed {
        direction.reverse();
    }
    Ok(Some((d1.point, direction)))
}

impl Walk {
    /// The first edge of the wire; None when it has no p-curve on the reference face.
    fn start(edge: Edge, face: &Face) -> Result<Option<Self>, Failure> {
        let found = BRepTool::curve_on_face(&edge, face)?;
        let Some(curve) = found.pcurve.get() else {
            return Ok(None);
        };
        let (start_vertex, cur_vertex) = topexp::vertices(&edge, true);
        let range = (found.first, found.last);
        let start_point = curve.value(oriented_parameter(&edge, range, false))?;
        let cur_point = curve.value(oriented_parameter(&edge, range, true))?;
        let mut wire = Wire::default();
        let builder = BRepBuilder::default();
        builder.make_wire(&mut wire);
        builder.add(&mut wire, &edge)?;
        Ok(Some(Self {
            wire,
            splitting: IndexedMapOfShape::default(),
            start_vertex,
            start_point,
            cur_edge: edge,
            cur_vertex,
            cur_point,
        }))
    }

    fn offsets_to_start(&self) -> (f64, f64) {
        (
            (self.start_point.x() - self.cur_point.x()).abs(),
            (self.start_point.y() - self.cur_point.y()).abs(),
        )
    }

    fn reconstruct(&self, c: WalkContext<'_>) -> Result<Option<(Edge, Point2)>, Failure> {
        reconstruct_missed_seam(
            c.removed,
            c.ref_face,
            (&self.cur_edge, &self.cur_vertex, self.cur_point),
            c.periods,
        )
    }

    /// The candidates at the current vertex: the unused edges starting there, or the one
    /// turning the most when there are several on a non-periodic surface.
    fn candidates(
        &mut self,
        list: &[Shape],
        used: &MapOfShape,
        face: &Face,
        periodic: bool,
    ) -> Result<Vec<Edge>, Failure> {
        let mut starting = Vec::new();
        for shape in list {
            let edge = topods::edge(shape)?;
            if used.contains(shape) || !topexp::first_vertex(&edge, true).is_same(&self.cur_vertex)
            {
                continue;
            }
            starting.push(edge);
        }
        let has_pcurve = !BRepTool::curve_on_face(&self.cur_edge, face)?
            .pcurve
            .is_null();
        if starting.len() <= 1 || periodic || !has_pcurve {
            return Ok(starting);
        }
        // "we must choose the closest direction - the biggest angle".
        let (point, current) = tangent(&self.cur_edge, face, true)?.ok_or_else(null_pcurve)?;
        self.cur_point = point;
        let mut best: Option<(f64, Edge)> = None;
        for edge in starting {
            let Some((_, direction)) = tangent(&edge, face, false)? else {
                continue;
            };
            let angle = current.angle(direction)?;
            if best.as_ref().is_none_or(|(max, _)| angle > *max) {
                best = Some((angle, edge));
            }
        }
        Ok(best.into_iter().map(|(_, edge)| edge).collect())
    }

    /// The next edge among the candidates: one starting at the current point (within the
    /// tolerance, or a period away), a rebuilt seam when it is a period away.
    fn choose(
        &self,
        candidates: &[Edge],
        c: WalkContext<'_>,
    ) -> Result<(Option<(Edge, Point2)>, bool), Failure> {
        let (up, vp) = (c.periods.u, c.periods.v);
        for edge in candidates {
            let found = BRepTool::curve_on_face(edge, c.ref_face)?;
            let Some(curve) = found.pcurve.get() else {
                continue;
            };
            let range = (found.first, found.last);
            let point = curve.value(oriented_parameter(edge, range, false))?;
            let (du, dv) = (
                (point.x() - self.cur_point.x()).abs(),
                (point.y() - self.cur_point.y()).abs(),
            );
            if up != 0. && du > c.coord_tol && (du - up).abs() > c.coord_tol {
                continue; // "may be it is a deg.vertex"
            }
            if vp != 0. && dv > c.coord_tol && (dv - vp).abs() > c.coord_tol {
                continue;
            }
            if (up != 0. && du > up / 2.) || (vp != 0. && dv > vp / 2.) {
                // "Check: may be it is the end": the native seam vertex and point stay
                // unset (a null vertex and the origin).
                let end = Vertex::default().is_same(&self.start_vertex)
                    && self.start_point.x().abs() < up / 2.;
                return Ok((self.reconstruct(c)?, end));
            }
            let next_point = curve.value(oriented_parameter(edge, range, true))?;
            return Ok((Some((edge.clone(), next_point)), false));
        }
        Ok((None, false))
    }

    /// One step of the walk from the current vertex.
    fn step(
        &mut self,
        ve_map: &rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape,
        used: &MapOfShape,
        c: WalkContext<'_>,
    ) -> Result<Step, Failure> {
        let list: Vec<Shape> = match ve_map.seek(self.cur_vertex.as_ref()) {
            Some(list) => list.iter().cloned().collect(),
            None => Vec::new(),
        };
        let mut next = None;
        if list.is_empty() && self.cur_vertex.is_same(&self.start_vertex) {
            if !c.periods.is_far(self.offsets_to_start()) {
                return Ok(Step::EndOfWire);
            }
            next = self.reconstruct(c)?;
        }
        if next.is_none() {
            if !is_on_singularity(&list)? && list.len() > 1 {
                self.splitting.add(self.cur_vertex.clone().into_shape());
            }
            let periodic = c.periods.u != 0. || c.periods.v != 0.;
            let candidates = self.candidates(&list, used, c.ref_face, periodic)?;
            let (chosen, end_of_wire) = self.choose(&candidates, c)?;
            if end_of_wire {
                return Ok(Step::EndOfWire);
            }
            next = chosen;
        }
        if let Some((edge, point)) = next {
            return Ok(Step::Next(edge, point));
        }
        self.at_dead_end(c)
    }

    /// No next edge: the end of the wire when back at the start, else a rebuilt seam.
    fn at_dead_end(&self, c: WalkContext<'_>) -> Result<Step, Failure> {
        if c.periods.u == 0. && c.periods.v == 0. {
            return Ok(Step::Failed);
        }
        if self.cur_vertex.is_same(&self.start_vertex) && c.periods.is_near(self.offsets_to_start())
        {
            return Ok(Step::EndOfWire);
        }
        Ok(match self.reconstruct(c)? {
            Some((edge, point)) => Step::Next(edge, point),
            None => Step::Failed,
        })
    }
}

/// The first non-degenerated edge (the last one when all are) and its index.
fn start_edge(edges: &[Shape]) -> Result<(usize, Edge), Failure> {
    let mut index = 0;
    let mut edge = topods::edge(&edges[0])?;
    while BRepTool::degenerated(&edge)? && index + 1 < edges.len() {
        index += 1;
        edge = topods::edge(&edges[index])?;
    }
    Ok((index, edge))
}

/// "Building new wires from <edges> and build faces": None when a wire cannot be closed
/// (`isWireBuildingFailed`).
pub(super) fn build_new_boundary(
    group: &mut FaceGroup,
    frame: &mut MergeFrame,
) -> Result<Option<NewBoundary>, Failure> {
    let ref_face = group.forward_ref_face();
    let c = WalkContext {
        ref_face: &ref_face,
        removed: &group.removed,
        periods: Periods {
            u: group.u_period,
            v: group.v_period,
        },
        coord_tol: frame.coord_tol,
    };
    let mut used = MapOfShape::default();
    let mut result = NewBoundary::default();
    while !group.edges.is_empty() {
        let (index, start) = start_edge(&group.edges)?;
        remove_edge_from_map(start.as_ref(), &mut frame.ve_map)?;
        let Some(mut walk) = Walk::start(start.clone(), &ref_face)? else {
            group.edges.remove(index);
            continue;
        };
        if !walk_wire(&mut walk, frame, &mut used, c)? {
            return Ok(None);
        }
        walk.wire.set_closed(true)?;
        used.add(start.into_shape());
        group.edges.retain(|edge| !used.contains(edge));
        store_wire(walk, group, &ref_face, &mut result)?;
    }
    Ok(Some(result))
}

/// Walks the wire to its end; false when it fails.
fn walk_wire(
    walk: &mut Walk,
    frame: &mut MergeFrame,
    used: &mut MapOfShape,
    c: WalkContext<'_>,
) -> Result<bool, Failure> {
    loop {
        match walk.step(&frame.ve_map, used, c)? {
            Step::EndOfWire => return Ok(true),
            Step::Failed => return Ok(false),
            Step::Next(edge, point) => {
                walk.cur_point = point;
                walk.cur_vertex = topexp::last_vertex(&edge, true);
                BRepBuilder::default().add(&mut walk.wire, &edge)?;
                used.add(edge.clone().into_shape());
                remove_edge_from_map(edge.as_ref(), &mut frame.ve_map)?;
                walk.cur_edge = edge;
            }
        }
    }
}

/// "add just built wire to current face or save it in the sequence of wires".
fn store_wire(
    walk: Walk,
    group: &FaceGroup,
    ref_face: &Face,
    result: &mut NewBoundary,
) -> Result<(), Failure> {
    let mut on_bound = false;
    for child in ShapeIterator::new(&walk.wire, true, true) {
        if BRepTool::is_closed_on_face(&topods::edge(&child)?, &group.ref_face)? {
            on_bound = true;
            break;
        }
    }
    if on_bound {
        // "this wire can not be a hole".
        let (surface, location) = BRepTool::surface_with_location(&group.ref_face)?;
        let mut face = Face::default();
        let builder = BRepBuilder::default();
        builder.make_face_on_located_surface(&mut face, &surface, &location, 0.)?;
        builder.add(&mut face, &walk.wire)?;
        face.set_orientation(group.ref_orientation);
        result.faces.push(face.into_shape());
    } else if !walk.splitting.is_empty() {
        split_wire(&walk.wire, ref_face, &walk.splitting, &mut result.wires)?;
    } else {
        result.wires.push(walk.wire.into_shape());
    }
    Ok(())
}
