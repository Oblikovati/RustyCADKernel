// Rust translation of the topology commands of Draw/TKDraw/DBRep/DBRep.cxx (tclean,
// compound, emptycopy, add, explode, exwire, invert, orientation, setflags, purgemmgt,
// check, removeinternals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Commands that build, explode or modify shape variables.
mod nexplode;

use super::{CommandEntry, sub_name};
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{self, WireExplorer};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator, TShape, wire as to_wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) use nexplode::NEXPLODE;

/// `tclean`.
pub(super) const TCLEAN: CommandEntry = (
    "tclean",
    "tclean [-force] [-geom] [name1]..., depending on using or not key -geom, \n\t\t -geom  : erase geometry\n\t\t          if [-geom] is omitted - erase triangulations and \n\t\t          polygons on triangulations from shapes \n\t\t -force : force delete all representations not relevant to the given shape \n",
    tclean,
);
/// `compound`.
pub(super) const COMPOUND: CommandEntry =
    ("compound", "compound [name1 name2 ..] compound", compound);
/// `add`.
pub(super) const ADD: CommandEntry = (
    "add",
    "add what where\n  adds shape \"what\" to shape \"where\" ",
    add,
);
/// `explode`.
pub(super) const EXPLODE: CommandEntry = ("explode", "explode name [Cd/C/So/Sh/F/W/E/V]", explode);
/// `exwire`.
pub(super) const EXWIRE: CommandEntry = ("exwire", "exwire wirename", exwire);
/// `emptycopy`.
pub(super) const EMPTYCOPY: CommandEntry = (
    "emptycopy",
    "emptycopy [copyshape] originalshape",
    emptycopy,
);
/// `check`.
pub(super) const CHECK: CommandEntry = ("check", "check shape1 shape2 ...", check);
/// `orientation`.
pub(super) const ORIENTATION: CommandEntry = (
    "orientation",
    "orientation name1 name2.. F/R/E/I",
    orientation,
);
/// `treverse`.
pub(super) const TREVERSE: CommandEntry = ("treverse", "treverse name1 name2 ...", orientation);
/// `complement`.
pub(super) const COMPLEMENT: CommandEntry =
    ("complement", "complement name1 name2 ...", orientation);
/// `invert`.
pub(super) const INVERT: CommandEntry = ("invert", "invert name, reverse subshapes", invert);
/// `setflags`.
pub(super) const SETFLAGS: CommandEntry = (
    "setflags",
    "setflags shape_name flag1[flag2...]\n sets flags for shape(free, modified, checked, orientable, closed, infinite, convex, locked), for example <setflags a free> or <setflags a -free> if necessary unflag ",
    set_flags,
);
/// `purgemmgt`.
pub(super) const PURGEMMGT: CommandEntry = (
    "purgemmgt",
    "returns the free memory from the system to the memory manager",
    purgemmgt,
);
/// `removeinternals`.
pub(super) const REMOVEINTERNALS: CommandEntry = (
    "removeinternals",
    "removeinternals shape [force flag {0/1}]\n\t\t             Removes sub-shapes with internal orientation from the shape.\n\n\t\t             Force flag disables the check on topological connectivity andremoves all internal sub-shapes\n",
    remove_internals,
);

/// `tclean [-force] [-geom] names...`: removes the triangulations and polygons (or the
/// geometry with `-geom`) of the shapes.
fn tclean(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() <= 1 {
        di.print_help(a[0])?;
        return Ok(1);
    }
    let (mut geometry, mut force) = (false, false);
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for argument in &a[1..] {
        match *argument {
            "-geom" => geometry = true,
            "-force" => force = true,
            name => {
                let shape = session.shape(name)?;
                if shape.is_null() {
                    di.append(format!("Syntax error : NULL shape '{name}'"))?;
                    return Ok(1);
                }
                builder.add(&mut compound, &shape)?;
            }
        }
    }
    if compound.nb_children() == 0 {
        di.append("Syntax error : wrong number of arguments")?;
        di.print_help(a[0])?;
        return Ok(1);
    }
    if force && geometry {
        di.append(
            "Syntax error: wrong usage of arguments: do not use 'force' and 'geom' flags together",
        )?;
        return Ok(1);
    }
    if geometry {
        brep_tools::clean_geometry(&compound)?;
    } else {
        brep_tools::clean(&compound, force)?;
    }
    Ok(0)
}

/// `compound names... result`: a compound of the non-null shapes.
fn compound(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let n = a.len();
    if n <= 1 {
        return Ok(1);
    }
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for name in &a[1..n - 1] {
        let shape = session.shape(name)?;
        if !shape.is_null() {
            builder.add(&mut compound, &shape)?;
        }
    }
    session.set_shape(a[n - 1], &compound)?;
    Ok(0)
}

/// `emptycopy [copy] original`: an empty copy (same TShape kind, no children).
fn emptycopy(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let n = a.len();
    if n <= 1 {
        return Ok(1);
    }
    let mut shape = session.shape(a[if n == 2 { 1 } else { 2 }])?;
    if shape.is_null() {
        return Ok(1);
    }
    shape.empty_copy()?;
    session.set_shape(a[1], &shape)?;
    Ok(0)
}

/// `add what where`: adds `what` to `where`.
fn add(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() < 3 {
        return Ok(1);
    }
    let what = session.shape(a[1])?;
    if what.is_null() {
        return Ok(1);
    }
    let mut target = session.shape(a[2])?;
    if target.is_null() {
        return Ok(1);
    }
    BRepBuilder::default().add(&mut target, &what)?;
    session.set_shape(a[2], &target)?;
    Ok(0)
}

/// The shape kind of an `explode` type argument (`Cd`, `C`, `So`, `Sh`, `F`, `W`, `E`,
/// `V`, first letters, any case).
fn explode_kind(text: &str) -> Option<ShapeEnum> {
    let bytes = text.as_bytes();
    let second = bytes.get(1).copied().unwrap_or(0).to_ascii_uppercase();
    Some(match bytes.first()?.to_ascii_uppercase() {
        b'C' if second == b'D' => ShapeEnum::Compound,
        b'C' => ShapeEnum::CompSolid,
        b'S' if second == b'O' => ShapeEnum::Solid,
        b'S' if second == b'H' => ShapeEnum::Shell,
        b'F' => ShapeEnum::Face,
        b'W' => ShapeEnum::Wire,
        b'E' => ShapeEnum::Edge,
        b'V' => ShapeEnum::Vertex,
        _ => return None,
    })
}

/// `explode name [type]`: `name_1`, `name_2`... for the children, or for the distinct
/// sub-shapes of a type; the names are appended to the result.
fn explode(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let n = a.len();
    if n <= 1 {
        return Ok(1);
    }
    let shape = session.shape(a[1])?;
    if shape.is_null() {
        return Ok(0);
    }
    let parts = if n == 2 {
        children(&shape)?
    } else {
        let Some(kind) = explode_kind(a[2]) else {
            return Ok(1);
        };
        distinct_sub_shapes(&shape, kind)?
    };
    for (index, part) in parts.iter().enumerate() {
        let name = sub_name(a[1], index + 1);
        session.set_shape(&name, part)?;
        di.append_element(&name)?;
    }
    Ok(0)
}

/// The children of `shape` (`TopoDS_Iterator`, cumulated orientation and location).
fn children(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut parts = Vec::new();
    let mut iterator = ShapeIterator::new(shape, true, true);
    while iterator.more() {
        parts.push(iterator.value()?.clone());
        iterator.next();
    }
    Ok(parts)
}

/// The sub-shapes of `kind`, each once (`IsSame`), the shape itself excluded.
fn distinct_sub_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut seen = MapOfShape::default();
    seen.add(shape.clone());
    let mut parts = Vec::new();
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        let current = explorer.current().clone();
        if seen.add(current.clone()) {
            parts.push(current);
        }
        explorer.next()?;
    }
    Ok(parts)
}

/// `exwire wire`: `wire_1`, `wire_2`... for the edges in connection order.
fn exwire(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() <= 1 {
        return Ok(1);
    }
    let shape = session.shape(a[1])?;
    if shape.is_null() || shape.shape_type()? != ShapeEnum::Wire {
        return Ok(0);
    }
    let mut explorer = WireExplorer::with_wire(&to_wire(&shape)?)?;
    let mut index = 0;
    while explorer.more() {
        index += 1;
        session.set_shape(&sub_name(a[1], index), explorer.current())?;
        explorer.next()?;
    }
    Ok(0)
}

/// `invert name`: the shape rebuilt with reversed children (closed flag kept).
fn invert(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() <= 1 {
        return Ok(1);
    }
    let shape = session.shape(a[1])?;
    if shape.is_null() {
        return Ok(0);
    }
    let mut inverted = shape.empty_copied()?;
    let closed = tshape_of(&shape).is_some_and(|t| t.closed());
    if let Some(tshape) = tshape_of(&inverted) {
        tshape.set_closed(closed);
    }
    let builder = BRepBuilder::default();
    for child in children(&shape)? {
        builder.add(&mut inverted, &child.reversed())?;
    }
    session.set_shape(a[1], &inverted)?;
    Ok(0)
}

/// The TShape of `shape`, `None` for a null shape.
fn tshape_of(shape: &Shape) -> Option<&dyn TShape> {
    shape.tshape().get()
}

/// `orientation names... F|R|E|I`, `treverse names...`, `complement names...`.
fn orientation(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let n = a.len();
    if n <= 1 {
        return Ok(1);
    }
    let mut last = n;
    let mut target = Orientation::Forward;
    let command = a[0].to_ascii_lowercase();
    if command == "orientation" {
        if n <= 2 {
            return Ok(1);
        }
        last -= 1;
        target = match a[n - 1].as_bytes().first() {
            Some(b'R') => Orientation::Reversed,
            Some(b'I') => Orientation::Internal,
            Some(b'E') => Orientation::External,
            _ => Orientation::Forward,
        };
    }
    for name in &a[1..last] {
        let mut shape = session.shape(name)?;
        if shape.is_null() {
            continue;
        }
        match command.as_str() {
            "complement" => shape.complement(),
            "treverse" => shape.reverse(),
            _ => shape.set_orientation(target),
        }
        session.set_shape(name, &shape)?;
    }
    Ok(0)
}

/// A TShape flag setter.
type FlagSetter = fn(&dyn TShape, bool);

/// `setProp`: the flags named in `a[2..]` (`-flag` clears) on the TShape of `shape`.
fn set_properties(shape: &Shape, a: &[&str]) {
    let Some(tshape) = tshape_of(shape) else {
        return;
    };
    let setters: [(&str, FlagSetter); 8] = [
        ("free", |t, v| t.set_free(v)),
        ("modified", |t, v| t.set_modified(v)),
        ("checked", |t, v| t.set_checked(v)),
        ("orientable", |t, v| t.set_orientable(v)),
        ("closed", |t, v| t.set_closed(v)),
        ("infinite", |t, v| t.set_infinite(v)),
        ("convex", |t, v| t.set_convex(v)),
        ("locked", |t, v| t.set_locked(v)),
    ];
    for argument in &a[2..] {
        for (flag, set) in &setters {
            if argument.contains(flag) {
                set(tshape, !argument.starts_with('-'));
            }
        }
    }
}

/// `setflags shape flags...`: the flags on the shape, then on its vertices, edges and
/// faces.
fn set_flags(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() < 3 {
        return Ok(1);
    }
    let shape = session.shape(a[1])?;
    if shape.is_null() {
        return Ok(1);
    }
    set_properties(&shape, a);
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        let mut explorer = Explorer::new(&shape, kind, ShapeEnum::Shape)?;
        while explorer.more() {
            set_properties(explorer.current(), a);
            explorer.next()?;
        }
    }
    Ok(0)
}

/// `purgemmgt`: `Standard::Purge()`.
fn purgemmgt(_session: &DrawSession, _di: &DrawInterpretor, _a: &[&str]) -> Result<i32, Failure> {
    rustycad_tkernel::standard::memory::purge();
    Ok(0)
}

/// `check shapes...`: each face marked not checked, then `BRepTools::Update`.
fn check(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() < 2 {
        return Ok(1);
    }
    for name in &a[1..] {
        let shape = session.shape(name)?;
        if shape.is_null() {
            continue;
        }
        let mut explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)?;
        while explorer.more() {
            let face = explorer.current().clone();
            if let Some(tshape) = tshape_of(&face) {
                tshape.set_checked(false);
            }
            brep_tools::update(&face)?;
            explorer.next()?;
        }
    }
    Ok(0)
}

/// `removeinternals shape [force]`: the INTERNAL sub-shapes removed.
fn remove_internals(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        di.print_help(a[0])?;
        return Ok(1);
    }
    let mut shape = session.shape(a[1])?;
    if shape.is_null() {
        di.append(a[1])?.append("is a null shape\n")?;
        return Ok(1);
    }
    let force = a.len() > 2 && session.atoi(a[2]) != 0;
    brep_tools::remove_internals(&mut shape, force)?;
    session.set_shape(a[1], &shape)?;
    Ok(0)
}
