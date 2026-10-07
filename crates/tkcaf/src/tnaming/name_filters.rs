// Rust port of the FILTERBYNEIGHBOURGS and ORIENTATION algorithms of
// ApplicationFramework/TKCAF/TNaming/TNaming_Name.cxx (FilterByNeighbourgs, ORientation,
// FindSubShapeInAncestor, Aggregation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The FILTERBYNEIGHBOURGS name (a candidate connected to every neighbour) and the
//! ORIENTATION name (the selection as it is oriented in its context).
use super::name_solve::{NameArgs, argument, current_shapes, forbidden_by, nth_child, valid_args};
use super::tool_shapes::make_shape;
use super::{TNamingBuilder, TNamingNamedShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfLabelMap};

/// FILTERBYNEIGHBOURGS: the candidates (current shapes of the first argument, the items of a
/// single compound) that share a boundary (an edge of a face, a vertex of an edge or vertex)
/// with a current shape of every other argument. The stop shape's own label stays allowed.
pub(crate) fn filter_by_neighbourgs(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
    shape_type: ShapeEnum,
) -> Result<bool, Failure> {
    let builder = TNamingBuilder::new(label)?;
    if !valid_args(args) {
        return Ok(false);
    }
    let mut forbidden = forbidden_by(stop)?;
    if let Some(stop) = stop.get() {
        forbidden.remove(&stop.label());
    }
    let candidates = candidates_of(
        &current_shapes(valid, &forbidden, argument(args, 0)?)?,
        shape_type,
    )?;
    // szy 31.03.10: a vertex candidate uses its vertices.
    let boundary_type = match shape_type {
        ShapeEnum::Edge | ShapeEnum::Vertex => ShapeEnum::Vertex,
        _ => ShapeEnum::Edge,
    };
    let mut done = false;
    for candidate in &candidates {
        let boundaries = boundaries_of(candidate, boundary_type)?;
        let mut keep = true;
        for neighbour in &args[1..] {
            let neighbours = current_shapes(valid, &forbidden, neighbour)?;
            if !connected(&neighbours, &boundaries, boundary_type)? {
                keep = false;
                break;
            }
        }
        if keep {
            builder.select(candidate, candidate)?;
            done = true;
        }
    }
    Ok(done)
}

/// The candidates: the current shapes, or the items of a single compound of another type.
fn candidates_of(shapes: &IndexedMapOfShape, shape_type: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let all: Vec<Shape> = (1..=shapes.extent())
        .map(|index| shapes.find_key(index).cloned())
        .collect::<Result<_, _>>()?;
    if let [single] = all.as_slice()
        && !single.is_null()
        && single.shape_type()? == ShapeEnum::Compound
        && shape_type != ShapeEnum::Compound
    {
        let mut items = IndexedMapOfShape::default();
        for item in ShapeIterator::new(single, true, true) {
            items.add(item);
        }
        return (1..=items.extent())
            .map(|index| items.find_key(index).cloned())
            .collect();
    }
    Ok(all)
}

/// The boundaries of a candidate: a vertex itself, else its sub-shapes of the boundary type.
fn boundaries_of(candidate: &Shape, boundary_type: ShapeEnum) -> Result<MapOfShape, Failure> {
    let mut boundaries = MapOfShape::default();
    if candidate.shape_type()? == ShapeEnum::Vertex {
        boundaries.add(candidate.clone());
        return Ok(boundaries);
    }
    let mut explorer = Explorer::new(candidate, boundary_type, ShapeEnum::Shape)?;
    while explorer.more() {
        boundaries.add(explorer.current().clone());
        explorer.next()?;
    }
    Ok(boundaries)
}

/// A neighbour has a boundary in common with the candidate.
fn connected(
    neighbours: &IndexedMapOfShape,
    boundaries: &MapOfShape,
    boundary_type: ShapeEnum,
) -> Result<bool, Failure> {
    for index in 1..=neighbours.extent() {
        let mut explorer =
            Explorer::new(neighbours.find_key(index)?, boundary_type, ShapeEnum::Shape)?;
        while explorer.more() {
            if boundaries.contains(explorer.current()) {
                return Ok(true);
            }
            explorer.next()?;
        }
    }
    Ok(false)
}

/// ORIENTATION: the selection (current shape of the first argument) found, with its
/// orientation, in its context (the current shape of the last argument; for a seam edge the
/// index picks the edge of the wire). A compound selection pairs each item with the context of
/// the next arguments.
pub(crate) fn orientation(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
    index: i32,
) -> Result<bool, Failure> {
    if !valid_args(args) {
        return Ok(false);
    }
    let forbidden = forbidden_by(stop)?;
    let shapes = current_shapes(valid, &forbidden, argument(args, 0)?)?;
    let split = shapes.extent() > 1;
    let shape = make_shape(&shapes)?;
    let builder = TNamingBuilder::new(label)?;
    if shape.is_null() {
        return Ok(false);
    }
    // A temporary native solution.
    let items: Vec<Shape> = if shape.shape_type()? == ShapeEnum::Compound && !split {
        ShapeIterator::new(&shape, true, true).collect()
    } else {
        Vec::new()
    };
    if items.is_empty() {
        let last = args.last().unwrap_or(&args[0]);
        let contexts = current_shapes(valid, &forbidden, last)?;
        return oriented_in_context(&builder, &shape, &contexts, index);
    }
    let pairs = item_contexts(&items, &args[1..], valid, &forbidden)?;
    if let [(item, context)] = pairs.as_slice() {
        return select_item(&builder, item, context);
    }
    for (item, context) in &pairs {
        if !select_item(&builder, item, context)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The single-selection branch of ORientation.
fn oriented_in_context(
    builder: &TNamingBuilder,
    shape: &Shape,
    contexts: &IndexedMapOfShape,
    index: i32,
) -> Result<bool, Failure> {
    if contexts.extent() != 1 {
        let context = make_shape(contexts)?;
        let found = find_sub_shape_in_ancestor(shape, &context)?;
        return select_or_aggregate(builder, shape, &found, &context);
    }
    let context = contexts.find_key(1)?.clone();
    let found = if index > 0 {
        seam_edge(shape, &context, index)
    } else {
        find_sub_shape_in_ancestor(shape, &context)?
    };
    select_or_aggregate(builder, shape, &found, &context)
}

/// The 21.10.2009 seam-edge case: the child at the index of a child of the context that is
/// the same as the selection.
fn seam_edge(shape: &Shape, context: &Shape, index: i32) -> Shape {
    for wire in ShapeIterator::new(context, true, true) {
        let candidate = nth_child(&wire, index);
        if !candidate.is_null() && candidate.is_same(shape) {
            return candidate;
        }
    }
    Shape::default()
}

/// The found shape, else the aggregation of the selection's items in the context.
fn select_or_aggregate(
    builder: &TNamingBuilder,
    shape: &Shape,
    found: &Shape,
    context: &Shape,
) -> Result<bool, Failure> {
    if !found.is_null() {
        builder.select(found, found)?;
        return Ok(true);
    }
    Ok(aggregation(shape, context, builder)? != 0)
}

/// Each compound item with the current shape (or compound) of the argument after the
/// first one at its position.
fn item_contexts(
    items: &[Shape],
    contexts: &NameArgs,
    valid: &TdfLabelMap,
    forbidden: &TdfLabelMap,
) -> Result<Vec<(Shape, Shape)>, Failure> {
    let mut pairs = Vec::new();
    for (position, item) in items.iter().enumerate() {
        let named = argument(contexts, position)?;
        let shapes = current_shapes(valid, forbidden, named)?;
        let context = if shapes.extent() == 1 {
            shapes.find_key(1)?.clone()
        } else {
            make_shape(&shapes)?
        };
        pairs.push((item.clone(), context));
    }
    Ok(pairs)
}

/// One item of the compound branch: itself in its context, or the aggregation of a compound.
fn select_item(builder: &TNamingBuilder, item: &Shape, context: &Shape) -> Result<bool, Failure> {
    if item.shape_type()? == ShapeEnum::Compound {
        return Ok(aggregation(item, context, builder)? != 0);
    }
    let found = find_sub_shape_in_ancestor(item, context)?;
    if found.is_null() {
        return Ok(false);
    }
    builder.select(&found, &found)?;
    Ok(true)
}

/// The file-static `FindSubShapeInAncestor(Selection, Context)`: the sub-shape of the context
/// that is the same as the selection, as oriented in the context; null for a compound.
fn find_sub_shape_in_ancestor(selection: &Shape, context: &Shape) -> Result<Shape, Failure> {
    let kind = selection.shape_type()?;
    if kind == ShapeEnum::Compound {
        return Ok(Shape::default());
    }
    let mut explorer = Explorer::new(context, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        if explorer.current().is_same(selection) {
            return Ok(explorer.current().clone());
        }
        explorer.next()?;
    }
    Ok(Shape::default())
}

/// The file-static `Aggregation(S, AS, B)`: each simple item of the (nested) compound found
/// in the context is selected; the number selected.
fn aggregation(shape: &Shape, context: &Shape, builder: &TNamingBuilder) -> Result<i32, Failure> {
    let mut count = 0;
    for item in ShapeIterator::new(shape, true, true) {
        if (item.shape_type()? as i32) > ShapeEnum::CompSolid as i32 {
            let found = find_sub_shape_in_ancestor(&item, context)?;
            if !found.is_null() {
                builder.select(&found, &found)?;
                count += 1;
            }
        } else {
            count += aggregation(&item, context, builder)?;
        }
    }
    Ok(count)
}
