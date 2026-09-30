// Rust translation of BRepCheck_Shell::Orientation (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Shell.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Shell::Orientation`: two faces sharing an edge must use it with
//! opposite orientations; when one does not, the shell is checked to be orientable
//! by reversing faces from the last reference face (not on a Moebius band).
use super::errors::no_such_object;
use super::explore::sub_shapes;
use super::shell::BRepCheckShell;
use super::status::BRepCheckStatus;
use super::wire_closed::record;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// `MapOfShapeOrientation`: the orientation of each face in the shell.
type FaceOrientations = DataMap<Shape, Orientation, ShapeMapHasher>;

/// An explorer of the edges of `face` moved to the first edge same as `edge`; at the
/// end its current shape is the face itself, as natively.
fn edge_explorer_at(face: &Shape, edge: &Shape) -> Result<Explorer, Failure> {
    let mut explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    advance_to_same(&mut explorer, edge)?;
    Ok(explorer)
}

/// Moves `explorer` to the next shape same as `edge` (or to its end).
fn advance_to_same(explorer: &mut Explorer, edge: &Shape) -> Result<(), Failure> {
    while explorer.more() && !explorer.current().is_same(edge) {
        explorer.next()?;
    }
    Ok(())
}

/// `face` with the orientation recorded for it; `SubshapeNotInShape` when it has none.
fn oriented_face(orientations: &FaceOrientations, face: &Shape) -> Result<Shape, BRepCheckStatus> {
    match orientations.seek(face) {
        Some(orientation) => Ok(face.oriented(*orientation)),
        None => Err(BRepCheckStatus::SubshapeNotInShape),
    }
}

/// The walk of the edges: the reference face of the last shared edge and the status.
struct EdgeScan {
    reference: Shape,
    status: BRepCheckStatus,
}

impl BRepCheckShell {
    /// `Orientation(Update)`: after `Closed` (a closure defect other than `NotClosed`
    /// is returned as is), `RedundantFace` for a face met twice (with `update` the check
    /// goes on), `SubshapeNotInShape`, `BadOrientationOfSubshape` for an edge used with
    /// one orientation by two faces (or unbalanced on a multi-connected edge), and
    /// `UnorientableShape` when no face reversal makes the shell coherent. With
    /// `update`, the first bad edge stops the scan and the orientability is checked;
    /// without it the status is returned at once, as natively;
    /// `checker.orientation(false)?`.
    pub fn orientation(&self, update: bool) -> Result<BRepCheckStatus, Failure> {
        let list = self.own_list()?;
        let (odone, ostat) = {
            let fields = self.fields.read();
            (fields.odone, fields.ostat)
        };
        if odone {
            record(&list, update, ostat);
            return Ok(ostat);
        }
        self.fields.write().odone = true;
        let closed = self.closed(false)?;
        self.fields.write().ostat = closed;
        if closed != BRepCheckStatus::NotClosed && closed != BRepCheckStatus::NoError {
            record(&list, update, closed);
            return Ok(closed);
        }
        let status = self.compute_orientation(update, &list)?;
        self.fields.write().ostat = status;
        record(&list, update, status);
        Ok(status)
    }

    /// The whole check (a redundant face is recorded at once with `update`, and the
    /// check goes on; without it the check stops there).
    fn compute_orientation(
        &self,
        update: bool,
        list: &super::result::SharedStatusList,
    ) -> Result<BRepCheckStatus, Failure> {
        let mut status = BRepCheckStatus::NoError;
        // The orientation of each face in the shell: a face met twice keeps the later.
        let mut orientations = FaceOrientations::default();
        for face in sub_shapes(&self.shell(), ShapeEnum::Face)? {
            let orientation = face.orientation();
            if !orientations.bind(face, orientation) {
                status = BRepCheckStatus::RedundantFace;
                if !update {
                    return Ok(status);
                }
                record(list, true, status);
            }
        }
        let fields = self.fields.read();
        let map_ef = &fields.map_ef;
        let scan = match scan_edges(map_ef, &orientations, update, status)? {
            Ok(scan) => scan,
            Err(leave) => return Ok(leave),
        };
        if scan.status != BRepCheckStatus::BadOrientationOfSubshape {
            return Ok(scan.status);
        }
        // With `update` the bad edge is recorded before the scan breaks (without it,
        // the scan has returned).
        record(list, update, scan.status);
        if scan.reference.is_null() || map_ef.extent() == 0 {
            return Ok(scan.status);
        }
        check_orientable(map_ef, &mut orientations, scan.reference)
    }
}

/// The scan of the shared edges; `Err(status)` when the check leaves at once.
fn scan_edges(
    map_ef: &IndexedDataMapOfShapeListOfShape,
    orientations: &FaceOrientations,
    update: bool,
    mut status: BRepCheckStatus,
) -> Result<Result<EdgeScan, BRepCheckStatus>, Failure> {
    let mut reference = Shape::default();
    for index in 1..=map_ef.extent() {
        let edge = map_ef.find_key(index)?.clone();
        if BRepTool::degenerated(&Edge::unchecked(edge.clone()))? {
            continue;
        }
        let faces = map_ef.find_from_index(index)?;
        let bad = if faces.extent() <= 2 {
            let first = faces
                .first()
                .ok_or_else(|| no_such_object("BRepCheck_Shell::Orientation: edge without face"))?;
            reference = first.clone();
            match pair_is_bad(&edge, faces, orientations, &mut reference)? {
                Ok(bad) => bad,
                Err(leave) => return Ok(Err(leave)),
            }
        } else {
            match multi_is_bad(&edge, faces, orientations)? {
                Ok(bad) => bad,
                Err(leave) => return Ok(Err(leave)),
            }
        };
        if bad {
            status = BRepCheckStatus::BadOrientationOfSubshape;
            if !update {
                return Ok(Err(status));
            }
            break;
        }
    }
    Ok(Ok(EdgeScan { reference, status }))
}

/// An edge of at most two faces: bad when the second face uses it with the
/// orientation of the first (and not a second time otherwise). `reference` becomes
/// the oriented first face.
fn pair_is_bad(
    edge: &Shape,
    faces: &List<Shape>,
    orientations: &FaceOrientations,
    reference: &mut Shape,
) -> Result<Result<bool, BRepCheckStatus>, Failure> {
    if !orientations.is_bound(reference) {
        return Ok(Err(BRepCheckStatus::SubshapeNotInShape));
    }
    let Some(second) = faces.iter().nth(1) else {
        return Ok(Ok(false));
    };
    *reference = match oriented_face(orientations, reference) {
        Ok(face) => face,
        Err(leave) => return Ok(Err(leave)),
    };
    // An edge "closed" on one face is not examined.
    if second.is_same(reference) {
        return Ok(Ok(false));
    }
    let orient = edge_explorer_at(reference, edge)?.current().orientation();
    let current = match oriented_face(orientations, second) {
        Ok(face) => face,
        Err(leave) => return Ok(Err(leave)),
    };
    let mut explorer = edge_explorer_at(&current, edge)?;
    if explorer.current().orientation() != orient {
        return Ok(Ok(false));
    }
    // The loop goes on over the edges as many times as the edge is in the wire.
    explorer.next()?;
    advance_to_same(&mut explorer, edge)?;
    let found = explorer.more();
    Ok(Ok(!found || explorer.current().orientation() == orient))
}

/// An edge of more than two faces: bad when its FORWARD and REVERSED uses differ in
/// number (the second use counts for a face met twice).
fn multi_is_bad(
    edge: &Shape,
    faces: &List<Shape>,
    orientations: &FaceOrientations,
) -> Result<Result<bool, BRepCheckStatus>, Failure> {
    let (mut forward, mut reversed) = (0, 0);
    let mut seen = MapOfShape::default();
    for face in faces {
        let current = match oriented_face(orientations, face) {
            Ok(oriented) => oriented,
            Err(leave) => return Ok(Err(leave)),
        };
        let mut explorer = edge_explorer_at(&current, edge)?;
        if seen.contains(&current) {
            // The edge is "closed" on the face, met twice.
            explorer.next()?;
            advance_to_same(&mut explorer, edge)?;
        }
        if explorer.current().orientation() == Orientation::Forward {
            forward += 1;
        } else {
            reversed += 1;
        }
        seen.add(current);
    }
    Ok(Ok(forward != reversed))
}

/// The orientability check from `start`: the faces are visited breadth first and a
/// neighbour using a shared edge with the same orientation is reversed; one met again
/// is `UnorientableShape` (a Moebius band).
fn check_orientable(
    map_ef: &IndexedDataMapOfShapeListOfShape,
    orientations: &mut FaceOrientations,
    start: Shape,
) -> Result<BRepCheckStatus, Failure> {
    let mut visited = MapOfShape::default();
    let mut pending: Vec<Shape> = vec![start];
    while !pending.is_empty() {
        let face = pending.remove(0);
        let reference = match oriented_face(orientations, &face) {
            Ok(oriented) => oriented,
            Err(leave) => return Ok(leave),
        };
        visited.add(reference.clone());
        for edge in sub_shapes(&reference, ShapeEnum::Edge)? {
            let faces = map_ef.find_from_key(&edge)?;
            let mut iter = faces.iter();
            let mut neighbour = iter.next().cloned().unwrap_or_default();
            if neighbour.is_same(&reference) {
                match iter.next() {
                    Some(next) => neighbour = next.clone(),
                    // From the free border one goes to the next edge.
                    None => continue,
                }
            }
            let Some(&orientation) = orientations.seek(&neighbour) else {
                return Ok(BRepCheckStatus::SubshapeNotInShape);
            };
            let current = neighbour.oriented(orientation);
            if edge_explorer_at(&current, &edge)?.current().orientation() == edge.orientation() {
                if visited.contains(&current) {
                    // A face already examined would have to be returned: the shell
                    // cannot be oriented (quit, otherwise it may take too long).
                    return Ok(BRepCheckStatus::UnorientableShape);
                }
                *orientations.change_find(&current)? = top_abs::reverse(orientation);
            }
            if visited.add(current.clone()) {
                pending.push(current);
            }
        }
    }
    Ok(BRepCheckStatus::BadOrientationOfSubshape)
}
