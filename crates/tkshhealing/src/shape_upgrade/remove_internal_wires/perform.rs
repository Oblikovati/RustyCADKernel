use super::*;
use crate::shape_analysis::ShapeAnalysis;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::{
    topexp::{self, Explorer},
    topods::{self, ShapeIterator},
};
use rustycad_tkernel::ncollection::Sequence;

impl ShapeUpgradeRemoveInternalWires {
    /// Removes every eligible internal wire in the initialized shape; `remover.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        let mut fields = self.fields.write();
        clear(&mut fields);
        if fields.shape.is_null() {
            return fail_initial_shape(&mut fields);
        }
        let context_handle = self.context();
        let context = context_handle.get().ok_or_else(null_context)?;
        let input = fields.shape.clone();
        scan_all_faces(&mut fields, context, &input)?;
        finish_perform(&mut fields, context, &input)
    }

    /// Processes only listed faces and wires; `sequence.append(face); remover.perform_on(&sequence)?`.
    pub fn perform_on(&self, shapes: &Sequence<Shape>) -> Result<bool, Failure> {
        let mut fields = self.fields.write();
        if fields.shape.is_null() {
            return fail_initial_shape(&mut fields);
        }
        clear(&mut fields);
        let context_handle = self.context();
        let context = context_handle.get().ok_or_else(null_context)?;
        let input = fields.shape.clone();
        scan_selected_shapes(&mut fields, context, &input, shapes)?;
        finish_perform(&mut fields, context, &input)
    }
}

fn scan_all_faces(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    input: &Shape,
) -> Result<(), Failure> {
    for face in Explorer::new(input, ShapeEnum::Face, ShapeEnum::Shape)? {
        remove_small_wire(fields, context, &face, None)?;
    }
    if fields.remove_face_mode {
        remove_small_faces(fields, context)?;
    }
    Ok(())
}

fn scan_selected_shapes(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    input: &Shape,
    shapes: &Sequence<Shape>,
) -> Result<(), Failure> {
    let mut wire_faces = None;
    for shape in shapes.iter() {
        process_selected_shape(fields, context, input, shape, &mut wire_faces)?;
    }
    if fields.remove_face_mode {
        remove_small_faces(fields, context)?;
    }
    Ok(())
}

fn finish_perform(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    input: &Shape,
) -> Result<bool, Failure> {
    fields.result = context.apply(input, ShapeEnum::Shape)?;
    Ok(ShapeExtend::decode_status(
        fields.status,
        ShapeExtendStatus::Done,
    ))
}

fn process_selected_shape(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
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
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
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

fn clear(fields: &mut RemoveInternalWiresFields) {
    fields.remove_edges.clear(true);
    fields.removed_faces.clear();
    fields.removed_wires.clear();
    fields.status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
}

fn fail_initial_shape(fields: &mut RemoveInternalWiresFields) -> Result<bool, Failure> {
    fields.status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail1);
    Ok(false)
}

fn remove_small_wire(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
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
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    face: &rustycad_tkbrep::topods::Face,
    wire: &Shape,
) -> Result<(), Failure> {
    if !wire_is_small(fields, wire)? {
        return Ok(());
    }
    context.remove(wire)?;
    fields.removed_wires.push(wire.clone());
    fields.status |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
    if !fields.remove_face_mode {
        return Ok(());
    }
    record_removed_edges(fields, wire, &face.clone().into_shape());
    Ok(())
}

fn wire_is_small(fields: &RemoveInternalWiresFields, wire: &Shape) -> Result<bool, Failure> {
    let area = ShapeAnalysis::contour_area(&topods::wire(wire)?)?;
    let threshold = fields.min_area - rustycad_tkernel::precision::confusion();
    Ok(area
        .partial_cmp(&threshold)
        .is_some_and(|order| order.is_lt()))
}

fn record_removed_edges(fields: &mut RemoveInternalWiresFields, wire: &Shape, face: &Shape) {
    for edge in ShapeIterator::new(wire, false, true) {
        append_removed_edge_face(fields, &edge, face);
    }
}

fn append_removed_edge_face(fields: &mut RemoveInternalWiresFields, edge: &Shape, face: &Shape) {
    if let Some(faces) = fields.remove_edges.change_seek(edge) {
        faces.append(face.clone());
        return;
    }
    let mut faces = List::default();
    faces.append(face.clone());
    fields.remove_edges.add(edge.clone(), faces);
}

fn remove_small_faces(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
) -> Result<(), Failure> {
    for wire in fields.removed_wires.clone() {
        let mut candidates = Vec::new();
        gather_face_candidates(fields, context, &wire, &mut candidates)?;
        remove_empty_candidates(fields, context, &candidates)?;
    }
    if !fields.removed_faces.is_empty() {
        fields.status |= ShapeExtend::encode_status(ShapeExtendStatus::Done2);
    }
    Ok(())
}

fn gather_face_candidates(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    wire: &Shape,
    candidates: &mut Vec<Shape>,
) -> Result<(), Failure> {
    for edge in ShapeIterator::new(wire, false, true) {
        collect_edge_candidates(fields, context, &edge, candidates)?;
    }
    Ok(())
}

fn collect_edge_candidates(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    edge: &Shape,
    candidates: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let Some(faces) = fields.edge_faces.seek(edge) else {
        fields.status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail2);
        return Ok(());
    };
    let removed = fields
        .remove_edges
        .seek(edge)
        .ok_or_else(|| missing_edge_faces(edge))?;
    let mut removed_cursor = 0;
    for face in faces.iter() {
        collect_candidate(
            context,
            face,
            removed,
            &mut removed_cursor,
            edge,
            candidates,
        )?;
    }
    Ok(())
}

fn collect_candidate(
    context: &ShapeBuildReShape,
    face: &Shape,
    removed: &List<Shape>,
    removed_cursor: &mut usize,
    edge: &Shape,
    candidates: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let applied = context.apply(face, ShapeEnum::Shape)?;
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
    context: &ShapeBuildReShape,
    candidate: &Shape,
    removed: &List<Shape>,
    cursor: &mut usize,
) -> Result<bool, Failure> {
    for (index, old) in removed.iter().enumerate().skip(*cursor) {
        let applied = context.apply(old, ShapeEnum::Shape)?;
        if candidate.is_same(&applied) {
            *cursor = index;
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
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    candidates: &[Shape],
) -> Result<(), Failure> {
    for face in candidates {
        if has_nonremoved_neighbor(fields, context, face, candidates)? {
            continue;
        }
        context.remove(face)?;
        fields.removed_faces.push(face.clone());
    }
    Ok(())
}

fn has_nonremoved_neighbor(
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
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
    fields: &mut RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    face: &Shape,
    candidates: &[Shape],
    wire_data: &crate::shape_extend::ShapeExtendWireData,
    index: i32,
) -> Result<usize, Failure> {
    if wire_data.is_seam(index) {
        return Ok(0);
    }
    let edge = wire_data.edge(index)?.into_shape();
    if fields.remove_edges.contains(&edge) {
        return Ok(0);
    }
    count_external_adjacent_faces(fields, context, face, candidates, &edge)
}

fn count_external_adjacent_faces(
    fields: &RemoveInternalWiresFields,
    context: &ShapeBuildReShape,
    face: &Shape,
    candidates: &[Shape],
    edge: &Shape,
) -> Result<usize, Failure> {
    let mut count = 0;
    for adjacent in fields.edge_faces.find_from_key(edge)?.iter() {
        if face_is_external(context, face, adjacent, candidates)? {
            count += 1;
        }
    }
    Ok(count)
}

fn face_is_external(
    context: &ShapeBuildReShape,
    face: &Shape,
    adjacent: &Shape,
    candidates: &[Shape],
) -> Result<bool, Failure> {
    let applied = context.apply(adjacent, ShapeEnum::Shape)?;
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
