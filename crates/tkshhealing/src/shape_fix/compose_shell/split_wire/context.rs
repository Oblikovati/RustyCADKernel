use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::{ShapeFixWireSegment, WirePatchIndex};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn apply_context(
    wire: &ShapeFixWireSegment,
    index: i32,
    context: &Handle<ShapeBuildReShape>,
) -> Result<i32, Failure> {
    let source = wire.edge(index)?;
    let reshape = context.get().ok_or_else(null_context)?;
    let applied = reshape.apply(&source.clone().into_shape(), ShapeEnum::Shape)?;
    if applied.is_same(&source.clone().into_shape()) {
        return Ok(1);
    }
    replace_context_edge(wire, index, &source, &applied)
}

fn replace_context_edge(
    wire: &ShapeFixWireSegment,
    index: i32,
    source: &Edge,
    applied: &Shape,
) -> Result<i32, Failure> {
    if applied.shape_type()? == ShapeEnum::Edge {
        wire.set_edge(index, &topods::edge(applied)?)?;
        return Ok(1);
    }
    let patch = wire.patch_index(index)?;
    let replacements = replacement_edges(applied, source.orientation())?;
    insert_context_edges(wire, index, &replacements, patch)?;
    Ok(replacements.len() as i32)
}

fn replacement_edges(applied: &Shape, orientation: Orientation) -> Result<Vec<Edge>, Failure> {
    let mut children = ShapeIterator::new(applied, true, true);
    let mut replacements = Vec::new();
    while children.more() {
        replacements.push(topods::edge(children.value()?)?);
        children.next();
    }
    reverse_reversed_edges(&mut replacements, orientation);
    Ok(replacements)
}

fn reverse_reversed_edges(edges: &mut [Edge], orientation: Orientation) {
    if !matches!(orientation, Orientation::Forward | Orientation::Internal) {
        edges.reverse();
    }
}

fn insert_context_edges(
    wire: &ShapeFixWireSegment,
    index: i32,
    edges: &[Edge],
    patch: WirePatchIndex,
) -> Result<(), Failure> {
    let Some(first) = edges.first() else {
        return Ok(());
    };
    wire.set_edge(index, first)?;
    for (offset, edge) in edges.iter().enumerate().skip(1) {
        wire.add_edge_with_patch(index + offset as i32, edge, patch)?;
    }
    Ok(())
}

pub(super) fn distribute_split_points(
    wire: &ShapeFixWireSegment,
    face: rustycad_tkbrep::topods::Face,
    index: i32,
    count: i32,
    indices: &mut Sequence<i32>,
    values: &Sequence<f64>,
) -> Result<(), Failure> {
    let (parameters, reversed) = split_edge_starts(wire, &face, index, count)?;
    let mut point = first_index_at_or_after(indices, index);
    redistribute_same_edge_points(
        indices,
        values,
        &parameters,
        reversed,
        index,
        count,
        &mut point,
    )?;
    shift_later_indices(indices, point, count - 1)
}

fn split_edge_starts(
    wire: &ShapeFixWireSegment,
    face: &rustycad_tkbrep::topods::Face,
    index: i32,
    count: i32,
) -> Result<(Vec<f64>, bool), Failure> {
    let reversed = count > 0 && wire.edge(index)?.orientation() == Orientation::Reversed;
    let parameters = (0..count)
        .map(|offset| {
            let edge = wire.edge(index + offset)?;
            let (first, last) = BRepTool::range_on_face(&edge, face)?;
            Ok(if reversed { last } else { first })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((parameters, reversed))
}

fn first_index_at_or_after(indices: &Sequence<i32>, index: i32) -> i32 {
    (1..=indices.length())
        .find(|position| {
            indices
                .value(*position as usize)
                .is_ok_and(|value| *value >= index)
        })
        .unwrap_or(indices.length() + 1)
}

fn redistribute_same_edge_points(
    indices: &mut Sequence<i32>,
    values: &Sequence<f64>,
    parameters: &[f64],
    reversed: bool,
    index: i32,
    count: i32,
    point: &mut i32,
) -> Result<(), Failure> {
    let mut shift = 1;
    while *point <= indices.length() && index_at(indices, *point)? == index {
        while shift < count && reversed != (value_at(values, *point)? > parameters[shift as usize])
        {
            shift += 1;
        }
        indices.set_value(*point as usize, index + shift - 1)?;
        *point += 1;
    }
    Ok(())
}

fn shift_later_indices(indices: &mut Sequence<i32>, point: i32, shift: i32) -> Result<(), Failure> {
    for position in point..=indices.length() {
        let value = index_at(indices, position)?;
        indices.set_value(position as usize, value + shift)?;
    }
    Ok(())
}

fn index_at(indices: &Sequence<i32>, position: i32) -> Result<i32, Failure> {
    indices.value(position as usize).copied()
}

fn value_at(values: &Sequence<f64>, position: i32) -> Result<f64, Failure> {
    values.value(position as usize).copied()
}

fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitWire: null context",
    )
}
