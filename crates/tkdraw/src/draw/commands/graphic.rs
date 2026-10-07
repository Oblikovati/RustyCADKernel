//! The view commands of `Draw_GraphicCommands.cxx`: `view`, `delete`, `fit`, `zoom`,
//! `wzoom`, `wclick`.
use crate::draw::{DrawColor, DrawColorKind, DrawInterpretor, DrawSession, MAXVIEW};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Transform3};
use std::fmt::Write;

/// A view and two opposite corners in window pixels: `(view, x1, y1, x2, y2)`.
type ViewRectangle = (i32, i32, i32, i32, i32);

/// `DEFFRAME`: the margin of `fit`, in pixels.
pub(super) const FRAME: f64 = 10.0;

/// `ViewId`: the view number of `text`, or `None` after printing why it is not usable.
pub(super) fn view_id(session: &DrawSession, text: &str) -> Option<i32> {
    let id = session.atoi(text);
    if !(0..MAXVIEW as i32).contains(&id) {
        session.console().write_out(&format!(
            "Incorrect view-id, must be in 0..{}\n",
            MAXVIEW - 1
        ));
        return None;
    }
    if !session.viewer().has_view(id) {
        session
            .console()
            .write_out(&format!("View {id} does not exist.\n"));
        return None;
    }
    Some(id)
}

/// `SetTitle`: `id : type - Zoom z` (`%f`).
pub(super) fn set_title(session: &DrawSession, id: i32) {
    let mut viewer = session.viewer();
    if viewer.has_view(id) {
        let title = format!(
            "{id} : {} - Zoom {:.6}",
            viewer.view_type(id),
            viewer.zoom(id)
        );
        viewer.set_title(id, &title);
    }
}

/// The views of one kind: 2D views when `two_d`, 3D views otherwise.
pub(super) fn views_of_kind(session: &DrawSession, two_d: bool) -> Vec<i32> {
    let viewer = session.viewer();
    (0..MAXVIEW as i32)
        .filter(|id| viewer.has_view(*id) && viewer.is_3d(*id) != two_d)
        .collect()
}

/// `zoom z` (all 3D views), `2dzoom z` (all 2D views), `zoom id z`.
pub(super) fn zoom(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() == 2 {
        let z = session.atof(a[1]);
        for id in views_of_kind(session, a[0].eq_ignore_ascii_case("2dzoom")) {
            session.viewer().set_zoom(id, z);
            set_title(session, id);
            session.viewer().repaint_view(id);
        }
        return Ok(0);
    }
    if a.len() < 3 {
        return Ok(1);
    }
    let Some(id) = view_id(session, a[1]) else {
        return Ok(1);
    };
    let z = session.atof(a[2]);
    session.viewer().set_zoom(id, z);
    session.viewer().repaint_view(id);
    set_title(session, id);
    Ok(0)
}

/// `view id type [X Y W H]` creates a window view (500x500 by default, or the size of
/// the view it replaces); `view id type window` attaches to a native window.
pub(super) fn view(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if session.is_batch() || a.len() < 3 {
        return Ok(1);
    }
    let id = session.atoi(a[1]);
    if !(0..MAXVIEW as i32).contains(&id) {
        di.append(format!("View-id must be in 0..{}\n", MAXVIEW - 1))?;
        return Ok(1);
    }
    if a.len() == 4 {
        session.viewer().make_view_on(id, a[2], a[3]);
    } else {
        let previous = session.viewer().pos_size(id).unwrap_or((0, 0, 500, 500));
        let field =
            |index: usize, default: i32| a.get(index).map_or(default, |text| session.atoi(text));
        let (x, y, w, h) = (
            field(3, previous.0),
            field(4, previous.1),
            field(5, previous.2),
            field(6, previous.3),
        );
        session.viewer().make_view(id, a[2], x, y, w, h);
    }
    if !session.viewer().has_view(id) {
        di.append("View creation failed\n")?;
        return Ok(1);
    }
    set_title(session, id);
    session.viewer().display_view(id);
    Ok(0)
}

/// `delete [id]`: deletes one view or all.
pub(super) fn delview(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() == 1 {
        (0..MAXVIEW as i32).for_each(|id| session.viewer().delete_view(id));
        return Ok(0);
    }
    let Some(id) = view_id(session, a[1]) else {
        return Ok(1);
    };
    session.viewer().delete_view(id);
    Ok(0)
}

/// `fit [id]` / `2dfit [id]`: fits one view, or all views of the kind with the smallest
/// of their fitted zooms.
pub(super) fn fit(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() >= 2 {
        let Some(id) = view_id(session, a[1]) else {
            return Ok(1);
        };
        session.viewer().fit_view(id, FRAME as i32);
        session.viewer().repaint_view(id);
        set_title(session, id);
        return Ok(0);
    }
    let views = views_of_kind(session, a[0].eq_ignore_ascii_case("2dfit"));
    let mut zoom = f64::MAX;
    for id in &views {
        session.viewer().fit_view(*id, FRAME as i32);
        zoom = zoom.min(session.viewer().zoom(*id));
    }
    for id in views {
        session.viewer().set_zoom(id, zoom);
        session.viewer().repaint_view(id);
        set_title(session, id);
    }
    Ok(0)
}

/// `wclick`: waits for a click.
pub(super) fn wclick(
    session: &DrawSession,
    di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    session.viewer().flush();
    di.append("Just click.\n")?;
    let mut errors = String::new();
    let _ = session.viewer().select(true, &mut errors);
    session.console().write_err(&errors);
    Ok(0)
}

/// `wzoom [id X1 Y1 X2 Y2]`: zooms a view onto a rectangle given in pixels, or picked
/// with a rubber band (xor mode) when there are no arguments.
pub(super) fn wzoom(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 1 && a.len() != 6 {
        di.append(format!("Usage : {} [view-id X1 Y1 X2 Y2]\n", a[0]))?;
        return Ok(1);
    }
    let rectangle = if a.len() == 1 {
        pick_rectangle(session, di)?
    } else {
        given_rectangle(session, a)?
    };
    let Some((id, x1, y1, x2, y2)) = rectangle else {
        return Ok(if a.len() == 1 { 0 } else { 1 });
    };
    zoom_on_rectangle(session, id, (x1, y1, x2, y2));
    Ok(0)
}

/// The arguments of `wzoom id X1 Y1 X2 Y2` (`atoi`, not `Draw::Atoi`).
fn given_rectangle(session: &DrawSession, a: &[&str]) -> Result<Option<ViewRectangle>, Failure> {
    let id = c_atoi(a[1]);
    if !(0..MAXVIEW as i32).contains(&id) {
        let _ = write!(
            Message::send_fail(),
            "Incorrect view-id, must be in 0..{}",
            MAXVIEW - 1
        );
        return Ok(None);
    }
    if !session.viewer().has_view(id) {
        let _ = write!(Message::send_fail(), "View {id} does not exist");
        return Ok(None);
    }
    Ok(Some((
        id,
        c_atoi(a[2]),
        c_atoi(a[3]),
        c_atoi(a[4]),
        c_atoi(a[5]),
    )))
}

/// C `atoi`: optional blanks and sign, then decimal digits (0 when there are none).
pub(super) fn c_atoi(text: &str) -> i32 {
    let text = text.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let value = digits
        .bytes()
        .take_while(u8::is_ascii_digit)
        .fold(0i64, |v, d| {
            (v * 10 + i64::from(d - b'0')).min(i64::from(u32::MAX))
        });
    let value = if negative { -value } else { value };
    value as i32
}

/// The zoom of `wzoom` once the rectangle is known.
fn zoom_on_rectangle(
    session: &DrawSession,
    id: i32,
    (mut x1, mut y1, x2, y2): (i32, i32, i32, i32),
) {
    let Some((_, _, w, h)) = session.viewer().pos_size(id) else {
        return;
    };
    if x1 == x2 || y1 == y2 {
        return;
    }
    let mut ratio = (x2 - x1).abs() as f64 / w as f64;
    let ratio_y = (y2 - y1).abs() as f64 / h as f64;
    if ratio_y > ratio {
        ratio = ratio_y;
    }
    ratio = 1.0 / ratio;
    x1 = x1.min(x2);
    y1 = y1.max(y2);
    let (x1, y1) = ((x1 as f64 * ratio) as i32, (y1 as f64 * ratio) as i32);
    let zoom = ratio * session.viewer().zoom(id);
    session.viewer().set_zoom(id, zoom);
    session.viewer().set_pan(id, -x1, -y1);
    session.viewer().repaint_view(id);
    set_title(session, id);
}

/// One `Select` of the rubber band; `(view, x, y, button)`, zeros in batch mode.
fn select(session: &DrawSession, wait: bool) -> (i32, i32, i32, i32) {
    let mut errors = String::new();
    let selection = session
        .viewer()
        .select(wait, &mut errors)
        .unwrap_or_default();
    session.console().write_err(&errors);
    (selection.view, selection.x, selection.y, selection.button)
}

/// A rectangle of the rubber band: two opposite corners in view units.
type Band = ((f64, f64), (f64, f64));

/// Draws the rectangles (4 segments each) on the current display state, then flushes
/// once, as the C++ loop does with its single display.
fn draw_bands(session: &DrawSession, id: i32, bands: &[Band]) {
    let mut viewer = session.viewer();
    let mut display = viewer.resume_display(id);
    for ((x1, y1), (x2, y2)) in bands.iter().copied() {
        let sides = [
            (x1, y1, x1, y2),
            (x1, y2, x2, y2),
            (x2, y2, x2, y1),
            (x2, y1, x1, y1),
        ];
        for (ax, ay, bx, by) in sides {
            display.draw(Point2::new(ax, ay), Point2::new(bx, by));
        }
    }
    display.flush();
}

/// The interactive part of `wzoom`: two corners picked with button 1.
fn pick_rectangle(
    session: &DrawSession,
    di: &DrawInterpretor,
) -> Result<Option<ViewRectangle>, Failure> {
    di.append("Pick first corner\n")?;
    let (id, x1, y1, button) = select(session, true);
    let mut inverse = view_trsf(session, id);
    // A view matrix is a rigid motion; the identity of batch mode is invertible too.
    let _ = inverse.invert();
    let origin = Point3::new(0.0, 0.0, 0.0).transformed(inverse);
    let zoom = session.viewer().zoom(id);
    let to_view = move |x: i32, y: i32| {
        (
            (x as f64 - origin.x()) / zoom,
            (y as f64 - origin.y()) / zoom,
        )
    };
    if button != 1 || id < 0 {
        return Ok(None);
    }
    {
        let mut viewer = session.viewer();
        let mut display = viewer.make_display(id);
        display.set_color(DrawColor::new(DrawColorKind::Blanc));
        display.set_mode(10);
    }
    let (x2, y2, button) = rubber_band(session, di, id, to_view(x1, y1), to_view)?;
    if button != 1 {
        return Ok(None);
    }
    session.viewer().resume_display(id).set_mode(0);
    Ok(Some((id, x1, y1, x2, y2)))
}

/// Follows the pointer until a button is pressed, redrawing the band in xor; returns
/// the last pointer position (pixels) and the button.
fn rubber_band(
    session: &DrawSession,
    di: &DrawInterpretor,
    id: i32,
    first: (f64, f64),
    to_view: impl Fn(i32, i32) -> (f64, f64),
) -> Result<(i32, i32, i32), Failure> {
    let mut previous = first;
    draw_bands(session, id, &[(first, previous)]);
    di.append("Pick second corner\n")?;
    let (mut x2, mut y2, mut button) = (0, 0, 0);
    while button == 0 {
        (_, x2, y2, button) = select(session, false);
        let corner = to_view(x2, y2);
        draw_bands(session, id, &[(first, previous), (first, corner)]);
        previous = corner;
    }
    draw_bands(session, id, &[(first, previous)]);
    Ok((x2, y2, button))
}

/// The view matrix of `id`, identity when absent (C++ leaves its `gp_Trsf` untouched).
pub(super) fn view_trsf(session: &DrawSession, id: i32) -> Transform3 {
    session.viewer().trsf(id).unwrap_or_default()
}
