// Rust translation of DBRep::BasicCommands and the display commands of
// Draw/TKDraw/DBRep/DBRep.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DBRep::BasicCommands`: the `Basic shape commands` group.
mod counts;
mod display;
mod files;
mod normals;
mod topology;

use super::drawable_shape::DBRepDrawableShape;
use crate::draw::{DrawSession, SessionCommand, commands};
use rustycad_tkernel::standard::Failure;

/// The source file recorded for the commands.
const FILE: &str = file!();

/// The group of the shape commands.
const GROUP: &str = "Basic shape commands";

/// One command: name, help, body.
type CommandEntry = (&'static str, &'static str, SessionCommand);

/// `DBRep::BasicCommands`: the `Draw` commands, the `DBRep_DrawableShape` restore
/// factory and the shape commands, once per session.
pub fn basic_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("DBRep::BasicCommands") {
        return Ok(());
    }
    commands(session)?;
    DBRepDrawableShape::register_factory(session);
    register(session, GROUP, &shape_commands())?;
    register(session, "DE: General", &[files::XPROGRESS])?;
    register(session, GROUP, &file_commands())
}

/// The shape commands before `XProgress`, in the C++ registration order.
fn shape_commands() -> Vec<CommandEntry> {
    vec![
        display::ISOS,
        display::HLR,
        display::VORI,
        display::TRIANGLES,
        topology::TCLEAN,
        display::POLYGONS,
        display::VCONN,
        display::DISCRETISATION,
        topology::COMPOUND,
        topology::ADD,
        topology::EXPLODE,
        topology::NEXPLODE,
        topology::EXWIRE,
        topology::EMPTYCOPY,
        topology::CHECK,
        topology::ORIENTATION,
        topology::TREVERSE,
        topology::COMPLEMENT,
        topology::INVERT,
        normals::NORMALS,
        counts::NBSHAPES,
        counts::NUMSHAPES,
        counts::COUNTSHAPES,
        topology::SETFLAGS,
        topology::PURGEMMGT,
    ]
}

/// The file commands after `XProgress`, in the C++ registration order.
fn file_commands() -> Vec<CommandEntry> {
    vec![
        files::WRITEBREP,
        files::READBREP,
        files::BINSAVE,
        files::BINRESTORE,
        topology::REMOVEINTERNALS,
    ]
}

/// `theCommands.Add(name, help, __FILE__, body, group)` for each entry.
fn register(session: &DrawSession, group: &str, entries: &[CommandEntry]) -> Result<(), Failure> {
    for (name, help, body) in entries {
        session.add_command(name, help, FILE, *body, group)?;
    }
    Ok(())
}

/// `body` applied to the drawable shape of variable `name`, if it holds one
/// (`occ::down_cast<DBRep_DrawableShape>(Draw::Get(name))`).
fn with_drawable_shape<R>(
    session: &DrawSession,
    name: &str,
    body: impl FnOnce(&DBRepDrawableShape) -> R,
) -> Option<R> {
    let drawable = session.get(name)?;
    let shape = drawable.downcast_kind::<DBRepDrawableShape>()?;
    Some(body(shape))
}

/// `TopoDS_Shape` name for a variable: `name_<index>`.
fn sub_name(base: &str, index: usize) -> String {
    format!("{base}_{index}")
}
