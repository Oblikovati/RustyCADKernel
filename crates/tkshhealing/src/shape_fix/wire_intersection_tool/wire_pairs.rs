//! `ShapeFix_IntersectionTool::FixIntersectingWires`, line by line: every pair of direct
//! face wires, every pair of their edges with point then segment repairs, and the face
//! rebuilt from the repaired wires.
use super::ShapeFixIntersectionTool;
use super::aggregate::points;
use super::boxes::{self, EdgeBox};
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox2;

#[path = "wire_pairs/big_segment.rs"]
mod big_segment;
#[path = "wire_pairs/cross_points.rs"]
mod cross_points;
#[path = "wire_pairs/segments.rs"]
mod segments;

/// One direct wire of the face: its sequence entry (`SeqWir`), its edge boxes
/// (`aSeqWirEdgeBoxes`) and their union (`aSeqWirBoxes`).
struct WireSlot {
    wire: Wire,
    boxes: Vec<EdgeBox>,
    total: BoundingBox2,
}

/// How the native edge loops go on after a pair: `num2++`, the same `num2` again
/// (`num2--; continue;`) or the same `num1` again (`num1--; break;`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Step {
    NextSecond,
    SameSecond,
    SameFirst,
}

/// The state shared by the edge pairs of one wire pair: the two wire data (`sewd1`,
/// `sewd2`), their edge boxes, the vertex tolerance bound and the native counters.
pub(super) struct PairScan<'a> {
    pub(super) tool: &'a ShapeFixIntersectionTool,
    pub(super) face: &'a Face,
    pub(super) data: [&'a ShapeExtendWireData; 2],
    pub(super) boxes: [&'a mut Vec<EdgeBox>; 2],
    pub(super) max_tolerance: &'a mut f64,
    pub(super) modifications: i32,
    pub(super) replaced: i32,
    pub(super) modified: bool,
}

/// The edges of one pair: `edge1` read once for each num1, `edge2` for each num2 and
/// replaced in place when its vertices change.
pub(super) struct EdgePair {
    pub(super) index: [i32; 2],
    pub(super) first: Edge,
    pub(super) second: Edge,
    pub(super) second_box: BoundingBox2,
}

/// Repairs the intersections between the direct wires of `face`; `wire_pairs::fix_intersecting_wires(tool, &mut face)?`.
pub(super) fn fix_intersecting_wires(
    tool: &ShapeFixIntersectionTool,
    face: &mut Face,
) -> Result<bool, Failure> {
    if tool.context().is_null() || face.is_null() {
        return Ok(false);
    }
    let (wires, non_wires) = direct_children(face)?;
    if wires.len() < 2 {
        return Ok(false);
    }
    let mut max_tolerance = face_vertex_tolerance(face)?;
    let mut slots = wires
        .into_iter()
        .map(|wire| wire_slot(wire, face))
        .collect::<Result<Vec<_>, _>>()?;
    let mut done = false;
    for first in 0..slots.len() - 1 {
        done |= scan_first_wire(tool, face, &mut slots, first, &mut max_tolerance)?;
    }
    if done {
        replace_face(tool, face, &slots, &non_wires)?;
    }
    Ok(done)
}

// `TopoDS_Iterator(face, false)`: wires oriented FORWARD or REVERSED, every other child
// kept as a non-manifold shape.
fn direct_children(face: &Face) -> Result<(Vec<Wire>, Vec<Shape>), Failure> {
    let shape = face.clone().into_shape();
    let mut iterator = ShapeIterator::new(&shape, false, true);
    let (mut wires, mut others) = (Vec::new(), Vec::new());
    while iterator.more() {
        let child = iterator.value()?.clone();
        iterator.next();
        let oriented = matches!(
            child.orientation(),
            Orientation::Forward | Orientation::Reversed
        );
        if child.shape_type()? == ShapeEnum::Wire && oriented {
            wires.push(topods::wire(&child)?);
        } else {
            others.push(child);
        }
    }
    Ok((wires, others))
}

fn face_vertex_tolerance(face: &Face) -> Result<f64, Failure> {
    let shape = face.clone().into_shape();
    let mut maximum: f64 = 0.0;
    for vertex in Explorer::new(&shape, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let tolerance = BRepTool::vertex_tolerance(&topods::vertex(&vertex)?)?;
        maximum = native_max(maximum, tolerance);
    }
    Ok(maximum)
}

/// `std::max(a, b)`, that is `a < b ? b : a`; `native_max(1.0, 2.0) == 2.0`.
pub(super) fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}

fn wire_slot(wire: Wire, face: &Face) -> Result<WireSlot, Failure> {
    let data = ShapeExtendWireData::from_wire(&wire, true, true)?;
    let (boxes, total) = edge_boxes(&data, face)?;
    Ok(WireSlot { wire, boxes, total })
}

// CreateBoxes2d: the edge boxes and their union.
fn edge_boxes(
    data: &ShapeExtendWireData,
    face: &Face,
) -> Result<(Vec<EdgeBox>, BoundingBox2), Failure> {
    let boxes = boxes::create_edge_boxes(data, face)?;
    let mut total = BoundingBox2::default();
    boxes.iter().for_each(|entry| total.add_box(&entry.bounds));
    Ok((boxes, total))
}

/// The first wire of the n1 loop: `wire1` (the key of its next context replacement),
/// `sewd1` kept over all n2 pairs, and the total box `aBox1` copied once (a later
/// update of the wire's box is not seen by the next pairs).
struct FirstWire {
    key: Wire,
    data: ShapeExtendWireData,
    total: BoundingBox2,
}

fn scan_first_wire(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    slots: &mut [WireSlot],
    first: usize,
    max_tolerance: &mut f64,
) -> Result<bool, Failure> {
    let mut wire = FirstWire {
        key: slots[first].wire.clone(),
        data: ShapeExtendWireData::from_wire(&slots[first].wire, true, true)?,
        total: slots[first].total.clone(),
    };
    let mut done = false;
    for second in first + 1..slots.len() {
        let second_total = &slots[second].total;
        if !wire.total.is_void() && !second_total.is_void() && wire.total.is_out_box(second_total) {
            continue;
        }
        done |= scan_wire_pair(tool, face, slots, (first, second), &mut wire, max_tolerance)?;
    }
    Ok(done)
}

fn scan_wire_pair(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    slots: &mut [WireSlot],
    (first, second): (usize, usize),
    wire: &mut FirstWire,
    max_tolerance: &mut f64,
) -> Result<bool, Failure> {
    let second_key = slots[second].wire.clone();
    let second_data = ShapeExtendWireData::from_wire(&second_key, true, true)?;
    let (left, right) = slots.split_at_mut(second);
    let mut scan = PairScan {
        tool,
        face,
        data: [&wire.data, &second_data],
        boxes: [&mut left[first].boxes, &mut right[0].boxes],
        max_tolerance,
        modifications: 0,
        replaced: 0,
        modified: false,
    };
    scan_edges(&mut scan)?;
    if !scan.modified {
        return Ok(false);
    }
    update_wire_slot(tool, face, &mut left[first], &wire.data, &wire.key)?;
    wire.key = wire.data.wire()?;
    update_wire_slot(tool, face, &mut right[0], &second_data, &second_key)?;
    Ok(true)
}

// `SeqWir.SetValue(n, sewd->Wire()); Context()->Replace(wire, sewd->Wire());`: each
// `Wire()` call builds another wire, so the sequence entry and the replacement differ.
fn update_wire_slot(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    slot: &mut WireSlot,
    data: &ShapeExtendWireData,
    key: &Wire,
) -> Result<(), Failure> {
    slot.wire = data.wire()?;
    tool.context()
        .replace(&key.clone().into_shape(), &data.wire()?.into_shape())?;
    let (boxes, total) = edge_boxes(data, face)?;
    slot.boxes = boxes;
    slot.total = total;
    Ok(())
}

// The num1 and num2 loops; `edge1` is read once for each num1, as natively.
fn scan_edges(scan: &mut PairScan<'_>) -> Result<(), Failure> {
    let mut first = 1;
    while first <= scan.data[0].nb_edges() && scan.modifications < 30 {
        let edge1 = scan.data[0].edge(first)?;
        if !scan_second_edges(scan, &edge1, first)? {
            first += 1;
        }
    }
    Ok(())
}

// The num2 loop for one `edge1`; true when it ends with `num1--; break;`.
fn scan_second_edges(scan: &mut PairScan<'_>, edge1: &Edge, first: i32) -> Result<bool, Failure> {
    let mut second = 1;
    while second <= scan.data[1].nb_edges() && scan.modifications < 30 {
        match scan_edge_pair(scan, edge1, [first, second])? {
            Step::NextSecond => second += 1,
            Step::SameSecond => {}
            Step::SameFirst => return Ok(true),
        }
    }
    Ok(false)
}

// The body of the native num2 loop.
fn scan_edge_pair(scan: &mut PairScan<'_>, edge1: &Edge, index: [i32; 2]) -> Result<Step, Failure> {
    let edge2 = scan.data[1].edge(index[1])?;
    if edge1.is_same(&edge2) || BRepTool::degenerated(edge1)? || BRepTool::degenerated(&edge2)? {
        return Ok(Step::NextSecond);
    }
    let (Some(box1), Some(box2)) = (
        boxes::find_edge_box(scan.boxes[0], edge1),
        boxes::find_edge_box(scan.boxes[1], &edge2),
    ) else {
        return Ok(Step::NextSecond);
    };
    if box1.is_out_box(&box2) {
        return Ok(Step::NextSecond);
    }
    let Some(curves) = points::pair_curves(edge1, &edge2, scan.face)? else {
        return Ok(Step::NextSecond);
    };
    if !curves.intersections.is_done() {
        return Ok(Step::NextSecond);
    }
    let mut pair = EdgePair {
        index,
        first: edge1.clone(),
        second: edge2,
        second_box: box2,
    };
    if let Some(step) = cross_points::repair(scan, &mut pair, &curves)? {
        return Ok(step);
    }
    // `hasModifWire = (hasModifWire || NbModif || nbReplaced);` sits only here.
    scan.modified |= scan.modifications != 0 || scan.replaced != 0;
    segments::repair(scan, &mut pair, &curves)
}

fn replace_face(
    tool: &ShapeFixIntersectionTool,
    face: &mut Face,
    slots: &[WireSlot],
    non_wires: &[Shape],
) -> Result<(), Failure> {
    let orientation = face.orientation();
    let mut replacement = face.clone().into_shape().empty_copied()?;
    replacement.set_orientation(Orientation::Forward);
    let builder = BRepBuilder::default();
    for slot in slots {
        builder.add(&mut replacement, &slot.wire.clone().into_shape())?;
    }
    for child in non_wires {
        builder.add(&mut replacement, child)?;
    }
    replacement.set_orientation(orientation);
    tool.context()
        .replace(&face.clone().into_shape(), &replacement)?;
    *face = topods::face(&replacement)?;
    Ok(())
}

#[cfg(test)]
#[path = "wire_pairs_tests.rs"]
mod tests;
