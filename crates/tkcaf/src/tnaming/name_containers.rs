// Rust port of the WIREIN and SHELLIN algorithms of
// ApplicationFramework/TKCAF/TNaming/TNaming_Name.cxx (WireIN, ShellIN)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The WIREIN and SHELLIN names: a wire of a face (a shell of a solid) given by its outer
//! position or by its edges (faces). The two native functions are the same code over
//! (face, wire, edge) and (solid, shell, face); they share one body here.
use super::name_solve::{NameArgs, argument, current_shapes, forbidden_by, valid_args};
use super::tool_shapes::make_shape;
use super::{TNaming, TNamingBuilder, TNamingNamedShape, TNamingShapesSet};
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Shell, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfLabel, TdfLabelMap};

/// The types and the outer-boundary function of one container name.
struct ContainerKind {
    /// The selected sub-shape type (wire, shell).
    part: ShapeEnum,
    /// The type of the shapes that identify it (edge, face).
    piece: ShapeEnum,
    /// `TNaming::OuterWire` or `TNaming::OuterShell`, as a shape.
    outer: fn(&Shape) -> Result<Shape, Failure>,
}

/// WIREIN: a wire of the face of the first argument.
pub(crate) fn wire_in(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
    index: i32,
) -> Result<bool, Failure> {
    let kind = ContainerKind {
        part: ShapeEnum::Wire,
        piece: ShapeEnum::Edge,
        outer: outer_wire,
    };
    solve_container(&kind, label, valid, args, stop, index)
}

/// SHELLIN: a shell of the solid of the first argument.
pub(crate) fn shell_in(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
    index: i32,
) -> Result<bool, Failure> {
    let kind = ContainerKind {
        part: ShapeEnum::Shell,
        piece: ShapeEnum::Face,
        outer: outer_shell,
    };
    solve_container(&kind, label, valid, args, stop, index)
}

/// `TNaming::OuterWire` of a face; a null wire when there is none.
fn outer_wire(face: &Shape) -> Result<Shape, Failure> {
    let mut wire = Wire::default();
    TNaming::outer_wire(&topods::face(face)?, &mut wire)?;
    Ok(wire.into_shape())
}

/// `TNaming::OuterShell` of a solid.
fn outer_shell(solid: &Shape) -> Result<Shape, Failure> {
    let mut shell = Shell::default();
    TNaming::outer_shell(&topods::solid(solid)?, &mut shell)?;
    Ok(shell.into_shape())
}

/// The common body: with index 1 the outer part; else the part whose pieces are exactly the
/// pieces of the other arguments, or, without one, every part but the outer one (the answer
/// is then false, as natively).
fn solve_container(
    kind: &ContainerKind,
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
    index: i32,
) -> Result<bool, Failure> {
    if !valid_args(args) {
        return Ok(false);
    }
    if args.is_empty() {
        return Err(exceptions::construction_error(
            "TNaming_Name::Solve: a container name without argument; expected at least 1",
        ));
    }
    let containers = current_shapes(valid, &TdfLabelMap::default(), argument(args, 0)?)?;
    if containers.extent() != 1 {
        return Ok(false);
    }
    let container = containers.find_key(1)?.clone();
    let builder = TNamingBuilder::new(label)?;
    if index == 1 {
        let outer = (kind.outer)(&container)?;
        if outer.is_null() {
            return Ok(false);
        }
        builder.select(&outer, &outer)?;
        return Ok(true);
    }
    let pieces = pieces_of(kind, valid, &args[1..], stop)?;
    for part in ShapeIterator::new(&container, true, true) {
        if is_part_of_pieces(kind, &part, &pieces)? {
            builder.select(&part, &part)?;
            return Ok(true);
        }
    }
    select_inner_parts(kind, &builder, &container)?;
    Ok(false)
}

/// The pieces (edges, faces) of the current shapes of the identifying arguments.
fn pieces_of(
    kind: &ContainerKind,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
) -> Result<TNamingShapesSet, Failure> {
    let forbidden = forbidden_by(stop)?;
    let first = make_shape(&current_shapes(valid, &forbidden, argument(args, 0)?)?)?;
    let mut pieces = TNamingShapesSet::from_shape(&first, Some(kind.piece))?;
    for named in &args[1..] {
        let current = make_shape(&current_shapes(valid, &forbidden, named)?)?;
        pieces.add_set(&TNamingShapesSet::from_shape(&current, Some(kind.piece))?);
    }
    Ok(pieces)
}

/// A child of the container of the part type whose children are the pieces: the same
/// number, every piece among them.
fn is_part_of_pieces(
    kind: &ContainerKind,
    part: &Shape,
    pieces: &TNamingShapesSet,
) -> Result<bool, Failure> {
    if part.is_null() || part.shape_type()? != kind.part {
        return Ok(false);
    }
    let mut view = MapOfShape::default();
    for child in ShapeIterator::new(part, true, true) {
        view.add(child);
    }
    let mut remaining = view.extent();
    if remaining == pieces.map().extent() {
        for piece in pieces.map().iter() {
            if view.contains(piece) {
                remaining -= 1;
            }
        }
    }
    Ok(remaining == 0)
}

/// The fallback: every part of the container but the outer one (when there is an outer
/// one).
fn select_inner_parts(
    kind: &ContainerKind,
    builder: &TNamingBuilder,
    container: &Shape,
) -> Result<(), Failure> {
    let outer = (kind.outer)(container)?;
    if outer.is_null() {
        return Ok(());
    }
    for part in ShapeIterator::new(container, true, true) {
        if part.is_null() || part.shape_type()? != kind.part || part.is_equal(&outer) {
            continue;
        }
        builder.select(&part, &part)?;
    }
    Ok(())
}
