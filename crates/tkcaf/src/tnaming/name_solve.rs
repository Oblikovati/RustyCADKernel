// Rust port of TNaming_Name::Solve and the IDENTITY, MODIFUNTIL, CONSTSHAPE and INTERSECTION
// algorithms of ApplicationFramework/TKCAF/TNaming/TNaming_Name.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Solving a `TNaming_Name`: each name type rebuilds the selection on the name's label
//! (the other algorithms are in `name_union.rs`, `name_generated.rs`, `name_filters.rs` and
//! `name_containers.rs`).
use super::name_union::shape_with_type;
use super::tool_shapes::make_shape;
use super::{
    TNamingBuilder, TNamingIterator, TNamingName, TNamingNameType, TNamingNamedShape,
    TNamingNamingTool, TNamingShapesSet,
};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfLabelMap};

/// The arguments of a name.
pub(crate) type NameArgs = [Handle<TNamingNamedShape>];

impl TNamingName {
    /// `Solve(aLab, Valid)`: rebuilds the selection on `aLab` with the algorithm of the name
    /// type. Every failure inside is caught, as natively (`SUBSTRACTION` is not implemented):
    /// the answer is then false.
    pub fn solve(&self, label: &TdfLabel, valid: &TdfLabelMap) -> bool {
        self.solve_by_type(label, valid).unwrap_or(false)
    }
    fn solve_by_type(&self, label: &TdfLabel, valid: &TdfLabelMap) -> Result<bool, Failure> {
        let (args, stop, index) = (self.arguments(), &self.stop_named_shape(), self.index());
        match self.name_type() {
            TNamingNameType::Unknown => Ok(false),
            TNamingNameType::Identity => identity(label, valid, args, self.shape_type()),
            TNamingNameType::ModifUntil => modif_until(label, valid, args, stop),
            TNamingNameType::Generation => super::name_generated::generated(label, valid, args),
            TNamingNameType::Intersection => {
                intersection(label, valid, args, stop, self.shape_type(), index)
            }
            TNamingNameType::Union => super::name_union::union(
                label,
                valid,
                args,
                stop,
                self.shape_type(),
                self.context_label(),
            ),
            TNamingNameType::Substraction => Err(exceptions::not_implemented(
                "TNaming_Name::Solve: SUBSTRACTION is not implemented natively",
            )),
            TNamingNameType::ConstShape => const_shape(label, valid, args, stop, index),
            TNamingNameType::FilterByNeighbourgs => super::name_filters::filter_by_neighbourgs(
                label,
                valid,
                args,
                stop,
                self.shape_type(),
            ),
            TNamingNameType::Orientation => {
                super::name_filters::orientation(label, valid, args, stop, index)
            }
            TNamingNameType::WireIn => {
                super::name_containers::wire_in(label, valid, args, stop, index)
            }
            TNamingNameType::ShellIn => {
                super::name_containers::shell_in(label, valid, args, stop, index)
            }
        }
    }
}

/// The file-static `ValidArgs`: every argument is a valid, non-empty named shape.
pub(crate) fn valid_args(args: &NameArgs) -> bool {
    args.iter().all(|argument| {
        argument
            .get()
            .is_some_and(|named| !named.is_empty() && named.is_valid())
    })
}

/// `NCollection_List::First`/`Last` of the arguments: `Standard_NoSuchObject` when empty.
pub(crate) fn argument(
    args: &NameArgs,
    index: usize,
) -> Result<&Handle<TNamingNamedShape>, Failure> {
    args.get(index).ok_or_else(|| {
        exceptions::no_such_object(
            "TNaming_Name::Solve: the name has too few arguments; expected one more",
        )
    })
}

/// The descendants of the stop named shape (`TNaming_NamingTool::BuildDescendants`).
pub(crate) fn forbidden_by(stop: &Handle<TNamingNamedShape>) -> Result<TdfLabelMap, Failure> {
    let mut forbidden = TdfLabelMap::default();
    TNamingNamingTool::build_descendants(stop, &mut forbidden)?;
    Ok(forbidden)
}

/// `TNaming_NamingTool::CurrentShape` into a new map.
pub(crate) fn current_shapes(
    valid: &TdfLabelMap,
    forbidden: &TdfLabelMap,
    named: &Handle<TNamingNamedShape>,
) -> Result<IndexedMapOfShape, Failure> {
    let mut shapes = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape(valid, forbidden, named, &mut shapes)?;
    Ok(shapes)
}

/// Selects each shape of a map in itself.
pub(crate) fn select_all(
    builder: &TNamingBuilder,
    shapes: &IndexedMapOfShape,
) -> Result<(), Failure> {
    for index in 1..=shapes.extent() {
        let shape = shapes.find_key(index)?;
        builder.select(shape, shape)?;
    }
    Ok(())
}

/// IDENTITY: the current shapes of the argument (of the name's shape type).
fn identity(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    shape_type: ShapeEnum,
) -> Result<bool, Failure> {
    if args.len() > 2 {
        return Err(exceptions::construction_error(&format!(
            "TNaming_Name::Solve: IDENTITY with {} arguments; expected at most 2",
            args.len()
        )));
    }
    if !valid_args(args) {
        return Ok(false);
    }
    let shapes = current_shapes(valid, &TdfLabelMap::default(), argument(args, 0)?)?;
    let builder = TNamingBuilder::new(label)?;
    for index in 1..=shapes.extent() {
        let shape = shape_with_type(shapes.find_key(index)?, shape_type)?;
        builder.select(&shape, &shape)?;
    }
    Ok(true)
}

/// MODIFUNTIL: the last modifications of the last argument, except through the descendants
/// of the stop shape (the same as IDENTITY without a stop shape).
fn modif_until(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
) -> Result<bool, Failure> {
    if !valid_args(args) {
        return Ok(false);
    }
    let forbidden = forbidden_by(stop)?;
    let last = args.last().ok_or_else(|| {
        exceptions::no_such_object("TNaming_Name::Solve: MODIFUNTIL without argument; expected one")
    })?;
    let shapes = current_shapes(valid, &forbidden, last)?;
    select_all(&TNamingBuilder::new(label)?, &shapes)?;
    Ok(true)
}

/// CONSTSHAPE: the new shape of the first argument at the name's index, and its last
/// modifications outside the descendants of the stop shape.
fn const_shape(
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
    let mut shape = Shape::default();
    let mut iterator = TNamingIterator::new(argument(args, 0)?)?;
    let mut position = 1;
    while iterator.more() {
        if index == position {
            shape = iterator.new_shape()?;
            break;
        }
        iterator.next()?;
        position += 1;
    }
    if shape.is_null() {
        return Ok(false);
    }
    let mut shapes = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape_from_shape(valid, &forbidden, label, &shape, &mut shapes)?;
    select_all(&TNamingBuilder::new(label)?, &shapes)?;
    Ok(true)
}

/// INTERSECTION: the sub-shapes of the name's type common to the current shapes of every
/// argument; for an edge with an index, the edge the index encodes (face, wire and edge
/// positions) when the description is exact.
fn intersection(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
    shape_type: ShapeEnum,
    index: i32,
) -> Result<bool, Failure> {
    if args.is_empty() || !valid_args(args) {
        return Ok(false);
    }
    let forbidden = forbidden_by(stop)?;
    let mut ancestors = Vec::new();
    let first = make_shape(&current_shapes(valid, &forbidden, &args[0])?)?;
    let mut common = TNamingShapesSet::from_shape(&first, Some(shape_type))?;
    ancestors.push(first);
    for named in &args[1..] {
        let current = make_shape(&current_shapes(valid, &forbidden, named)?)?;
        common.filter(&TNamingShapesSet::from_shape(&current, Some(shape_type))?);
        ancestors.push(current);
    }
    let builder = TNamingBuilder::new(label)?;
    let indexed = common.map().extent() > 1 && index > 0 && shape_type == ShapeEnum::Edge;
    if indexed && select_indexed_edge(&builder, &ancestors, index)? {
        return Ok(true);
    }
    let mut done = false;
    for shape in common.map().iter() {
        builder.select(shape, shape)?;
        done = true;
    }
    Ok(done)
}

/// The edge an intersection index encodes: bits 0-7 the edge, 8-15 the number of edges of
/// the wire, 16-19 the wire, 20-23 the number of wires of the face, 24-27 the argument of the
/// face. Only an exact description (the same numbers of wires and edges) selects it.
fn select_indexed_edge(
    builder: &TNamingBuilder,
    ancestors: &[Shape],
    index: i32,
) -> Result<bool, Failure> {
    let edge_index = index & 0xFF;
    let nb_edges = (index & 0xFF00) >> 8;
    let wire_index = (index & 0x000F_0000) >> 16;
    let nb_wires = (index & 0x00F0_0000) >> 20;
    let face_index = (index & 0x0F00_0000) >> 24;
    let face = usize::try_from(face_index - 1)
        .ok()
        .and_then(|position| ancestors.get(position))
        .cloned()
        .unwrap_or_default();
    if face.nb_children() != nb_wires {
        return Ok(false);
    }
    let wire = nth_child(&face, wire_index);
    if wire.nb_children() != nb_edges {
        return Ok(false);
    }
    let edge = nth_child(&wire, edge_index);
    if edge.is_null() {
        return Ok(false);
    }
    builder.select(&edge, &edge)?;
    Ok(true)
}

/// The child of a shape at a 1-based position, null when there is none.
pub(crate) fn nth_child(shape: &Shape, position: i32) -> Shape {
    usize::try_from(position - 1)
        .ok()
        .and_then(|position| ShapeIterator::new(shape, true, true).nth(position))
        .unwrap_or_default()
}
