// Rust translation of DrawTrSurf::BasicCommands and the display commands of
// Draw/TKDraw/DrawTrSurf/DrawTrSurf.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf::BasicCommands`: the geometric display and transformation commands.
mod colors;
mod transform;

use super::bezier_curve::DrawTrSurfBezierCurve;
use super::bezier_curve2d::DrawTrSurfBezierCurve2d;
use super::bezier_surface::DrawTrSurfBezierSurface;
use super::bspline_curve::DrawTrSurfBSplineCurve;
use super::bspline_curve2d::DrawTrSurfBSplineCurve2d;
use super::bspline_surface::DrawTrSurfBSplineSurface;
use super::curve::DrawTrSurfCurve;
use super::curve2d::DrawTrSurfCurve2d;
use super::drawable::DrawTrSurfDrawable;
use super::params::parameters;
use super::point::DrawTrSurfPoint;
use super::polygon::{DrawTrSurfPolygon2D, DrawTrSurfPolygon3D};
use super::spline_common::{KnotsStyle, PolesStyle};
use super::surface::DrawTrSurfSurface;
use super::triangulation::DrawTrSurfTriangulation;
use crate::draw::{DrawInterpretor, DrawSession, Drawable3D, SessionCommand};
use rustycad_tkernel::standard::Failure;

/// The source file recorded for the commands.
const FILE: &str = file!();

/// The colors the color commands accept.
const COLORS_HINT: &str = "The possible colors are: \n  white, red, green, blue, cyan,\n  golden, magenta, brown, orange, pink,\n  salmon, violet, yellow, darkgreen, coral";

/// The markers the marker commands accept.
const MARKERS_HINT: &str =
    "The possible markers are: \n  square, diamond, x, plus, circle, circle_zoom";

/// `DrawTrSurf::BasicCommands`: registers the restore factories of the DrawTrSurf
/// drawables and the `geometric display commands` and `Geometric transformations`
/// groups, once per session.
pub fn basic_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("DrawTrSurf::BasicCommands") {
        return Ok(());
    }
    register_factories(session);
    register(session, "geometric display commands", &display_commands())?;
    register(session, "geometric display commands", &colors::commands())?;
    register(session, "Geometric transformations", &transform::commands())
}

/// The `RegisterFactory` calls, in the C++ order.
fn register_factories(session: &DrawSession) {
    DrawTrSurfBezierCurve::register_factory(session);
    DrawTrSurfBezierCurve2d::register_factory(session);
    DrawTrSurfBezierSurface::register_factory(session);
    DrawTrSurfBSplineCurve::register_factory(session);
    DrawTrSurfBSplineCurve2d::register_factory(session);
    DrawTrSurfBSplineSurface::register_factory(session);
    DrawTrSurfCurve::register_factory(session);
    DrawTrSurfCurve2d::register_factory(session);
    DrawTrSurfPoint::register_factory(session);
    DrawTrSurfPolygon2D::register_factory(session);
    DrawTrSurfPolygon3D::register_factory(session);
    DrawTrSurfSurface::register_factory(session);
    DrawTrSurfTriangulation::register_factory(session);
}

/// One command: name, help, body.
type CommandEntry = (&'static str, String, SessionCommand);

/// `theCommands.Add(name, help, __FILE__, body, group)` for each entry.
fn register(session: &DrawSession, group: &str, entries: &[CommandEntry]) -> Result<(), Failure> {
    for (name, help, body) in entries {
        session.add_command(name, help, FILE, *body, group)?;
    }
    Ok(())
}

/// The display commands other than the color and marker ones.
fn display_commands() -> Vec<CommandEntry> {
    vec![
        ("nbiso", "nbiso name [names...] nuiso nviso".into(), nbiso),
        (
            "clpoles",
            "clpoles [name], no args : modal ".into(),
            drawpoles,
        ),
        (
            "shpoles",
            "shpoles [name], no args : modal ".into(),
            drawpoles,
        ),
        (
            "clknots",
            "clknots [name], no args : modal ".into(),
            drawpoles,
        ),
        (
            "shknots",
            "shknots [name], no args : modal ".into(),
            drawpoles,
        ),
        ("dmode", "dmode [names...] Uniform/Discret".into(), draw),
        ("discr", "discr [names...] nbintervals".into(), draw),
        ("defle", "defle [names...] defle".into(), draw),
    ]
}

/// `nbiso [names...] nu nv`: without names, prints (and with two numbers sets) the
/// default iso counts; with names, sets the isos of those surfaces.
fn nbiso(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let n = a.len();
    if n < 4 {
        let params = parameters(session);
        if n == 3 {
            let mut params = params.borrow_mut();
            params.nb_u_isos = session.atoi(a[1]);
            params.nb_v_isos = session.atoi(a[2]);
        }
        let (nu, nv) = (params.borrow().nb_u_isos, params.borrow().nb_v_isos);
        di.append(nu)?.append(" ")?.append(nv)?;
        return Ok(0);
    }
    for name in &a[1..n - 2] {
        let Some(drawable) = session.get(name) else {
            continue;
        };
        if let Some(surface) = drawable.downcast_kind::<DrawTrSurfSurface>() {
            surface.show_isos(session.atoi(a[n - 2]), session.atoi(a[n - 1]));
            session.repaint();
        }
    }
    Ok(0)
}

/// The modal form of `clpoles`/`shpoles`/`clknots`/`shknots`: the defaults of new
/// splines.
fn set_modal_poles(session: &DrawSession, command: &str) {
    let params = parameters(session);
    let mut params = params.borrow_mut();
    match command {
        "shpoles" => params.is_show_poles = true,
        "clpoles" => params.is_show_poles = false,
        "shknots" => params.is_show_knots = true,
        "clknots" => params.is_show_knots = false,
        _ => {}
    }
}

/// `clpoles|shpoles|clknots|shknots [name]`: the modal default without a name,
/// otherwise the poles or knots of that spline (1 for knots of a Bezier object or a
/// name that is no spline).
///
/// Deviation: the C++ code calls `Draw::Get` once per spline kind it tries, so `.`
/// asks for up to six picks; here the object is picked once.
fn drawpoles(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() == 1 {
        set_modal_poles(session, a[0]);
    }
    if a.len() < 2 {
        return Ok(0);
    }
    let applied = session
        .get(a[1])
        .is_some_and(|drawable| apply_to_spline(&*drawable, a[0]));
    if !applied {
        return Ok(1);
    }
    session.repaint();
    Ok(0)
}

/// Applies `command` to `drawable` when it is a spline, tried in the C++ order
/// (Bezier surface, BSpline surface, Bezier curve, BSpline curve, 2D Bezier, 2D
/// BSpline); false when it is none or the command is a knot command on a Bezier.
fn apply_to_spline(drawable: &dyn Drawable3D, command: &str) -> bool {
    if let Some(s) = drawable.downcast_kind::<DrawTrSurfBezierSurface>() {
        return apply_poles(command, s.poles());
    }
    if let Some(s) = drawable.downcast_kind::<DrawTrSurfBSplineSurface>() {
        return apply_spline(command, s.poles(), s.knots());
    }
    if let Some(c) = drawable.downcast_kind::<DrawTrSurfBezierCurve>() {
        return apply_poles(command, c.poles());
    }
    if let Some(c) = drawable.downcast_kind::<DrawTrSurfBSplineCurve>() {
        return apply_spline(command, c.poles(), c.knots());
    }
    if let Some(c) = drawable.downcast_kind::<DrawTrSurfBezierCurve2d>() {
        return apply_poles(command, c.poles());
    }
    if let Some(c) = drawable.downcast_kind::<DrawTrSurfBSplineCurve2d>() {
        return apply_spline(command, c.poles(), c.knots());
    }
    false
}

/// `shpoles`/`clpoles` on a Bezier object; false for the knot commands.
fn apply_poles(command: &str, poles: &PolesStyle) -> bool {
    match command {
        "shpoles" => poles.show(),
        "clpoles" => poles.clear(),
        _ => return false,
    }
    true
}

/// The four commands on a BSpline object (another command name does nothing).
fn apply_spline(command: &str, poles: &PolesStyle, knots: &KnotsStyle) -> bool {
    match command {
        "shpoles" => poles.show(),
        "clpoles" => poles.clear(),
        "shknots" => knots.show(),
        "clknots" => knots.clear(),
        _ => {}
    }
    true
}

/// `dmode|discr|defle`: with at most one argument, prints (and with one argument
/// sets) the default; with names, sets the mode of the drawable named by the first
/// argument, once per name (native quirk: `a[1]` is used for every iteration).
fn draw(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let n = a.len();
    if n <= 2 {
        print_or_set_default(session, di, a)?;
        return Ok(0);
    }
    for _ in 1..n - 1 {
        let Some(drawable) = session.get(a[1]) else {
            continue;
        };
        let Some(part) = drawable.downcast_kind::<DrawTrSurfDrawable>() else {
            continue;
        };
        match a[0] {
            "dmode" => part.set_draw_mode(i32::from(a[n - 1].starts_with(['U', 'u']))),
            "discr" => part.set_discretisation(session.atoi(a[n - 1])),
            "defle" => part.set_deflection(session.atof(a[n - 1])),
            _ => {}
        }
        session.repaint();
    }
    Ok(0)
}

/// The modal form of `dmode`/`discr`/`defle`. Native quirk kept: `dmode x` sets the
/// uniform mode whatever `x` is.
fn print_or_set_default(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<(), Failure> {
    let params = parameters(session);
    let mut params = params.borrow_mut();
    match a[0] {
        "dmode" => {
            if a.len() == 2 {
                params.draw_mode = 1;
            }
            di.append(if params.draw_mode != 0 { "u" } else { "d" })?;
        }
        "discr" => {
            if a.len() == 2 {
                params.discret = session.atoi(a[1]);
            }
            di.append(params.discret)?;
        }
        "defle" => {
            if a.len() == 2 {
                params.deflection = session.atof(a[1]);
            }
            di.append(params.deflection)?;
        }
        _ => {}
    }
    Ok(())
}
