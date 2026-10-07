// Rust translation of the color and marker commands of
// Draw/TKDraw/DrawTrSurf/DrawTrSurf.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `setcurvcolor`, `changecurvcolor`, `setpointcolor`, `changepointcolor`,
//! `setpointmarker` and `changepointmarker`.
use super::{COLORS_HINT, CommandEntry, MARKERS_HINT};
use crate::draw::{DrawColor, DrawColorKind, DrawInterpretor, DrawMarkerShape, DrawSession};
use crate::draw_tr_surf::curve::DrawTrSurfCurve;
use crate::draw_tr_surf::params::parameters;
use crate::draw_tr_surf::point::DrawTrSurfPoint;
use rustycad_tkernel::standard::Failure;

/// The English color names of the commands, with the palette entry they name.
const COLOR_WORDS: [(&str, DrawColorKind); 15] = [
    ("white", DrawColorKind::Blanc),
    ("red", DrawColorKind::Rouge),
    ("green", DrawColorKind::Vert),
    ("blue", DrawColorKind::Bleu),
    ("cyan", DrawColorKind::Cyan),
    ("golden", DrawColorKind::Or),
    ("magenta", DrawColorKind::Magenta),
    ("brown", DrawColorKind::Marron),
    ("orange", DrawColorKind::Orange),
    ("pink", DrawColorKind::Rose),
    ("salmon", DrawColorKind::Saumon),
    ("violet", DrawColorKind::Violet),
    ("yellow", DrawColorKind::Jaune),
    ("darkgreen", DrawColorKind::Kaki),
    ("coral", DrawColorKind::Corail),
];

/// The marker names of the commands.
const MARKER_WORDS: [(&str, DrawMarkerShape); 6] = [
    ("square", DrawMarkerShape::Square),
    ("diamond", DrawMarkerShape::Losange),
    ("x", DrawMarkerShape::X),
    ("plus", DrawMarkerShape::Plus),
    ("circle", DrawMarkerShape::Circle),
    ("circle_zoom", DrawMarkerShape::CircleZoom),
];

/// The color and marker commands.
pub(super) fn commands() -> Vec<CommandEntry> {
    let set_curve = "setcurvcolor [color] : set curve color by default, or print the current curve color if no argument (this does not modify the color of the curve)\n\n";
    let change_curve = "changecurvcolor color curve: change color of the curve\n\n";
    let set_point = "setpointcolor [color] : set point color by default, or print the current point color if no argument (this does not modify the color of the point)\n\n";
    let change_point = "changepointcolor color point: change color of the point\n\n";
    let set_marker = "setpointmarker [marker] : set point marker by default, or print the current point marker if no argument (this does not modify the marker of the point)\n\n";
    let change_marker = "changepointmarker marker point: change marker of the point\n\n";
    vec![
        (
            "setcurvcolor",
            format!("{set_curve}{COLORS_HINT}"),
            setcurvcolor,
        ),
        (
            "changecurvcolor",
            format!("{change_curve}{COLORS_HINT}"),
            changecurvcolor,
        ),
        (
            "setpointcolor",
            format!("{set_point}{COLORS_HINT}"),
            setpointcolor,
        ),
        (
            "changepointcolor",
            format!("{change_point}{COLORS_HINT}"),
            changepointcolor,
        ),
        (
            "setpointmarker",
            format!("{set_marker}{MARKERS_HINT}"),
            setpointmarker,
        ),
        (
            "changepointmarker",
            format!("{change_marker}{MARKERS_HINT}"),
            changepointmarker,
        ),
    ]
}

/// `printColor`: the English name and a newline (`dark-olive green` for the
/// `darkgreen` entry, as upstream).
fn print_color(di: &DrawInterpretor, color: DrawColor) -> Result<(), Failure> {
    if color.id() == DrawColorKind::Kaki {
        di.append("dark-olive green \n")?;
        return Ok(());
    }
    let name = COLOR_WORDS
        .iter()
        .find(|(_, kind)| *kind == color.id())
        .map_or("", |(name, _)| name);
    di.append(name)?.append(" ")?.append("\n")?;
    Ok(())
}

/// `recognizeColor`: the color named by `word` (any case), else `default`.
fn recognize_color(word: &str, default: DrawColor) -> DrawColor {
    COLOR_WORDS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(word))
        .map_or(default, |(_, kind)| DrawColor::new(*kind))
}

/// `printMarker`.
fn print_marker(di: &DrawInterpretor, marker: DrawMarkerShape) -> Result<(), Failure> {
    if marker == DrawMarkerShape::CircleZoom {
        di.append("circle_zoom \n")?;
        return Ok(());
    }
    let name = MARKER_WORDS
        .iter()
        .find(|(_, shape)| *shape == marker)
        .map_or("", |(name, _)| name);
    di.append(name)?.append(" ")?.append("\n")?;
    Ok(())
}

/// `recognizeMarker`: the marker named by `word` (any case), else `default`.
fn recognize_marker(word: &str, default: DrawMarkerShape) -> DrawMarkerShape {
    MARKER_WORDS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(word))
        .map_or(default, |(_, shape)| *shape)
}

/// `setcurvcolor [color]`: prints the default curve color, or sets it.
fn setcurvcolor(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let params = parameters(session);
    let current = params.borrow().curv_color;
    if a.len() < 2 {
        print_color(di, current)?;
    } else {
        params.borrow_mut().curv_color = recognize_color(a[1], current);
    }
    Ok(0)
}

/// `changecurvcolor color curve`: the color of a curve variable.
fn changecurvcolor(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        return Ok(1);
    }
    let color = recognize_color(a[1], parameters(session).borrow().curv_color);
    if let Some(drawable) = session.get(a[2])
        && let Some(curve) = drawable.downcast_kind::<DrawTrSurfCurve>()
    {
        curve.style().set_color(color);
        session.repaint();
    }
    Ok(0)
}

/// `setpointcolor [color]`: prints the default point color, or sets it.
fn setpointcolor(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let params = parameters(session);
    let current = params.borrow().pnt_color;
    if a.len() < 2 {
        print_color(di, current)?;
    } else {
        params.borrow_mut().pnt_color = recognize_color(a[1], current);
    }
    Ok(0)
}

/// `changepointcolor color point`: the color of a point variable.
fn changepointcolor(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        return Ok(1);
    }
    let color = recognize_color(a[1], parameters(session).borrow().pnt_color);
    if let Some(drawable) = session.get(a[2])
        && let Some(point) = drawable.downcast_kind::<DrawTrSurfPoint>()
    {
        point.set_color(color);
        session.repaint();
    }
    Ok(0)
}

/// `setpointmarker [marker]`: prints the default point marker, or sets it.
fn setpointmarker(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let params = parameters(session);
    let current = params.borrow().pnt_marker;
    if a.len() < 2 {
        print_marker(di, current)?;
    } else {
        params.borrow_mut().pnt_marker = recognize_marker(a[1], current);
    }
    Ok(0)
}

/// `changepointmarker marker point`: the marker of a point variable.
fn changepointmarker(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        return Ok(1);
    }
    let marker = recognize_marker(a[1], parameters(session).borrow().pnt_marker);
    if let Some(drawable) = session.get(a[2])
        && let Some(point) = drawable.downcast_kind::<DrawTrSurfPoint>()
    {
        point.set_shape(marker);
        session.repaint();
    }
    Ok(0)
}
