//! The diameter, radius, ellipse radius and fillet radius presentations.
use super::package::{DsgPrs, DsgPrsFillet, DsgPrsFilletInputs};
use super::support::*;
use super::symbol::{DsgPrsArrowSide, DsgPrsSymbolEnds, dsg_prs_compute_symbol};
use crate::prs3d::{Prs3dDimensionAspect, Prs3dDrawer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomEllipse, GeomOffsetCurve, GeomTrimmedCurve};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Direction3, Ellipse3, Line3, Point3, Vector3};
use std::f64::consts::PI;

fn aspect_ref(aspect: &Handle<Prs3dDimensionAspect>) -> Result<&Prs3dDimensionAspect, Failure> {
    aspect
        .get()
        .ok_or_else(|| null_object("DsgPrs: the dimension aspect is null"))
}

/// `TCollection_ExtendedString(prefix) + text`, the prefix bytes taken as Latin-1 as the
/// non-multibyte `char*` constructor does (`"\330"` is U+00D8).
fn prefixed(prefix: &[u8], text: &ExtendedString) -> ExtendedString {
    &ExtendedString::from_c_string(prefix, false) + text.clone()
}

/// `DsgPrs_DiameterPresentation`.
pub struct DsgPrsDiameterPresentation;

impl DsgPrsDiameterPresentation {
    /// `Add(prs, drawer, text, AttachmentPoint, circle, ArrowSide, IsDiamSymbol)`: a full
    /// diameter through the attachment point's circle point.
    /// Example: `DsgPrsDiameterPresentation::add(&prs, &drawer, &t, &p, &circle, DsgPrsArrowSide::BothAr, true)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        attachment: &Point3,
        circle: &Circle3,
        side: DsgPrsArrowSide,
        is_diam_symbol: bool,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let parat = elclib::parameter(circle, *attachment)?;
        let ptoncirc = elclib::value(parat, circle);
        let center = circle.location();
        let mut vecrap = Vector3::between(ptoncirc, center);
        let mut dist = center.distance(attachment);
        let radius = circle.radius();
        let inside = dist < radius;
        let mut pt1 = *attachment;
        if inside {
            pt1 = ptoncirc;
            dist = radius;
        }
        vecrap.normalize()?;
        vecrap *= dist + radius;
        let opposite = pt1.translated(vecrap);
        add_array(&group, &segments(2, &[pt1, opposite])?);
        let shown = if is_diam_symbol {
            prefixed(b"\xd8  ", text)
        } else {
            text.clone()
        };
        draw_text(&current_group(prs)?, la, &shown, attachment);
        let mut arrdir = Direction3::from_vector(vecrap)?;
        if inside {
            arrdir.reverse();
        }
        let mut vecrap2 = vecrap;
        vecrap2.normalize()?;
        vecrap2 *= circle.radius() * 2.0;
        let ptoncirc2 = ptoncirc.translated(vecrap2);
        let ends = DsgPrsSymbolEnds {
            first: ptoncirc,
            last: ptoncirc2,
            first_direction: arrdir,
            last_direction: arrdir.reversed(),
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }

    /// `Add(prs, drawer, text, AttachmentPoint, circle, uFirst, uLast, ArrowSide, IsDiamSymbol)`:
    /// a radius of an arc, clamped to its nearer end outside the domain.
    /// Example: `DsgPrsDiameterPresentation::add_arc(&prs, &drawer, &t, &p, &circle, 0.0, 1.0, DsgPrsArrowSide::LastAr, false)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_arc(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        attachment: &Point3,
        circle: &Circle3,
        u_first: f64,
        u_last: f64,
        side: DsgPrsArrowSide,
        is_diam_symbol: bool,
    ) -> Result<(), Failure> {
        let (mut fpara, mut lpara) = (u_first, u_last);
        while lpara > 2.0 * PI {
            fpara -= 2.0 * PI;
            lpara -= 2.0 * PI;
        }
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let (end_of_arrow, draw_position) =
            arc_arrow_end(circle, attachment, u_first, u_last, fpara, lpara)?;
        add_array(&group, &segments(2, &[draw_position, end_of_arrow])?);
        let shown = if is_diam_symbol {
            prefixed(b"\xd8 ", text)
        } else {
            text.clone()
        };
        draw_text(&current_group(prs)?, la, &shown, &draw_position);
        let dir = Direction3::from_xyz(*Vector3::between(draw_position, end_of_arrow).xyz())?;
        let ends = DsgPrsSymbolEnds {
            first: end_of_arrow,
            last: end_of_arrow,
            first_direction: dir,
            last_direction: dir,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }
}

/// `DsgPrs_InDomain` of the diameter presentation (with the wrapped-domain branch).
fn diameter_in_domain(fpar: f64, lpar: f64, para: f64) -> bool {
    if fpar >= 0.0 {
        if lpar > fpar {
            return para >= fpar && para <= lpar;
        }
        let delta = 2.0 * PI - fpar;
        let (mut lp, mut par) = (lpar + delta, para + delta);
        while lp > 2.0 * PI {
            lp -= 2.0 * PI;
        }
        while par > 2.0 * PI {
            par -= 2.0 * PI;
        }
        return par >= 0.0 && par <= lp;
    }
    para >= fpar + 2.0 * PI || para <= lpar
}

// The arrow end on the arc (or its opposite point, or the nearer arc end) and the text point.
fn arc_arrow_end(
    circle: &Circle3,
    attachment: &Point3,
    u_first: f64,
    u_last: f64,
    fpara: f64,
    lpara: f64,
) -> Result<(Point3, Point3), Failure> {
    let mut par = elclib::parameter(circle, *attachment)?;
    let center = circle.location();
    let first_point = elclib::value(u_first, circle);
    let second_point = elclib::value(u_last, circle);
    if diameter_in_domain(fpara, lpara, par) {
        return Ok((elclib::value(par, circle), *attachment));
    }
    let mut other = par + PI;
    if other > 2.0 * PI {
        other -= 2.0 * PI;
    }
    if diameter_in_domain(fpara, lpara, other) {
        par = other;
        return Ok((elclib::value(par, circle), *attachment));
    }
    let l1 = Line3::new(center, direction_between(&center, &first_point)?);
    let l2 = Line3::new(center, direction_between(&center, &second_point)?);
    if l1.distance(*attachment) < l2.distance(*attachment) {
        Ok((first_point, project_on_line(&l1, attachment)?))
    } else {
        Ok((second_point, project_on_line(&l2, attachment)?))
    }
}

/// `DsgPrs_RadiusPresentation`.
pub struct DsgPrsRadiusPresentation;

/// `DsgPrs_InDomain` of the radius presentation.
fn radius_in_domain(fpar: f64, lpar: f64, para: f64) -> bool {
    if fpar >= 0.0 {
        return para >= fpar && para <= lpar;
    }
    para >= fpar + 2.0 * PI || para <= lpar
}

impl DsgPrsRadiusPresentation {
    /// `Add(prs, drawer, text, AttachmentPoint, circle, firstparam, lastparam, drawFromCenter,
    /// reverseArrow)`: a radius line with an arrow on the arc.
    /// Example: `DsgPrsRadiusPresentation::add(&prs, &drawer, &t, &p, &circle, 0.0, PI, true, false)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        attachment: &Point3,
        circle: &Circle3,
        first_param: f64,
        last_param: f64,
        draw_from_center: bool,
        reverse_arrow: bool,
    ) -> Result<(), Failure> {
        let (mut fpara, mut lpara) = (first_param, last_param);
        while lpara > 2.0 * PI {
            fpara -= 2.0 * PI;
            lpara -= 2.0 * PI;
        }
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let (parat, attpoint, otherside) = radius_attachment(circle, attachment, fpara, lpara)?;
        let ptoncirc = elclib::value(parat, circle);
        let center = circle.location();
        let l = Line3::new(center, Direction3::from_xyz(attpoint.xyz() - center.xyz())?);
        let (mut firstpoint, mut drawtopoint) = (attpoint, ptoncirc);
        if draw_from_center && !otherside {
            let uatt = elclib::parameter(&l, attpoint)?;
            let uptc = elclib::parameter(&l, ptoncirc)?;
            if uatt.abs() > uptc.abs() {
                drawtopoint = center;
            } else {
                firstpoint = center;
            }
        }
        add_array(&group, &segments(2, &[firstpoint, drawtopoint])?);
        let arrdir = if reverse_arrow {
            l.direction().reversed()
        } else {
            l.direction()
        };
        let group = current_group(prs)?;
        draw_arrow(&group, la, &ptoncirc, &arrdir)?;
        draw_text(&group, la, text, &attpoint);
        Ok(())
    }

    /// `Add(prs, drawer, text, AttachmentPoint, Center, EndOfArrow, ArrowSide, drawFromCenter,
    /// reverseArrow)`: a radius line between computed ends with end symbols.
    /// Example: `DsgPrsRadiusPresentation::add_center(&prs, &drawer, &t, &p, &c, &e, DsgPrsArrowSide::LastAr, true, false)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_center(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        attachment: &Point3,
        center: &Point3,
        end_of_arrow: &Point3,
        side: DsgPrsArrowSide,
        draw_from_center: bool,
        reverse_arrow: bool,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let (origin, end) =
            DsgPrs::compute_radius_line(center, end_of_arrow, attachment, draw_from_center)?;
        add_array(&group, &segments(2, &[origin, end])?);
        draw_text(&current_group(prs)?, la, text, attachment);
        let mut dir = make_dir(&origin, &end)?;
        if reverse_arrow {
            dir.reverse();
        }
        let ends = DsgPrsSymbolEnds {
            first: *center,
            last: *end_of_arrow,
            first_direction: dir.reversed(),
            last_direction: dir,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, draw_from_center)
    }
}

// The arc parameter of the arrow, the (possibly moved) attachment point and whether the
// opposite side is used.
fn radius_attachment(
    circle: &Circle3,
    attachment: &Point3,
    fpara: f64,
    lpara: f64,
) -> Result<(f64, Point3, bool), Failure> {
    let mut parat = elclib::parameter(circle, *attachment)?;
    if radius_in_domain(fpara, lpara, parat) {
        return Ok((parat, *attachment, false));
    }
    let mut otherpar = parat + PI;
    if otherpar > 2.0 * PI {
        otherpar -= 2.0 * PI;
    }
    if radius_in_domain(fpara, lpara, otherpar) {
        return Ok((otherpar, *attachment, true));
    }
    let ecartpar = (fpara - parat).abs().min((lpara - parat).abs());
    let ecartoth = (fpara - otherpar).abs().min((lpara - otherpar).abs());
    let otherside = ecartpar > ecartoth;
    parat = if !otherside {
        if parat < fpara { fpara } else { lpara }
    } else if otherpar < fpara {
        fpara
    } else {
        lpara
    };
    let ptdir = elclib::value(parat, circle);
    let center = circle.location();
    let lsup = Line3::new(center, Direction3::from_xyz(ptdir.xyz() - center.xyz())?);
    Ok((parat, project_on_line(&lsup, attachment)?, otherside))
}

/// `DsgPrs_EllipseRadiusPresentation`.
pub struct DsgPrsEllipseRadiusPresentation;

impl DsgPrsEllipseRadiusPresentation {
    /// `Add(prs, drawer, value, text, AttachmentPoint, EndOfArrow, center, IsMaxRadius,
    /// ArrowSide)`: the semi-axis line and its "a = " or "b = " text.
    /// Example: `DsgPrsEllipseRadiusPresentation::add(&prs, &drawer, 2.0, &t, &p, &e, &c, true, DsgPrsArrowSide::LastAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        text: &ExtendedString,
        position: &Point3,
        end_of_arrow: &Point3,
        center: &Point3,
        is_max_radius: bool,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let inside = center.distance(position) <= value;
        let end_point = if inside { *end_of_arrow } else { *position };
        add_array(&group, &segments(2, &[*center, end_point])?);
        let label = prefixed(if is_max_radius { b"a = " } else { b"b = " }, text);
        draw_text(&current_group(prs)?, la, &label, position);
        let mut arrdir = direction_between(center, end_of_arrow)?;
        if !inside {
            arrdir.reverse();
        }
        let ends = DsgPrsSymbolEnds {
            first: *end_of_arrow,
            last: *end_of_arrow,
            first_direction: arrdir,
            last_direction: arrdir,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }

    /// `Add(..., anEllipse, ..., uFirst, IsInDomain, IsMaxRadius, ArrowSide)`: the arc to the
    /// apex when outside the domain, then the semi-axis presentation.
    /// Example: `DsgPrsEllipseRadiusPresentation::add_ellipse(&prs, &drawer, 2.0, &t, &ellipse, &p, &e, &c, 0.5, false, true, DsgPrsArrowSide::LastAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_ellipse(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        text: &ExtendedString,
        ellipse: &Ellipse3,
        position: &Point3,
        end_of_arrow: &Point3,
        center: &Point3,
        u_first: f64,
        is_in_domain: bool,
        is_max_radius: bool,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        if !is_in_domain {
            let u_last = elclib::parameter(ellipse, *end_of_arrow)?;
            let alpha = DsgPrs::distance_from_apex(ellipse, end_of_arrow, u_first)?;
            let vapex = Vector3::between(*center, elclib::value(u_last, ellipse));
            let vpnt = Vector3::between(*center, elclib::value(u_first, ellipse));
            let dir = Direction3::from_vector(vpnt ^ vapex)?;
            let first = if ellipse.position().direction().is_opposite(dir, angular()) {
                u_last
            } else {
                u_first
            };
            add_array(
                &group,
                &polyline(&apex_arc(alpha, first, |parameter| {
                    Ok(elclib::value(parameter, ellipse))
                })?)?,
            );
        }
        Self::add(
            prs,
            drawer,
            value,
            text,
            position,
            end_of_arrow,
            center,
            is_max_radius,
            side,
        )
    }

    /// `Add(..., Geom_OffsetCurve, ...)`: as the ellipse overload on an offset ellipse; nothing
    /// is drawn for a curve that is not C1 outside the domain.
    /// Example: `DsgPrsEllipseRadiusPresentation::add_offset_curve(&prs, &drawer, 2.0, &t, &curve, &p, &e, &c, 0.5, false, true, DsgPrsArrowSide::LastAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_offset_curve(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        text: &ExtendedString,
        curve: &Handle<GeomOffsetCurve>,
        position: &Point3,
        end_of_arrow: &Point3,
        center: &Point3,
        u_first: f64,
        is_in_domain: bool,
        is_max_radius: bool,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        if !is_in_domain {
            let offset_curve = curve.get().ok_or_else(|| {
                null_object("DsgPrs_EllipseRadiusPresentation: the curve is null")
            })?;
            if !offset_curve.is_cn(1)? {
                return Ok(());
            }
            let basis: Handle<GeomEllipse> = offset_curve.basis_curve().cast();
            let basis = basis.get().ok_or_else(|| {
                null_object("DsgPrs_EllipseRadiusPresentation: the basis curve is not an ellipse")
            })?;
            let mut b_ellipse = basis.elips()?;
            let offset = offset_curve.offset();
            b_ellipse.set_major_radius(b_ellipse.major_radius() + offset)?;
            b_ellipse.set_minor_radius(b_ellipse.minor_radius() + offset)?;
            let u_last = elclib::parameter(&b_ellipse, *end_of_arrow)?;
            let alpha = DsgPrs::distance_from_apex(&b_ellipse, end_of_arrow, u_first)?;
            let p1 = offset_curve.value(u_first)?;
            let dir = Direction3::from_vector(
                Vector3::between(*center, p1) ^ Vector3::between(*center, *end_of_arrow),
            )?;
            let first = if offset_curve.direction().is_opposite(dir, angular()) {
                u_last
            } else {
                u_first
            };
            add_array(
                &group,
                &polyline(&apex_arc(alpha, first, |parameter| {
                    offset_curve.value(parameter)
                })?)?,
            );
        }
        Self::add(
            prs,
            drawer,
            value,
            text,
            position,
            end_of_arrow,
            center,
            is_max_radius,
            side,
        )
    }
}

// `max(4, int(50 * Alpha / PI))` points from `first` over `alpha`.
fn apex_arc(
    alpha: f64,
    mut first: f64,
    point_at: impl Fn(f64) -> Result<Point3, Failure>,
) -> Result<Vec<Point3>, Failure> {
    let node_number = 4.max((50.0 * alpha / PI) as i32);
    let delta = alpha / (node_number - 1) as f64;
    let mut points = Vec::with_capacity(node_number as usize);
    for _ in 0..node_number {
        points.push(point_at(first)?);
        first += delta;
    }
    Ok(points)
}

/// `DsgPrs_FilletRadiusPresentation`.
pub struct DsgPrsFilletRadiusPresentation;

/// The outputs of `DsgPrs_FilletRadiusPresentation::Add`.
#[derive(Clone, Default)]
pub struct DsgPrsFilletOutputs {
    /// `DrawPosition`.
    pub draw_position: Point3,
    /// `EndOfArrow`.
    pub end_of_arrow: Point3,
    /// `TrimCurve` (unchanged when there is no circle).
    pub trim_curve: Handle<GeomTrimmedCurve>,
    /// `HasCircle`.
    pub has_circle: bool,
}

impl DsgPrsFilletRadiusPresentation {
    /// `Add`: the fillet arc (and its trimmed circle), the radius line, the text and the end
    /// symbol. The trimmed circle starts at the parameter after the arc sampling, as natively.
    /// Example: `DsgPrsFilletRadiusPresentation::add(&prs, &drawer, 2.0, &t, &inputs, DsgPrsArrowSide::LastAr, &mut out)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        text: &ExtendedString,
        inputs: &DsgPrsFilletInputs,
        side: DsgPrsArrowSide,
        out: &mut DsgPrsFilletOutputs,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let mut layout = DsgPrsFillet {
            draw_position: out.draw_position,
            end_of_arrow: out.end_of_arrow,
            ..DsgPrsFillet::default()
        };
        let inputs = DsgPrsFilletInputs {
            arrow_length: arrow_length(la),
            value,
            ..*inputs
        };
        DsgPrs::compute_fillet_radius_presentation(&inputs, &mut layout)?;
        (out.draw_position, out.end_of_arrow) = (layout.draw_position, layout.end_of_arrow);
        if !layout.spec_case {
            let alpha = (layout.last_par_circ - layout.first_par_circ).abs();
            let node_number = 4.max((50.0 * alpha / PI) as i32);
            let delta = alpha / (node_number - 1) as f64;
            let mut points = Vec::with_capacity(node_number as usize);
            for _ in 0..node_number {
                points.push(elclib::value(layout.first_par_circ, &layout.fillet_circ));
                layout.first_par_circ += delta;
            }
            add_array(&group, &polyline(&points)?);
            out.has_circle = true;
            let circle: Handle<dyn GeomCurve> =
                Handle::new(GeomCircle::from_circ(layout.fillet_circ)).cast();
            out.trim_curve = Handle::new(GeomTrimmedCurve::new(
                &circle,
                layout.first_par_circ,
                layout.last_par_circ,
                true,
                true,
            )?);
        } else {
            out.has_circle = false;
        }
        let group = current_group(prs)?;
        add_array(
            &group,
            &segments(2, &[out.draw_position, out.end_of_arrow])?,
        );
        draw_text(&group, la, text, &out.draw_position);
        let ends = DsgPrsSymbolEnds {
            first: out.end_of_arrow,
            last: out.end_of_arrow,
            first_direction: layout.dir_of_arrow,
            last_direction: layout.dir_of_arrow,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }
}
