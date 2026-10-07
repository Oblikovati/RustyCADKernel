//! The display commands of `Draw_VariableCommands.cxx`: `display`, `erase`, `donly`,
//! `clear`, `2dclear`, `draw`, `repaint`.
use crate::draw::{DrawInterpretor, DrawSession, DrawableHandle};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::standard::Failure;
use std::cell::Cell;
use std::fmt::Write;

thread_local! {
    /// The `static bool draw_erase_mute` of `erase`: `erase -mute` silences the next erase.
    static ERASE_MUTE: Cell<bool> = const { Cell::new(false) };
}

/// `display name ...`: shows the named drawables that are not visible and lists them.
pub(super) fn display(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() <= 1 {
        return Ok(1);
    }
    for name in &a[1..] {
        if let Some(drawable) = session.get(name)
            && !drawable.visible()
        {
            session.viewer().display(&drawable);
            di.append(*name)?.append(" ")?;
        }
    }
    Ok(0)
}

/// `erase [names]`, `donly names`, `clear`, `2dclear`: without names (or for `donly`)
/// clears the display, keeping the protected visible drawables; with names erases those.
pub(super) fn erase(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() == 2 && a[1].eq_ignore_ascii_case("-mute") {
        ERASE_MUTE.with(|mute| mute.set(true));
        return Ok(0);
    }
    let display_only = a[0].eq_ignore_ascii_case("donly");
    if a.len() <= 1 || display_only {
        if !clear_display(session, di, a, display_only)? {
            return Ok(0);
        }
    } else {
        erase_named(session, di, &a[1..])?;
    }
    ERASE_MUTE.with(|mute| mute.set(false));
    session.repaint_2d.set(false);
    session.repaint_3d.set(false);
    session.viewer().flush();
    Ok(0)
}

/// The clearing branch of `erase`; false when `donly` stopped at an unknown name.
fn clear_display(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
    display_only: bool,
) -> Result<bool, Failure> {
    // `donly` resolves the names (and `.` picks) before erasing.
    if display_only {
        for name in &a[1..] {
            if session.get(name).is_none() {
                if *name == "." {
                    session.console().write_out("Missed !!!\n");
                }
                return Ok(false);
            }
        }
    }
    let kept: Vec<DrawableHandle> = session
        .drawables()
        .into_iter()
        .filter(|d| d.protected() && d.visible())
        .collect();
    match a[0].as_bytes().first() {
        Some(b'2') => session.viewer().clear_2d(),
        Some(b'c') => session.viewer().clear_3d(),
        _ => session.viewer().clear(),
    }
    if display_only {
        show_names(session, di, &a[1..])?;
    }
    for drawable in &kept {
        session.viewer().display(drawable);
    }
    Ok(true)
}

/// Displays the named drawables for `donly`, listing them unless muted.
fn show_names(session: &DrawSession, di: &DrawInterpretor, names: &[&str]) -> Result<(), Failure> {
    for name in names {
        let Some(drawable) = session.get(name) else {
            continue;
        };
        if drawable.visible() {
            continue;
        }
        session.viewer().display(&drawable);
        if !ERASE_MUTE.with(Cell::get) {
            di.append(*name)?.append(" ")?;
        }
    }
    Ok(())
}

/// Erases the named visible drawables, listing their names unless muted.
fn erase_named(session: &DrawSession, di: &DrawInterpretor, names: &[&str]) -> Result<(), Failure> {
    for name in names {
        let Some(drawable) = session.get(name) else {
            continue;
        };
        if !drawable.visible() {
            continue;
        }
        session.viewer().remove_drawable(&drawable);
        if !ERASE_MUTE.with(Cell::get) {
            di.append(drawable.name().unwrap_or_default())?
                .append(" ")?;
        }
    }
    session.viewer().repaint_2d();
    session.viewer().repaint_3d();
    Ok(())
}

/// `draw view mode names`: draws the named drawables on one view with a drawing mode.
pub(super) fn draw(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        return Ok(1);
    }
    let view = session.atoi(a[1]);
    if !session.viewer().has_view(view) {
        let _ = write!(Message::send_fail(), "bad view number in draw");
        return Ok(1);
    }
    let mode = session.atoi(a[2]);
    // The names are resolved before the display is made (C++ resolves each inside the
    // loop); only a `.` pick could tell the difference, by picking before drawing.
    let drawables: Vec<DrawableHandle> =
        a[3..].iter().filter_map(|name| session.get(name)).collect();
    let mut viewer = session.viewer();
    let mut display = viewer.make_display(view);
    display.set_mode(mode);
    for drawable in &drawables {
        drawable.draw_on(&mut display);
    }
    display.flush();
    Ok(0)
}

/// `repaint`: repaints the views marked for repaint.
pub(super) fn repaint(
    session: &DrawSession,
    _di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    if session.repaint_2d.replace(false) {
        session.viewer().repaint_2d();
    }
    if session.repaint_3d.replace(false) {
        session.viewer().repaint_3d();
    }
    session.viewer().flush();
    Ok(0)
}
