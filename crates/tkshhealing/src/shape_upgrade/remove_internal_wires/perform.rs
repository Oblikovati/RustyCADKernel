use super::*;
use crate::shape_analysis::ShapeAnalysis;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::{
    topexp::{self, Explorer},
    topods::{self, ShapeIterator},
};
use rustycad_tkernel::ncollection::Sequence;

impl ShapeUpgradeRemoveInternalWires {
    /// Expose the native protected reset without changing the input, result, or settings.
    /// Example: `remover.clear();`.
    pub fn clear(&self) {
        clear(&self.fields);
    }

    /// Removes every eligible internal wire in the initialized shape; `remover.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        let fields = &self.fields;
        clear(fields);
        if fields.read().shape.is_null() {
            return fail_initial_shape(fields);
        }
        let context = self;
        let input = fields.read().shape.clone();
        scan_all_faces(fields, context, &input)?;
        finish_perform(fields, context, &input)
    }

    /// Processes only listed faces and wires; `sequence.append(face); remover.perform_on(&sequence)?`.
    pub fn perform_on(&self, shapes: &Sequence<Shape>) -> Result<bool, Failure> {
        let fields = &self.fields;
        if fields.read().shape.is_null() {
            return fail_initial_shape(fields);
        }
        clear(fields);
        let context = self;
        let input = fields.read().shape.clone();
        scan_selected_shapes(fields, context, &input, shapes)?;
        finish_perform(fields, context, &input)
    }
}

fn scan_all_faces(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    input: &Shape,
) -> Result<(), Failure> {
    for face in Explorer::new(input, ShapeEnum::Face, ShapeEnum::Shape)? {
        remove_small_wire(fields, context, &face, None)?;
    }
    if fields.read().remove_face_mode {
        remove_small_faces(fields, context)?;
    }
    Ok(())
}

fn scan_selected_shapes(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    input: &Shape,
    shapes: &Sequence<Shape>,
) -> Result<(), Failure> {
    let mut wire_faces = None;
    for shape in shapes.iter() {
        process_selected_shape(fields, context, input, shape, &mut wire_faces)?;
    }
    if fields.read().remove_face_mode {
        remove_small_faces(fields, context)?;
    }
    Ok(())
}

fn finish_perform(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    input: &Shape,
) -> Result<bool, Failure> {
    let result = context
        .context()
        .get()
        .ok_or_else(null_context)?
        .apply(input, ShapeEnum::Shape)?;
    fields.write().result = result;
    Ok(ShapeExtend::decode_status(
        fields.read().status,
        ShapeExtendStatus::Done,
    ))
}

fn process_selected_shape(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    input: &Shape,
    shape: &Shape,
    wire_faces: &mut Option<IndexedDataMapOfShapeListOfShape>,
) -> Result<(), Failure> {
    match shape.shape_type()? {
        ShapeEnum::Face => remove_small_wire(fields, context, shape, None),
        ShapeEnum::Wire => process_selected_wire(fields, context, input, shape, wire_faces),
        _ => Ok(()),
    }
}

fn process_selected_wire(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    input: &Shape,
    wire: &Shape,
    wire_faces: &mut Option<IndexedDataMapOfShapeListOfShape>,
) -> Result<(), Failure> {
    initialize_wire_faces(input, wire_faces)?;
    let map = wire_faces.as_ref().expect("wire map initialized");
    let Some(faces) = map.seek(wire) else {
        return Ok(());
    };
    for face in faces.iter() {
        remove_small_wire(fields, context, face, Some(wire))?;
    }
    Ok(())
}

fn initialize_wire_faces(
    input: &Shape,
    wire_faces: &mut Option<IndexedDataMapOfShapeListOfShape>,
) -> Result<(), Failure> {
    if wire_faces
        .as_ref()
        .is_some_and(|existing| existing.extent() > 0)
    {
        return Ok(());
    }
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(input, ShapeEnum::Wire, ShapeEnum::Face, &mut map)?;
    *wire_faces = Some(map);
    Ok(())
}

fn clear(fields: &SharedState<RemoveInternalWiresFields>) {
    let mut fields = fields.write();
    fields.remove_edges.clear(true);
    fields.removed_faces.change_sequence(|seq| seq.clear(None));
    fields.removed_wires.change_sequence(|seq| seq.clear(None));
    fields.status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
}

fn fail_initial_shape(fields: &SharedState<RemoveInternalWiresFields>) -> Result<bool, Failure> {
    fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail1);
    Ok(false)
}

fn remove_small_wire(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    face: &Shape,
    selected_wire: Option<&Shape>,
) -> Result<(), Failure> {
    let face = topods::face(face)?;
    let outer = ShapeAnalysis::outer_wire(&face)?.into_shape();
    for child in ShapeIterator::new(&face.clone().into_shape(), true, true) {
        if child.shape_type()? != ShapeEnum::Wire || child.is_same(&outer) {
            continue;
        }
        if selected_wire.is_some_and(|selected| !selected.is_same(&child)) {
            continue;
        }
        remove_wire_if_small(fields, context, &face, &child)?;
    }
    Ok(())
}

fn remove_wire_if_small(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    face: &rustycad_tkbrep::topods::Face,
    wire: &Shape,
) -> Result<(), Failure> {
    if !wire_is_small(fields, wire)? {
        return Ok(());
    }
    context
        .context()
        .get()
        .ok_or_else(null_context)?
        .remove(wire)?;
    context.removed_wires().append(wire.clone());
    fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
    if !fields.read().remove_face_mode {
        return Ok(());
    }
    record_removed_edges(fields, wire, &face.clone().into_shape());
    Ok(())
}

fn wire_is_small(
    fields: &SharedState<RemoveInternalWiresFields>,
    wire: &Shape,
) -> Result<bool, Failure> {
    let area = ShapeAnalysis::contour_area(&topods::wire(wire)?)?;
    let threshold = fields.read().min_area - rustycad_tkernel::precision::confusion();
    Ok(area
        .partial_cmp(&threshold)
        .is_some_and(|order| order.is_lt()))
}

fn record_removed_edges(
    fields: &SharedState<RemoveInternalWiresFields>,
    wire: &Shape,
    face: &Shape,
) {
    for edge in ShapeIterator::new(wire, false, true) {
        append_removed_edge_face(fields, &edge, face);
    }
}

fn append_removed_edge_face(
    fields: &SharedState<RemoveInternalWiresFields>,
    edge: &Shape,
    face: &Shape,
) {
    let mut fields = fields.write();
    if let Some(faces) = fields.remove_edges.change_seek(edge) {
        faces.append(face.clone());
        return;
    }
    let mut faces = List::default();
    faces.append(face.clone());
    fields.remove_edges.add(edge.clone(), faces);
}

fn remove_small_faces(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
) -> Result<(), Failure> {
    let wires: Vec<Shape> = context
        .removed_wires()
        .sequence(|seq| seq.iter().cloned().collect());
    for wire in wires {
        let mut candidates = Vec::new();
        gather_face_candidates(fields, context, &wire, &mut candidates)?;
        remove_empty_candidates(fields, context, &candidates)?;
    }
    if !context.removed_faces().sequence(|seq| seq.is_empty()) {
        fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done2);
    }
    Ok(())
}

fn gather_face_candidates(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    wire: &Shape,
    candidates: &mut Vec<Shape>,
) -> Result<(), Failure> {
    for edge in ShapeIterator::new(wire, false, true) {
        collect_edge_candidates(fields, context, &edge, candidates)?;
    }
    Ok(())
}

fn collect_edge_candidates(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    edge: &Shape,
    candidates: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let faces = fields.read().edge_faces.seek(edge).cloned();
    let Some(faces) = faces else {
        fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail2);
        return Ok(());
    };
    let removed = fields
        .read()
        .remove_edges
        .seek(edge)
        .cloned()
        .ok_or_else(|| missing_edge_faces(edge))?;
    let mut removed_cursor = 0;
    for face in faces.iter() {
        collect_candidate(
            context,
            face,
            &removed,
            &mut removed_cursor,
            edge,
            candidates,
        )?;
    }
    Ok(())
}

fn collect_candidate(
    context: &ShapeUpgradeRemoveInternalWires,
    face: &Shape,
    removed: &List<Shape>,
    removed_cursor: &mut usize,
    edge: &Shape,
    candidates: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let applied = context
        .context()
        .get()
        .ok_or_else(null_context)?
        .apply(face, ShapeEnum::Shape)?;
    if applied.is_null() || matches_removed_face(context, &applied, removed, removed_cursor)? {
        return Ok(());
    }
    if !outer_contains_edge(&applied, edge)? || candidates.iter().any(|old| old.is_same(&applied)) {
        return Ok(());
    }
    candidates.push(applied);
    Ok(())
}

fn matches_removed_face(
    context: &ShapeUpgradeRemoveInternalWires,
    candidate: &Shape,
    removed: &List<Shape>,
    cursor: &mut usize,
) -> Result<bool, Failure> {
    for (index, old) in removed.iter().enumerate().skip(*cursor) {
        let applied = context
            .context()
            .get()
            .ok_or_else(null_context)?
            .apply(old, ShapeEnum::Shape)?;
        if candidate.is_same(&applied) {
            *cursor = index + 1;
            return Ok(true);
        }
        *cursor = index + 1;
    }
    Ok(false)
}

fn outer_contains_edge(face: &Shape, edge: &Shape) -> Result<bool, Failure> {
    let face = topods::face(face)?;
    let outer = ShapeAnalysis::outer_wire(&face)?;
    Ok(ShapeIterator::new(&outer.into_shape(), false, true)
        .any(|candidate| candidate.is_same(edge)))
}

fn remove_empty_candidates(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    candidates: &[Shape],
) -> Result<(), Failure> {
    for face in candidates {
        if has_nonremoved_neighbor(fields, context, face, candidates)? {
            continue;
        }
        context
            .context()
            .get()
            .ok_or_else(null_context)?
            .remove(face)?;
        context.removed_faces().append(face.clone());
    }
    Ok(())
}

fn has_nonremoved_neighbor(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    face: &Shape,
    candidates: &[Shape],
) -> Result<bool, Failure> {
    let face = topods::face(face)?;
    let outer = ShapeAnalysis::outer_wire(&face)?;
    let wire_data = Handle::new(crate::shape_extend::ShapeExtendWireData::from_wire(
        &outer, true, true,
    )?);
    let mut count = 0;
    for index in 1..=wire_data.nb_edges() {
        count += edge_neighbor_count(fields, context, &face, candidates, &wire_data, index)?;
    }
    Ok(count > 0)
}

fn edge_neighbor_count(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    face: &Shape,
    candidates: &[Shape],
    wire_data: &crate::shape_extend::ShapeExtendWireData,
    index: i32,
) -> Result<usize, Failure> {
    if wire_data.is_seam(index) {
        return Ok(0);
    }
    let edge = wire_data.edge(index)?.into_shape();
    if fields.read().remove_edges.contains(&edge) {
        return Ok(0);
    }
    count_external_adjacent_faces(fields, context, face, candidates, &edge)
}

fn count_external_adjacent_faces(
    fields: &SharedState<RemoveInternalWiresFields>,
    context: &ShapeUpgradeRemoveInternalWires,
    face: &Shape,
    candidates: &[Shape],
    edge: &Shape,
) -> Result<usize, Failure> {
    let mut count = 0;
    let adjacent_faces = fields.read().edge_faces.find_from_key(edge)?.clone();
    for adjacent in adjacent_faces.iter() {
        if face_is_external(context, face, adjacent, candidates)? {
            count += 1;
        }
    }
    Ok(count)
}

fn face_is_external(
    context: &ShapeUpgradeRemoveInternalWires,
    face: &Shape,
    adjacent: &Shape,
    candidates: &[Shape],
) -> Result<bool, Failure> {
    let applied = context
        .context()
        .get()
        .ok_or_else(null_context)?
        .apply(adjacent, ShapeEnum::Shape)?;
    Ok(!applied.is_null()
        && !face.is_same(&applied)
        && !candidates
            .iter()
            .any(|candidate| candidate.is_same(&applied)))
}

fn missing_edge_faces(edge: &Shape) -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        &format!(
            "ShapeUpgrade_RemoveInternalWires: removed edge {:?} has no face map entry",
            edge.shape_type()
        ),
    )
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod cursor_tests;
