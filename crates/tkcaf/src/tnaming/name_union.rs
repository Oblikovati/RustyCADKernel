// Rust port of the UNION algorithm of ApplicationFramework/TKCAF/TNaming/TNaming_Name.cxx
// (Union, KeepInList, ShapeWithType) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The UNION name: the last modifications of every argument, or the one sub-shape of the
//! context they make up.
use super::name_solve::{NameArgs, current_shapes, forbidden_by, valid_args};
use super::package::compound_of;
use super::tool_shapes::make_shape;
use super::{TNamingBuilder, TNamingNamedShape, TNamingShapesSet};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, CompSolid, Shape, ShapeIterator, Shell};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfLabel, TdfLabelMap};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeSolid,
    BRepBuilderAPIMakeWire,
};

/// UNION: the current shapes of every argument; when the context label has a sub-shape of
/// the name's type made of exactly those shapes (szy 27.05.08), that sub-shape, else their
/// compound brought to the name's type.
pub(crate) fn union(
    label: &TdfLabel,
    valid: &TdfLabelMap,
    args: &NameArgs,
    stop: &Handle<TNamingNamedShape>,
    shape_type: ShapeEnum,
    context_label: &TdfLabel,
) -> Result<bool, Failure> {
    if args.is_empty() || !valid_args(args) {
        return Ok(false);
    }
    // Temporary native solution for the orientation name: always on.
    let forbidden = forbidden_by(stop)?;
    let mut kept = Vec::new();
    let mut all = TNamingShapesSet::new();
    for named in args {
        let current = make_shape(&current_shapes(valid, &forbidden, named)?)?;
        keep_in_list(&current, shape_type, &mut kept)?;
        all.add_set(&TNamingShapesSet::from_shape(&current, Some(shape_type))?);
    }
    let candidate = context_candidate(valid, &forbidden, &all, shape_type, context_label)?;
    let builder = TNamingBuilder::new(label)?;
    if let Some(candidate) = candidate {
        builder.select(&candidate, &candidate)?;
        return Ok(true);
    }
    let shape = shape_with_type(&compound_of(kept.iter())?, shape_type)?;
    builder.select(&shape, &shape)?;
    Ok(true)
}

/// The szy 27.05.08 search: the first sub-shape of the context (the current shape of the
/// context label) whose faces (edges for a wire) are all in the set, as many as it holds.
fn context_candidate(
    valid: &TdfLabelMap,
    forbidden: &TdfLabelMap,
    all: &TNamingShapesSet,
    shape_type: ShapeEnum,
    context_label: &TdfLabel,
) -> Result<Option<Shape>, Failure> {
    if context_label.is_null() {
        return Ok(None);
    }
    let mut context_named: Handle<TNamingNamedShape> = Handle::default();
    context_label.find_attribute_as(&TNamingNamedShape::get_id(), &mut context_named)?;
    let context = if context_named.is_some() {
        make_shape(&current_shapes(valid, forbidden, &context_named)?)?
    } else {
        Shape::default()
    };
    let part_type = if shape_type == ShapeEnum::Wire {
        ShapeEnum::Edge
    } else {
        ShapeEnum::Face
    };
    for candidate in explore(&context, shape_type)? {
        let mut remaining = all.map().extent();
        for part in explore(&candidate, part_type)? {
            if all.map().contains(&part) {
                remaining -= 1;
            }
        }
        if remaining == 0 {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

/// The sub-shapes of a type, in exploration order (none for a null shape).
fn explore(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut shapes = Vec::new();
    if shape.is_null() {
        return Ok(shapes);
    }
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(shapes)
}

/// The file-static `KeepInList(CS, Type, aList)`: the shapes `TNaming_ShapesSet(CS, Type)`
/// would hold, in order and with repetitions.
fn keep_in_list(
    shape: &Shape,
    shape_type: ShapeEnum,
    list: &mut Vec<Shape>,
) -> Result<(), Failure> {
    if shape.is_null() {
        return Ok(());
    }
    let kind = shape.shape_type()?;
    if shape_type == ShapeEnum::Shape {
        if matches!(
            kind,
            ShapeEnum::Solid | ShapeEnum::Face | ShapeEnum::Edge | ShapeEnum::Vertex
        ) {
            list.push(shape.clone());
        } else {
            list.extend(ShapeIterator::new(shape, true, true));
        }
        return Ok(());
    }
    if shape_type as i32 > kind as i32 {
        list.extend(explore(shape, shape_type)?);
    } else {
        list.push(shape.clone());
    }
    Ok(())
}

/// The file-static `ShapeWithType(theShape, theType)`: the shape brought to a type: the
/// single sub-shape of that type of a compound or of its items, or a shape built from
/// simpler items (a wire from edges, faces from wires, a shell, solids, a compsolid); the
/// shape itself when that fails or is ambiguous.
pub(crate) fn shape_with_type(shape: &Shape, wanted: ShapeEnum) -> Result<Shape, Failure> {
    if shape.is_null() || wanted == ShapeEnum::Shape || shape.shape_type()? == wanted {
        return Ok(shape.clone());
    }
    let mut kind = shape.shape_type()?;
    let mut items = Vec::new();
    if kind == ShapeEnum::Compound {
        items.extend(ShapeIterator::new(shape, true, true));
        if let Some(first) = items.first() {
            kind = first.shape_type()?;
        }
        if kind == wanted {
            return Ok(if items.len() == 1 {
                items[0].clone()
            } else {
                shape.clone()
            });
        }
    } else {
        items.push(shape.clone());
    }
    if (kind as i32) < wanted as i32 {
        return Ok(single_sub_shape(&items, wanted)?.unwrap_or_else(|| shape.clone()));
    }
    Ok(build_up(kind, items, wanted)?.unwrap_or_else(|| shape.clone()))
}

/// The branch of `ShapeWithType` for items above the wanted type: the only sub-shape of that
/// type (the first of each item); `None` when there are several or none.
fn single_sub_shape(items: &[Shape], wanted: ShapeEnum) -> Result<Option<Shape>, Failure> {
    let mut result = None;
    for item in items {
        let explorer = Explorer::new(item, wanted, ShapeEnum::Shape)?;
        if explorer.more() && !explorer.current().is_null() {
            if result.is_some() {
                return Ok(None);
            }
            result = Some(explorer.current().clone());
        }
    }
    Ok(result)
}

/// The fall-through `switch` of `ShapeWithType`: builds a more complex shape from the items
/// step by step until the wanted type; `None` keeps the input shape.
fn build_up(
    kind: ShapeEnum,
    items: Vec<Shape>,
    wanted: ShapeEnum,
) -> Result<Option<Shape>, Failure> {
    let mut items = items;
    let mut step = kind;
    loop {
        let next = match step {
            ShapeEnum::Edge if (wanted as i32) <= ShapeEnum::Solid as i32 => return Ok(None),
            ShapeEnum::Edge => wire_from_edges(&items, wanted)?,
            ShapeEnum::Wire if (wanted as i32) < ShapeEnum::Solid as i32 => return Ok(None),
            ShapeEnum::Wire => faces_from_wires(&items, wanted)?,
            ShapeEnum::Face if (wanted as i32) < ShapeEnum::Solid as i32 => return Ok(None),
            ShapeEnum::Face => shell_from_faces(&items, wanted)?,
            ShapeEnum::Shell => solids_from_shells(&items, wanted)?,
            ShapeEnum::Solid => return compsolid_from_solids(&items, wanted),
            _ => return Ok(None),
        };
        match next {
            Step::Done(result) => return Ok(result),
            Step::Next(shapes, kind) => {
                items = shapes;
                step = kind;
            }
        }
    }
}

/// One step of `build_up`: the answer, or the items for the next case.
enum Step {
    Done(Option<Shape>),
    Next(Vec<Shape>, ShapeEnum),
}

/// The EDGE case: a wire from the edges.
fn wire_from_edges(items: &[Shape], wanted: ShapeEnum) -> Result<Step, Failure> {
    let mut maker = BRepBuilderAPIMakeWire::new();
    let edges: List<Shape> = items.iter().cloned().collect();
    maker.add_list(&edges)?;
    if !maker.is_done() {
        return Ok(Step::Done(None));
    }
    let wire = maker.wire()?.into_shape();
    if wanted == ShapeEnum::Wire {
        return Ok(Step::Done(Some(wire)));
    }
    Ok(Step::Next(vec![wire], ShapeEnum::Wire))
}

/// The WIRE case: one face per wire. The native test is inverted (a face is taken when the
/// maker is not done), so `Face()` raises `StdFail_NotDone`, which `Solve` catches.
fn faces_from_wires(items: &[Shape], wanted: ShapeEnum) -> Result<Step, Failure> {
    let mut faces = Vec::new();
    for item in items {
        let maker = BRepBuilderAPIMakeFace::from_wire(&topods::wire(item)?, false)?;
        if !maker.is_done() {
            faces.push(maker.face()?.into_shape());
        }
    }
    if wanted == ShapeEnum::Face {
        return Ok(Step::Done((faces.len() == 1).then(|| faces[0].clone())));
    }
    Ok(Step::Next(faces, ShapeEnum::Face))
}

/// The FACE case: a shell of the faces, closed when it is.
fn shell_from_faces(items: &[Shape], wanted: ShapeEnum) -> Result<Step, Failure> {
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell = shell.into_shape();
    for item in items {
        builder.add(&mut shell, &topods::face(item)?.into_shape())?;
    }
    shell.set_closed(BRepTool::is_closed(&shell)?)?;
    if wanted == ShapeEnum::Shell {
        return Ok(Step::Done(Some(shell)));
    }
    Ok(Step::Next(vec![shell], ShapeEnum::Shell))
}

/// The SHELL case: one solid per shell.
fn solids_from_shells(items: &[Shape], wanted: ShapeEnum) -> Result<Step, Failure> {
    let mut solids = Vec::new();
    for item in items {
        let maker = BRepBuilderAPIMakeSolid::from_shell(&topods::shell(item)?)?;
        if maker.is_done() {
            solids.push(maker.solid()?.into_shape());
        }
    }
    if wanted == ShapeEnum::Solid {
        return Ok(Step::Done((solids.len() == 1).then(|| solids[0].clone())));
    }
    Ok(Step::Next(solids, ShapeEnum::Solid))
}

/// The SOLID case: a compsolid of the solids, when that is the wanted type.
fn compsolid_from_solids(items: &[Shape], wanted: ShapeEnum) -> Result<Option<Shape>, Failure> {
    let builder = BRepBuilder::default();
    let mut compsolid = CompSolid::default();
    builder.make_comp_solid(&mut compsolid);
    let mut compsolid = compsolid.into_shape();
    for item in items {
        builder.add(&mut compsolid, &topods::solid(item)?.into_shape())?;
    }
    Ok((wanted == ShapeEnum::CompSolid).then_some(compsolid))
}
