//! The view-parameter and output commands of `Draw_GraphicCommands.cxx`: focal,
//! magnify, rotate, panning, `ptv`, `dptv`, `color`, `hardcopy`, `dfont`, `hcolor`,
//! `xwd`, `grid`, `dflush`, `dtext`.
use super::graphic::{set_title, view_id, view_trsf, views_of_kind};
use crate::draw::{
    DrawColor, DrawColorKind, DrawGrid, DrawInterpretor, DrawOStream, DrawSession, DrawText2D,
    DrawText3D, DrawableHandle, MAXVIEW, downcast_handle,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Transform3, Vector3};
use std::f64::consts::{PI, SQRT_2};
use std::rc::Rc;

/// `DEFROTATE`: the rotation step, 5 degrees.
const STEP_ROTATE: f64 = 5.0 * PI / 180.0;
/// `DEFMAGNIFY`.
const STEP_MAGNIFY: f64 = 1.1;
/// `DEFPANNING`: a fraction of the window size.
const STEP_PAN: f64 = 0.1;
/// `DEFFOCAL`.
const STEP_FOCAL: f64 = 1.1;
/// `DEFGRIDSTEP`.
const DEFAULT_GRID_STEP: f64 = 100.0;

/// The views a command applies to: the one named by `a[1]`, or all slots.
fn target_views(session: &DrawSession, a: &[&str]) -> Option<Vec<i32>> {
    match a.get(1) {
        Some(text) => view_id(session, text).map(|id| vec![id]),
        None => Some((0..MAXVIEW as i32).collect()),
    }
}

/// `fu` / `fd [id]`: focal distance up or down in perspective views.
pub(super) fn focal(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let Some(views) = target_views(session, a) else {
        return Ok(1);
    };
    let factor = match a[0].to_ascii_lowercase().as_str() {
        "fu" => STEP_FOCAL,
        "fd" => 1.0 / STEP_FOCAL,
        _ => 1.0,
    };
    for id in views {
        if session.viewer().view_type(id).eq_ignore_ascii_case("PERS") {
            let distance = session.viewer().focal(id) * factor;
            session.viewer().set_focal(id, distance);
            session.viewer().repaint_view(id);
        }
    }
    Ok(0)
}

/// `focal [f]`: prints or sets the focal distance of every perspective view.
pub(super) fn setfocal(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let perspective: Vec<i32> = (0..MAXVIEW as i32)
        .filter(|id| session.viewer().view_type(*id).eq_ignore_ascii_case("PERS"))
        .collect();
    let Some(text) = a.get(1) else {
        for id in perspective {
            di.append(format!("Focal view {id} is "))?
                .append(session.viewer().focal(id))?
                .append("\n")?;
        }
        return Ok(0);
    };
    let distance = session.atof(text);
    perspective
        .iter()
        .for_each(|id| session.viewer().set_focal(*id, distance));
    session.viewer().repaint_all();
    Ok(0)
}

/// `mu`/`md`/`2dmu`/`2dmd [id]`: magnify up or down.
pub(super) fn magnify(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let Some(views) = target_views(session, a) else {
        return Ok(1);
    };
    let two_d = a[0].starts_with('2');
    let command = if two_d { &a[0][2..] } else { a[0] };
    let factor = if command.eq_ignore_ascii_case("mu") {
        STEP_MAGNIFY
    } else {
        1.0 / STEP_MAGNIFY
    };
    let kind = views_of_kind(session, two_d);
    for id in views.into_iter().filter(|id| kind.contains(id)) {
        let zoom = session.viewer().zoom(id) * factor;
        session.viewer().set_zoom(id, zoom);
        set_title(session, id);
        session.viewer().repaint_view(id);
    }
    Ok(0)
}

/// `u`/`d`/`l`/`r [id]`: rotates axonometric and perspective views by 5 degrees.
pub(super) fn rotate(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let Some(views) = target_views(session, a) else {
        return Ok(1);
    };
    let (direction, angle) = match a[0].to_ascii_lowercase().as_str() {
        "u" => ((1.0, 0.0), -STEP_ROTATE),
        "d" => ((1.0, 0.0), STEP_ROTATE),
        "l" => ((0.0, 1.0), -STEP_ROTATE),
        "r" => ((0.0, 1.0), STEP_ROTATE),
        // A default gp_Dir2d is X, with no rotation.
        _ => ((1.0, 0.0), 0.0),
    };
    let Ok(direction) = Direction2::new(direction.0, direction.1) else {
        return Ok(0);
    };
    for id in views {
        let view_type = session.viewer().view_type(id);
        if view_type.eq_ignore_ascii_case("AXON") || view_type.eq_ignore_ascii_case("PERS") {
            session.viewer().rotate_view_2d(id, direction, angle);
            session.viewer().repaint_view(id);
        }
    }
    Ok(0)
}

/// `pu`/`pd`/`pl`/`pr` (and `2d` variants) `[id]`: pans by a tenth of the window.
pub(super) fn panning(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let Some(views) = target_views(session, a) else {
        return Ok(1);
    };
    let two_d = a[0].starts_with('2');
    let command = if two_d { &a[0][2..] } else { a[0] };
    let (dx, dy) = match command.to_ascii_lowercase().as_str() {
        "pu" => (0, 1),
        "pd" => (0, -1),
        "pl" => (-1, 0),
        "pr" => (1, 0),
        _ => (0, 0),
    };
    let kind = views_of_kind(session, two_d);
    for id in views.into_iter().filter(|id| kind.contains(id)) {
        let Some((_, _, w, h)) = session.viewer().pos_size(id) else {
            continue;
        };
        let shift = |size: i32, step: i32| (size as f64 * step as f64 * STEP_PAN) as i32;
        session.viewer().pan_view(id, shift(w, dx), shift(h, dy));
        session.viewer().repaint_view(id);
    }
    Ok(0)
}

/// The view and the three values of `ptv`/`dptv` (`[id] X Y Z`).
fn view_and_point(session: &DrawSession, a: &[&str]) -> Option<(Vec<i32>, Vector3)> {
    let (views, values) = if a.len() >= 5 {
        (vec![view_id(session, a[1])?], &a[2..5])
    } else {
        ((0..MAXVIEW as i32).collect(), &a[1..4])
    };
    let [x, y, z] = [values[0], values[1], values[2]].map(|text| session.atof(text));
    Some((views, Vector3::new(x, y, z)))
}

/// A translation by `-vector`.
fn back_translation(vector: Vector3) -> Transform3 {
    let mut translation = Transform3::default();
    translation.set_translation_part(Vector3::new(-vector.x(), -vector.y(), -vector.z()));
    translation
}

/// `ptv [id] X Y Z`: the view looks at the point (the translation of the view matrix is
/// replaced).
pub(super) fn ptv(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    translate_views(session, a, true)
}

/// `dptv [id] dX dY dZ`: shifts what the view looks at.
pub(super) fn dptv(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    translate_views(session, a, false)
}

/// The common part of `ptv` (`reset_translation`) and `dptv`.
fn translate_views(
    session: &DrawSession,
    a: &[&str],
    reset_translation: bool,
) -> Result<i32, Failure> {
    if a.len() < 4 {
        return Ok(1);
    }
    let Some((views, vector)) = view_and_point(session, a) else {
        return Ok(1);
    };
    for id in views {
        let mut matrix = view_trsf(session, id);
        if reset_translation {
            matrix.set_translation_part(Vector3::new(0.0, 0.0, 0.0));
        }
        session
            .viewer()
            .set_trsf(id, matrix * back_translation(vector));
        session.viewer().repaint_view(id);
    }
    Ok(0)
}

/// `color i name` defines palette entry `i`; without arguments toggles the black
/// background.
pub(super) fn color(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        session
            .black_background
            .set(!session.black_background.get());
        return Ok(0);
    }
    let index = usize::try_from(session.atoi(a[1])).unwrap_or(usize::MAX);
    if !session.viewer().define_color(index, a[2]) {
        di.append(format!("Could not allocate color {}\n", a[2]))?;
        return Ok(1);
    }
    Ok(0)
}

/// The paper format of `hardcopy`: margin, width and height in millimetres, scaled from
/// A4 (210 x 210*sqrt(2), 3 mm margin) with the C++ operations of each format.
fn paper_format(name: &str) -> (f64, f64, f64) {
    let scale = |value: f64| match name {
        "a7" => value / (2.0 * SQRT_2),
        "a6" => value / 2.0,
        "a5" => value / SQRT_2,
        "a3" => value * SQRT_2,
        "a2" => value * 2.0,
        "a1" => value * 2.0 * SQRT_2,
        "a0" => value * 4.0,
        _ => value,
    };
    (scale(3.0), scale(210.0), scale(210.0 * 2f64.sqrt()))
}

/// `hardcopy [file=a4.ps] [id=1] [format=a4]`: an EPS file of the view (the file is
/// created even when the view does not exist, as upstream).
pub(super) fn hardcopy(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let file = a.get(1).copied().unwrap_or("a4.ps");
    let id = match a.get(2) {
        Some(text) => match view_id(session, text) {
            Some(id) => id,
            None => return Ok(1),
        },
        None => 1,
    };
    let (margin, width, height) = paper_format(a.get(3).copied().unwrap_or("a4"));
    const PIXELS_PER_MM: f64 = 28.4;
    let paper = [
        margin * PIXELS_PER_MM,
        margin * PIXELS_PER_MM,
        (width - margin) * PIXELS_PER_MM,
        (height - margin) * PIXELS_PER_MM,
    ]
    .map(|v| v as i32);
    let mut output = DrawOStream::new();
    let frame = session
        .viewer()
        .frame(id)
        .filter(|_| session.viewer().has_view(id));
    if let Some(frame) = frame {
        write_postscript(session, id, frame, paper, &mut output);
    }
    // `std::ofstream` failures go unreported upstream.
    let _ = std::fs::write(file, output.text());
    Ok(0)
}

/// The EPS header, frame, view drawing and trailer of `hardcopy`.
fn write_postscript(
    session: &DrawSession,
    id: i32,
    (vx0, vy0, vx1, vy1): (i32, i32, i32, i32),
    paper: [i32; 4],
    out: &mut DrawOStream,
) {
    let [px0, py0, mut px1, mut py1] = paper;
    let kx = (px1 - px0) as f64 / (vx1 - vx0) as f64;
    let ky = (py1 - py0) as f64 / (vy1 - vy0) as f64;
    let k = kx.abs().min(ky.abs());
    let (kx, ky) = (if kx > 0.0 { k } else { -k }, if ky > 0.0 { k } else { -k });
    px1 = (px0 as f64 + kx * (vx1 - vx0) as f64) as i32;
    py1 = (py0 as f64 + ky * (vy1 - vy0) as f64) as i32;
    let (font, size) = session.font.borrow().clone();
    out.put_str(&format!(
        "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: {px0} {py0} {px1} {py1}\n%%Pages: 1\n"
    ));
    out.put_str(&format!(
        "%%DocumentFonts: {font}\n%%EndComments\n/{font} findfont\n{size} scalefont\nsetfont\n"
    ));
    out.put_str("/m {moveto} bind def\n/l {lineto} bind def\n.1 .1 scale\n3 setlinewidth\n0 setgray\nnewpath\n");
    let rectangle = format!("{px0} {py0} m\n{px1} {py0} l\n{px1} {py1} l\n{px0} {py1} l\n");
    out.put_str(&rectangle)
        .put_str("closepath\nstroke\nnewpath\n")
        .put_str(&rectangle)
        .put_str("closepath\nclip\n");
    session
        .viewer()
        .postscript_view(id, [vx0, vy0, vx1, vy1], [px0, py0, px1, py1], out);
    out.put_str("showpage\n%%EOF\n");
}

/// `dfont [name [size]]`: sets the PostScript font, or resets it without arguments.
pub(super) fn dfont(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    {
        let mut font = session.font.borrow_mut();
        match a.len() {
            1 => *font = ("Helvetica".to_string(), "150".to_string()),
            2 => font.0 = a[1].to_string(),
            3 => *font = (a[1].to_string(), a[2].to_string()),
            _ => {}
        }
    }
    let (name, size) = session.font.borrow().clone();
    di.append(format!("{name} {size}\n"))?;
    Ok(0)
}

/// `hcolor icol width gray`: the PostScript pen of a palette entry, or the palette.
pub(super) fn hcolor(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 4 {
        di.append(
            "code de couleur (Draw.cxx) : \n0 = White,\t 1 = Red,\t 2 = Green,\t 3 = Blue\n",
        )?;
        di.append("4 = Cyan,\t 5 = Gold,\t 6 = Magenta,\t 7 = Maroon\n8 = Orange,\t 9 = Pink,\t 10 = Salmon,\t 11 = Violet\n")?;
        di.append("12 = Yellow,\t 13 = Khaki,\t 14 = Coral\n1 <= width <= 11,  0 (noir)  <= gray <= 1 (blanc)\n")?;
        return Ok(0);
    }
    let (index, width, gray) = (session.atoi(a[1]), session.atoi(a[2]), session.atof(a[3]));
    session.viewer().post_color(index, width, gray);
    Ok(0)
}

/// `xwd [id=1] file`: saves the view image.
pub(super) fn xwd(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        return Ok(1);
    }
    session.repaint_now_if_necessary();
    let (id, file) = if a.len() > 2 {
        (session.atoi(a[1]), a[2])
    } else {
        (1, a[1])
    };
    match session.viewer().save_view(id, file) {
        Ok(true) => Ok(0),
        Ok(false) => Ok(1),
        Err(message) => {
            session.console().write_err(&message);
            Ok(1)
        }
    }
}

/// `grid [stepX [stepY [stepZ]]]`: the steps of the `grid` variable (default 100).
pub(super) fn grid(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let value = |index: usize| session.atof(a[index]).abs();
    let (x, y, z) = match a.len() {
        1 => (DEFAULT_GRID_STEP, DEFAULT_GRID_STEP, DEFAULT_GRID_STEP),
        2 => (value(1), value(1), value(1)),
        3 => (value(1), value(2), value(2)),
        4 => (value(1), value(2), value(3)),
        _ => return Ok(1),
    };
    let grid = session
        .get("grid")
        .and_then(|drawable| downcast_handle::<DrawGrid>(&drawable));
    let grid = grid.ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "grid: the variable 'grid' holds no Draw_Grid",
        )
    })?;
    grid.set_steps(x, y, z);
    session.viewer().repaint_all();
    Ok(0)
}

/// `dflush`: flushes the viewer.
pub(super) fn dflush(
    session: &DrawSession,
    _di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    session.viewer().flush();
    Ok(0)
}

/// `dtext [x y [z]] text`: displays a green text at a point, or at a picked position.
pub(super) fn dtext(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let (point, is_3d) = match a.len() {
        2 => match picked_text_position(session, di)? {
            Some(position) => position,
            None => return Ok(0),
        },
        n if n >= 4 => {
            let is_3d = n > 4;
            let z = if is_3d { session.atof(a[3]) } else { 0.0 };
            (
                Point3::new(session.atof(a[1]), session.atof(a[2]), z),
                is_3d,
            )
        }
        _ => return Ok(0),
    };
    let green = DrawColor::new(DrawColorKind::Vert);
    let text = a[a.len() - 1];
    let drawable: DrawableHandle = if is_3d {
        Rc::new(DrawText3D::new(point, text, green, 0.0, 0.0))
    } else {
        Rc::new(DrawText2D::new(
            Point2::new(point.x(), point.y()),
            text,
            green,
            0,
            0,
        ))
    };
    session.viewer().display(&drawable);
    Ok(0)
}

/// The picked position of `dtext name`, in model coordinates, with the view kind.
fn picked_text_position(
    session: &DrawSession,
    di: &DrawInterpretor,
) -> Result<Option<(Point3, bool)>, Failure> {
    di.append("Pick position with button 1, other button escape\n")?;
    let mut errors = String::new();
    let selection = session
        .viewer()
        .select(true, &mut errors)
        .unwrap_or_default();
    session.console().write_err(&errors);
    if selection.button != 1 {
        return Ok(None);
    }
    let zoom = session.viewer().zoom(selection.view);
    let mut inverse = view_trsf(session, selection.view);
    // A view matrix is a rigid motion and always invertible.
    let _ = inverse.invert();
    let point =
        Point3::new(selection.x as f64 / zoom, selection.y as f64 / zoom, 0.0).transformed(inverse);
    Ok(Some((point, session.viewer().is_3d(selection.view))))
}
