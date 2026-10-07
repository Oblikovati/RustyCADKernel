// Rust translation of the display commands of Draw/TKDraw/DBRep/DBRep.cxx (isos, hlr,
// dispor, discretisation, triangles, polygons) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `isos`, `hlr`, `vori`, `vconn`, `triangles`, `polygons` and `discretisation`.
use super::{CommandEntry, with_drawable_shape};
use crate::dbrep::drawable_shape::HlrSettings;
use crate::dbrep::params::{DBRepParams, parameters};
use crate::dbrep::write_color_orientation;
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

/// `isos [name1 ...] [nbisos]`.
pub(super) const ISOS: CommandEntry = ("isos", "isos [name1 ...] [nbisos]", isos);
/// `hlr`.
pub(super) const HLR: CommandEntry = ("hlr", "[no]hlr, rg1, rgn, hid, ang", hlr);
/// `vori`.
pub(super) const VORI: CommandEntry = (
    "vori",
    "vori [name1 ...], edges are colored by orientation (see vconn)",
    dispor,
);
/// `triangles`.
pub(super) const TRIANGLES: CommandEntry = (
    "triangles",
    "triangles [name1]..., display triangles of shapes if exists",
    triangles,
);
/// `polygons`.
pub(super) const POLYGONS: CommandEntry = (
    "polygons",
    "polygons [name1]..., display polygons of shapes if exists",
    polygons,
);
/// `vconn`.
pub(super) const VCONN: CommandEntry = (
    "vconn",
    "vconn [name1 ...] , edges are colored by number of faces (see vori)",
    dispor,
);
/// `discretisation`.
pub(super) const DISCRETISATION: CommandEntry = (
    "discretisation",
    "discretisation [nbpoints]",
    discretisation,
);

/// `Characters(i)`: the argument has a character outside `0123456789.+-eE`.
fn characters(text: &str) -> bool {
    text.bytes().any(|b| !b"0123456789.+-eE".contains(&b))
}

/// `Float(i)`: the argument has a character outside `0123456789+-`.
fn float(text: &str) -> bool {
    text.bytes().any(|b| !b"0123456789+-".contains(&b))
}

/// `isos [name1 ...] [nbisos]`: prints or sets the default number of isos, or prints or
/// changes the isos of the named shapes (a trailing real number is an error).
fn isos(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let params = parameters(session);
    let mut last = a.len() - 1;
    if last == 0 {
        di.append(format!(
            "Current number of isos : {}\n",
            params.borrow().nb_isos
        ))?;
        return Ok(0);
    }
    if !characters(a[last]) && float(a[last]) {
        return Ok(1);
    }
    let mut new_count = None;
    if !characters(a[last]) {
        new_count = Some(session.atoi(a[last]));
        last -= 1;
    }
    if last == 0 {
        params.borrow_mut().nb_isos = new_count.unwrap_or(0);
        di.append(format!(
            "New current number of isos : {}\n",
            params.borrow().nb_isos
        ))?;
        return Ok(0);
    }
    for name in &a[1..=last] {
        with_drawable_shape(session, name, |shape| -> Result<(), Failure> {
            match new_count {
                Some(count) => shape.change_nb_isos(count, session.console().as_ref()),
                None => {
                    di.append(format!("Number of isos for {name} : {}\n", shape.nb_isos()))?;
                    Ok(())
                }
            }
        })
        .transpose()?;
    }
    if new_count.is_some() {
        session.viewer().repaint_all()?;
    }
    Ok(0)
}

/// The HLR flags an `hlr` keyword sets; `None` for an unknown keyword.
fn apply_keyword(keyword: &str, settings: &mut HlrSettings) -> Option<()> {
    match keyword.to_ascii_lowercase().as_str() {
        "nohlr" => settings.with_hlr = false,
        "hlr" => settings.with_hlr = true,
        "nohid" => settings.with_hid = false,
        "hid" => (settings.with_hlr, settings.with_hid) = (true, true),
        "norg1" => (settings.with_rg1, settings.with_rgn) = (false, false),
        "rg1" => (settings.with_hlr, settings.with_rg1, settings.with_rgn) = (true, true, false),
        "norgn" => settings.with_rgn = false,
        "rgn" => (settings.with_hlr, settings.with_rg1, settings.with_rgn) = (true, true, true),
        _ => return None,
    }
    Some(())
}

/// The defaults as HLR settings, and back.
fn default_settings(params: &DBRepParams) -> HlrSettings {
    HlrSettings {
        with_hlr: params.with_hlr,
        with_rg1: params.with_rg1,
        with_rgn: params.with_rgn,
        with_hid: params.with_hid,
        angle: params.hlr_angle,
    }
}

/// `Angle of discretization : <degrees> degrees`.
fn print_angle(di: &DrawInterpretor, angle: f64) -> Result<(), Failure> {
    di.append("Angle of discretization : ")?
        .append(angle * 180. / PI)?
        .append(" degrees\n")?;
    Ok(())
}

/// The state line of `hlr` without arguments.
fn print_hlr_state(di: &DrawInterpretor, params: &DBRepParams) -> Result<(), Failure> {
    if params.with_hlr {
        let regular = if params.with_rgn {
            " RgNLines"
        } else if params.with_rg1 {
            " Rg1Lines"
        } else {
            " no RegLines"
        };
        let hidden = if params.with_hid {
            " HiddenLines"
        } else {
            " no HiddenLines"
        };
        di.append(" HLR")?
            .append(regular)?
            .append(hidden)?
            .append("\n")?;
        print_angle(di, params.hlr_angle)?;
    } else {
        di.append(" wireframe")?;
    }
    di.append("\n")?;
    Ok(())
}

/// `hlr [keyword] [shapes...]` / `hlr ang [degrees] [shapes...]`: prints or sets the
/// default hidden-line mode, or sets it on the named shapes.
fn hlr(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let params = parameters(session);
    let n = a.len();
    if n <= 1 {
        print_hlr_state(di, &params.borrow())?;
        return Ok(0);
    }
    if n == 2 && !set_default_mode(di, &params, a[1])? {
        return Ok(1);
    }
    let mut first = 2;
    if n >= 3 && a[1].eq_ignore_ascii_case("ang") {
        first = 3;
        if n == 3 {
            let mut p = params.borrow_mut();
            p.hlr_angle = session.atof(a[2]) * PI / 180.;
            if p.hlr_angle < p.h_ang_min {
                p.hlr_angle = p.h_ang_min;
            }
            if p.hlr_angle > p.h_ang_max {
                p.hlr_angle = p.h_ang_max;
            }
        }
        print_angle(di, params.borrow().hlr_angle)?;
    }
    for name in &a[first.min(n)..] {
        let applied = with_drawable_shape(session, name, |shape| {
            let mut settings = shape.hlr_settings();
            if a[1].eq_ignore_ascii_case("ang") {
                settings.angle = session.atof(a[2]) * PI / 180.;
            } else if apply_keyword(a[1], &mut settings).is_none() {
                return false;
            }
            shape.display_hlr(settings);
            true
        });
        if applied == Some(false) {
            di.append("Syntax error")?;
            return Ok(1);
        }
    }
    session.viewer().repaint_all()?;
    Ok(0)
}

/// The two-argument form: a keyword sets the default (or `ang` prints the angle);
/// false for an unknown keyword.
fn set_default_mode(
    di: &DrawInterpretor,
    params: &std::rc::Rc<std::cell::RefCell<DBRepParams>>,
    keyword: &str,
) -> Result<bool, Failure> {
    if keyword.eq_ignore_ascii_case("ang") {
        print_angle(di, params.borrow().hlr_angle)?;
        return Ok(true);
    }
    let mut settings = default_settings(&params.borrow());
    if apply_keyword(keyword, &mut settings).is_none() {
        return Ok(false);
    }
    let mut p = params.borrow_mut();
    (p.with_hlr, p.with_rg1, p.with_rgn, p.with_hid) = (
        settings.with_hlr,
        settings.with_rg1,
        settings.with_rgn,
        settings.with_hid,
    );
    Ok(true)
}

/// `vori|vconn [name1 ...]`: color the edges of the shapes by orientation (`vori`,
/// which also prints the legend) or by face count (`vconn`).
fn dispor(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let by_orientation = a[0].eq_ignore_ascii_case("vori");
    if by_orientation {
        write_color_orientation(session.console().as_ref());
    }
    for name in &a[1..] {
        if with_drawable_shape(session, name, |shape| {
            shape.display_orientation(by_orientation)
        })
        .is_some()
        {
            session.repaint();
        }
    }
    Ok(0)
}

/// `discretisation [nbpoints]`: prints or sets the default number of points.
fn discretisation(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let params = parameters(session);
    if a.len() <= 1 {
        let points = params.borrow().discretization;
        di.append(format!("Current number of points : {points}\n"))?;
    } else {
        params.borrow_mut().discretization = session.atoi(a[1]);
    }
    Ok(0)
}

/// `triangles [name1]...`: toggles the default triangulation display, or that of each
/// named shape.
fn triangles(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() == 1 {
        let params = parameters(session);
        let toggled = !params.borrow().disp_triangles;
        params.borrow_mut().disp_triangles = toggled;
    }
    for name in &a[1..] {
        with_drawable_shape(session, name, |shape| {
            shape.set_display_triangulation(!shape.display_triangulation())
        });
    }
    session.repaint();
    Ok(0)
}

/// `polygons [name1]...`: toggles the default polygon display, or that of each named
/// shape.
fn polygons(session: &DrawSession, _di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() == 1 {
        let params = parameters(session);
        let toggled = !params.borrow().display_polygons;
        params.borrow_mut().display_polygons = toggled;
    }
    for name in &a[1..] {
        with_drawable_shape(session, name, |shape| {
            shape.set_display_polygons(!shape.display_polygons())
        });
    }
    session.repaint();
    Ok(0)
}
