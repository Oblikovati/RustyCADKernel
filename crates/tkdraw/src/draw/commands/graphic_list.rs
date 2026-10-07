//! The command list of `Draw::GraphicCommands`.
use super::{graphic, graphic_view, postscript, register};
use crate::draw::session::{DrawSession, SessionCommand};
use rustycad_tkernel::standard::Failure;

/// `Draw::GraphicCommands`: the `DRAW Graphic Commands` group.
pub fn graphic_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("Draw::GraphicCommands") {
        return Ok(());
    }
    let (file, mut commands) = (file!(), view_commands());
    commands.extend(view_parameter_commands());
    register(session, "DRAW Graphic Commands", file, &commands)
}

/// The view management part of the graphic commands.
fn view_commands() -> Vec<(&'static str, &'static str, SessionCommand)> {
    vec![
        ("wclick", "wait for a mouse click", graphic::wclick),
        (
            "zoom",
            "zoom [view-id] z, or zoom z for all 3d views",
            graphic::zoom,
        ),
        (
            "2dzoom",
            "2dzoom [view-id] z, or zoom2d z for all 2d views",
            graphic::zoom,
        ),
        (
            "wzoom",
            "wzoom [view-id X1 Y1 X2 Y2]\n- fits the contents of a given rectangle into a view window.\n- The view window and rectangle corners are specified through the arguments\n- or selected interactively by the user if no arguments are given",
            graphic::wzoom,
        ),
        (
            "view",
            "view view-id type X(0) Y(0) W(500) H(500)",
            graphic::view,
        ),
        ("delete", "delete [view-id]", graphic::delview),
        ("fit", "fit [view-id]", graphic::fit),
        ("2dfit", "2dfit [view-id]", graphic::fit),
    ]
}

/// The view parameter and output part of the graphic commands.
fn view_parameter_commands() -> Vec<(&'static str, &'static str, SessionCommand)> {
    vec![
        ("fu", "fu [view-id], focal up", graphic_view::focal),
        ("fd", "fd [view-id], focal down", graphic_view::focal),
        ("focal", "focal [f]", graphic_view::setfocal),
        ("mu", "mu [view-id], magnify up", graphic_view::magnify),
        ("2dmu", "2dmu [view-id], magnify up", graphic_view::magnify),
        ("md", "md [view-id], magnify down", graphic_view::magnify),
        (
            "2dmd",
            "2dmd [view-id], magnify down",
            graphic_view::magnify,
        ),
        ("u", "u [view-id], rotate up", graphic_view::rotate),
        ("d", "d [view-id], rotate down", graphic_view::rotate),
        ("l", "l [view-id], rotate left", graphic_view::rotate),
        ("r", "r [view-id], rotate right", graphic_view::rotate),
        ("pu", "pu [view-id], panning up", graphic_view::panning),
        ("pd", "pd [view-id], panning down", graphic_view::panning),
        ("pl", "pl [view-id], panning left", graphic_view::panning),
        ("pr", "pr [view-id], panning right", graphic_view::panning),
        ("2dpu", "2dpu [view-id], panning up", graphic_view::panning),
        (
            "2dpd",
            "2dpd [view-id], panning down",
            graphic_view::panning,
        ),
        (
            "2dpl",
            "2dpl [view-id], panning left",
            graphic_view::panning,
        ),
        (
            "2dpr",
            "2dpr [view-id], panning right",
            graphic_view::panning,
        ),
        ("ptv", "ptv [view-id], X , Y , Z", graphic_view::ptv),
        ("dptv", "dptv [view-id], dX , dY , dZ", graphic_view::dptv),
        (
            "color",
            "color i colorname, define color i",
            graphic_view::color,
        ),
        (
            "hardcopy",
            "hardcopy [file = a4.ps] [view-id = 1] [format = a4]",
            postscript::hardcopy,
        ),
        (
            "xwd",
            "xwd [id = 1] <filename>.{png|bmp|jpg|gif}\n\t\t: Dump contents of viewer window to PNG, BMP, JPEG or GIF file",
            postscript::xwd,
        ),
        (
            "hcolor",
            "hcolor icol width gray (< 1, 0 black)",
            postscript::hcolor,
        ),
        (
            "grid",
            "grid [stepX(100) [stepY [stepZ]]] / 0",
            graphic_view::grid,
        ),
        ("dflush", "dflush, flush the viewer", graphic_view::dflush),
        ("dtext", "dtext [x y [z]] string", graphic_view::dtext),
        (
            "dfont",
            "dfont [name size] : set name and size of Draw font, or reset to default",
            postscript::dfont,
        ),
    ]
}
