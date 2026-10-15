//! `DsgPrs_AnglePresentation`: angle dimensions on an arc between two directions, on faces,
//! on a cone and around an axis.
use super::package::{DsgPrs, DsgPrsFacesAngle, DsgPrsFacesAngleInputs};
use super::support::*;
use super::symbol::{DsgPrsArrowSide, DsgPrsSymbolEnds, dsg_prs_compute_symbol};
use crate::prs3d::{Prs3dDimensionAspect, Prs3dDrawer};
use rustycad_tkernel::standard::c_string::{FormatArg, sprintf_into};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tkernel::units_api::UnitsApi;
use rustycad_tkgeombase::{gc, gce};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPolylines, Graphic3dArrayOfSegments, Graphic3dGroup,
};
use std::f64::consts::PI;
use std::sync::Arc;

/// `DsgPrs_AnglePresentation`.
pub struct DsgPrsAnglePresentation;

fn aspect_ref(aspect: &Handle<Prs3dDimensionAspect>) -> Result<&Prs3dDimensionAspect, Failure> {
    aspect
        .get()
        .ok_or_else(|| null_object("DsgPrs: the dimension aspect is null"))
}

/// `Sprintf(buffer, format, value)` as UTF-8 text.
pub(super) fn c_format_real(format: &[u8], value: f64) -> String {
    let mut buffer = Vec::new();
    let _ = sprintf_into(&mut buffer, format, &[FormatArg::Real(value)]);
    buffer.pop();
    String::from_utf8_lossy(&buffer).into_owned()
}

/// The arc of the angle between `dir1` and `dir2` on the circle through the offset point,
/// the attachment leaders and the arrow points and directions (the shared body of the
/// direction overloads).
struct AngleArc {
    prims: Graphic3dArrayOfPolylines,
    first: (Point3, Direction3),
    second: (Point3, Direction3),
}

/// The parameters of the arc: start, end and the arrow parameters `uc1`/`uc2`.
fn arc_parameters(
    cer: &Circle3,
    center: &Point3,
    dir2: &Direction3,
    offset: &Point3,
    value: f64,
) -> Result<(f64, f64, f64, f64), Failure> {
    let p2 = center.translated(Vector3::from(*dir2) * cer.radius());
    let (mut uc1, mut uc2) = (0.0, elclib::parameter(cer, p2)?);
    let uco = elclib::parameter(cer, *offset)?;
    let (mut udeb, mut ufin) = (uc1, uc2);
    if uco > ufin && value.abs() < PI && uco > udeb + PI && uco < ufin + PI {
        udeb += PI;
        ufin += PI;
        uc1 = udeb;
        uc2 = ufin;
    }
    if uco > ufin {
        if (uco - uc2) < (uc1 - uco + (2.0 * PI)) {
            ufin = uco;
        } else {
            udeb = uco - 2.0 * PI;
        }
    }
    Ok((udeb, ufin, uc1, uc2))
}

/// The shared arc construction: the arc bound, then the two arrow points rotated by `beta`.
#[allow(clippy::too_many_arguments)] // The native local inputs of the shared arc body.
fn angle_arc(
    cer: &Circle3,
    center: &Point3,
    normal: &Direction3,
    dir2: &Direction3,
    offset: &Point3,
    value: f64,
    length: f64,
    attach: [&Point3; 2],
) -> Result<AngleArc, Failure> {
    let (udeb, ufin, uc1, uc2) = arc_parameters(cer, center, dir2, offset, value)?;
    let alpha = (ufin - udeb).abs();
    let nbp = 4.max((50.0 * alpha / PI) as i32);
    let dteta = alpha / (nbp - 1) as f64;
    let prims = Graphic3dArrayOfPolylines::new(nbp + 4, 3, 0, 0)?;
    prims.add_bound(nbp)?;
    for i in 1..=nbp {
        prims.add_vertex_pnt(&elclib::value(udeb + dteta * (i - 1) as f64, cer))?;
    }
    let (ptarr, vecarr) = elclib::d1(uc1, cer);
    let mut ax1 = Axis1::new(ptarr, *normal);
    let mut dirarr = Direction3::from_vector(-vecarr)?;
    let ptarr2 = Point3::from(ptarr.xyz() + dirarr.xyz() * length);
    let ptarr3 = elclib::value(elclib::parameter(cer, ptarr2)?, cer);
    let beta = Vector3::between(ptarr, ptarr2).angle(Vector3::between(ptarr, ptarr3))?;
    dirarr.rotate(ax1, beta)?;
    prims.add_bound(2)?;
    prims.add_vertex_pnt(attach[0])?;
    prims.add_vertex_pnt(&ptarr)?;
    let (ptarr1, vecarr1) = elclib::d1(uc2, cer);
    ax1.set_location(ptarr1);
    let mut dirarr2 = Direction3::from_vector(vecarr1)?;
    dirarr2.rotate(ax1, -beta)?;
    prims.add_bound(2)?;
    prims.add_vertex_pnt(attach[1])?;
    prims.add_vertex_pnt(&ptarr1)?;
    Ok(AngleArc {
        prims,
        first: (ptarr, dirarr),
        second: (ptarr1, dirarr2),
    })
}

/// The dimension arrow length, never below 1e-4.
fn arc_arrow_length(la: &Prs3dDimensionAspect) -> f64 {
    let length = arrow_length(la);
    if length < confusion() { 1.0e-4 } else { length }
}

/// `Norm = dir1 ^ dir2`, reversed beyond a half turn.
fn plane_normal(dir1: &Direction3, dir2: &Direction3, value: f64) -> Result<Direction3, Failure> {
    let norm = dir1.crossed(*dir2)?;
    Ok(if value.abs() > PI {
        norm.reversed()
    } else {
        norm
    })
}

/// Draws the arc with arrows at both ends and the text (the overloads without ArrowSide).
#[allow(clippy::too_many_arguments)] // The shared body of three native overloads.
fn arc_with_arrows(
    group: &Handle<dyn Graphic3dGroup>,
    la: &Prs3dDimensionAspect,
    text: &ExtendedString,
    center: &Point3,
    a1: &Point3,
    a2: &Point3,
    normal: &Direction3,
    dir1: &Direction3,
    dir2: &Direction3,
    offset: &Point3,
    value: f64,
) -> Result<(), Failure> {
    let cer = Circle3::new(
        Axis2::with_x_direction(*center, *normal, *dir1)?,
        center.distance(offset),
    )?;
    let length = arc_arrow_length(la);
    let arc = angle_arc(&cer, center, normal, dir2, offset, value, length, [a1, a2])?;
    draw_text(group, la, text, offset);
    let angle = super::support::arrow_angle_length(la).0;
    crate::prs3d::Prs3dArrow::draw(group, &arc.first.0, &arc.first.1, angle, length)?;
    crate::prs3d::Prs3dArrow::draw(group, &arc.second.0, &arc.second.1, angle, length)?;
    add_array(group, &Handle::new(arc.prims));
    Ok(())
}

impl DsgPrsAnglePresentation {
    /// `Add(prs, drawer, theval, CenterPoint, A1, A2, dir1, dir2, OffsetPoint)`: the text is
    /// the value printed with `%5.2f`.
    /// Example: `DsgPrsAnglePresentation::add_value(&prs, &drawer, 0.5, &c, &a1, &a2, &d1, &d2, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_value(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        center: &Point3,
        a1: &Point3,
        a2: &Point3,
        dir1: &Direction3,
        dir2: &Direction3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let text =
            ExtendedString::from_ascii(&AsciiString::from(c_format_real(b"%5.2f", value).as_str()));
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let normal = plane_normal(dir1, dir2, value)?;
        arc_with_arrows(
            &group, la, &text, center, a1, a2, &normal, dir1, dir2, offset, value,
        )
    }

    /// `Add(prs, drawer, theval, text, CenterPoint, A1, A2, dir1, dir2, OffsetPoint)`: parallel
    /// directions take the plane through the offset point.
    /// Example: `DsgPrsAnglePresentation::add(&prs, &drawer, 0.5, &t, &c, &a1, &a2, &d1, &d2, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        text: &ExtendedString,
        center: &Point3,
        a1: &Point3,
        a2: &Point3,
        dir1: &Direction3,
        dir2: &Direction3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let mut normal = if !dir1.is_parallel(*dir2, angular()) {
            dir1.crossed(*dir2)?
        } else {
            dir1.crossed(direction_between(center, offset)?)?
        };
        if value.abs() > PI {
            normal.reverse();
        }
        arc_with_arrows(
            &group, la, text, center, a1, a2, &normal, dir1, dir2, offset, value,
        )
    }

    /// `Add(prs, drawer, theval, text, CenterPoint, A1, A2, dir1, dir2, axisdir, OffsetPoint)`:
    /// the arc in the plane of `axisdir`.
    /// Example: `DsgPrsAnglePresentation::add_axis(&prs, &drawer, 0.5, &t, &c, &a1, &a2, &d1, &d2, &ax, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_axis(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        text: &ExtendedString,
        center: &Point3,
        a1: &Point3,
        a2: &Point3,
        dir1: &Direction3,
        dir2: &Direction3,
        axis_dir: &Direction3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        arc_with_arrows(
            &group, la, text, center, a1, a2, axis_dir, dir1, dir2, offset, value,
        )
    }

    /// `Add(prs, drawer, theval, text, CenterPoint, A1, A2, dir1, dir2, OffsetPoint, ArrowSide)`:
    /// the arc with computed end symbols.
    /// Example: `DsgPrsAnglePresentation::add_with_arrow(&prs, &drawer, 0.5, &t, &c, &a1, &a2, &d1, &d2, &o, DsgPrsArrowSide::BothAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_with_arrow(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        text: &ExtendedString,
        center: &Point3,
        a1: &Point3,
        a2: &Point3,
        dir1: &Direction3,
        dir2: &Direction3,
        offset: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let normal = plane_normal(dir1, dir2, value)?;
        let cer = Circle3::new(
            Axis2::with_x_direction(*center, normal, *dir1)?,
            center.distance(offset),
        )?;
        let arc = angle_arc(
            &cer,
            center,
            &normal,
            dir2,
            offset,
            value,
            arc_arrow_length(la),
            [a1, a2],
        )?;
        draw_text(&group, la, text, offset);
        add_array(&group, &Handle::new(arc.prims));
        let ends = DsgPrsSymbolEnds {
            first: arc.first.0,
            last: arc.second.0,
            first_direction: arc.first.1,
            last_direction: arc.second.1,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }

    /// `Add(prs, drawer, theval, text, CenterPoint, A1, A2, dir1, dir2, axisdir, isPlane,
    /// AxisOfSurf, OffsetPoint, ArrowSide)`: the faces angle layout of
    /// `DsgPrs::ComputeFacesAnglePresentation`.
    /// Example: `DsgPrsAnglePresentation::add_faces(&prs, &drawer, &t, &inputs, DsgPrsArrowSide::BothAr)?`.
    pub fn add_faces(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        inputs: &DsgPrsFacesAngleInputs,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let inputs = DsgPrsFacesAngleInputs {
            arrow_length: arrow_length(la),
            ..*inputs
        };
        let mut layout = DsgPrsFacesAngle::default();
        DsgPrs::compute_faces_angle_presentation(&inputs, &mut layout)?;
        let value = inputs.value;
        let prims = if value > angular() && (PI - value).abs() > angular() {
            add_array(
                &group,
                &arc_polyline(
                    &layout.angle_circ,
                    layout.first_par_angle_circ,
                    layout.last_par_angle_circ,
                    true,
                )?,
            );
            Graphic3dArrayOfSegments::new(4, 0, 0)?
        } else {
            let prims = Graphic3dArrayOfSegments::new(6, 0, 0)?;
            prims.add_vertex_pnt(&inputs.offset_point)?;
            prims.add_vertex_pnt(&layout.end_of_arrow1)?;
            prims
        };
        let ends = DsgPrsSymbolEnds {
            first: layout.end_of_arrow1,
            last: layout.end_of_arrow2,
            first_direction: layout.dir_of_arrow1,
            last_direction: layout.dir_of_arrow2,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)?;
        let group = current_group(prs)?;
        draw_text(&group, la, text, &inputs.offset_point);
        for point in [
            &inputs.attachment_point1,
            &layout.end_of_arrow1,
            &layout.proj_attach_point2,
            &layout.end_of_arrow2,
        ] {
            prims.add_vertex_pnt(point)?;
        }
        add_array(&group, &Handle::new(prims));
        if inputs
            .attachment_point2
            .distance(&layout.proj_attach_point2)
            <= confusion()
        {
            return Ok(());
        }
        if inputs.is_plane {
            add_array(
                &group,
                &segments(2, &[inputs.attachment_point2, layout.proj_attach_point2])?,
            );
        } else {
            add_array(
                &group,
                &arc_polyline(
                    &layout.attach_circ,
                    layout.first_par_attach_circ,
                    layout.last_par_attach_circ,
                    false,
                )?,
            );
        }
        Ok(())
    }
}

// `max(4, int(50 * Alpha / PI))` points from the first parameter, in a one-bound
// `(NodeNumber + 4, 3)` array for the angle arc or a plain `(NodeNumber)` array.
fn arc_polyline(
    circle: &Circle3,
    mut first: f64,
    last: f64,
    bounded: bool,
) -> Result<Handle<Graphic3dArrayOfPolylines>, Failure> {
    let alpha = (last - first).abs();
    let node_number = 4.max((50.0 * alpha / PI) as i32);
    let delta = alpha / (node_number - 1) as f64;
    let prims = if bounded {
        let prims = Graphic3dArrayOfPolylines::new(node_number + 4, 3, 0, 0)?;
        prims.add_bound(node_number)?;
        prims
    } else {
        Graphic3dArrayOfPolylines::new(node_number, 0, 0, 0)?
    };
    for _ in 0..node_number {
        prims.add_vertex_pnt(&elclib::value(first, circle))?;
        first += delta;
    }
    Ok(Handle::new(prims))
}

impl DsgPrsAnglePresentation {
    /// `Add(prs, drawer, theval, theCenter, A1, theAxe, ArrowSide)`: the arc of the angle
    /// around an axis from the attachment point, with arrows at the requested ends.
    /// Example: `DsgPrsAnglePresentation::add_revolution(&prs, &drawer, 1.0, &c, &a1, &axis, DsgPrsArrowSide::BothAr)?`.
    pub fn add_revolution(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        value: f64,
        center: &Point3,
        a1: &Point3,
        axis: &Axis1,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let dir1 = direction_between(center, a1)?;
        let cer = Circle3::new(
            Axis2::with_x_direction(*center, axis.direction(), dir1)?,
            center.distance(a1),
        )?;
        let nbp = 4.max((50.0 * value / PI) as i32);
        let dteta = value / (nbp - 1) as f64;
        let points: Vec<Point3> = (1..=nbp)
            .map(|i| elclib::value(dteta * (i - 1) as f64, &cer))
            .collect();
        add_array(&group, &polyline(&points)?);
        let uc2 = elclib::parameter(&cer, a1.rotated(*axis, value)?)?;
        let length = arc_arrow_length(la);
        let angle = arrow_angle_length(la).0;
        let group = current_group(prs)?;
        let arrow = |parameter: f64, reverse: bool| -> Result<(), Failure> {
            let (point, vector) = elclib::d1(parameter, &cer);
            let direction = Direction3::from_vector(if reverse { -vector } else { vector })?;
            crate::prs3d::Prs3dArrow::draw(&group, &point, &direction, angle, length)
        };
        match side {
            DsgPrsArrowSide::FirstAr => arrow(0.0, true),
            DsgPrsArrowSide::LastAr => arrow(uc2, false),
            DsgPrsArrowSide::BothAr => {
                arrow(0.0, true)?;
                arrow(uc2, false)
            }
            _ => Ok(()),
        }
    }
}

/// `AboveInBelowCone`: -1 below, 1 above, 0 inside the cone limits.
fn above_in_below_cone(c_max: &Circle3, c_min: &Circle3, c: &Circle3) -> i32 {
    let d = c_max.location().distance(&c_min.location());
    let d1 = c_max.location().distance(&c.location());
    let d2 = c_min.location().distance(&c.location());
    if d >= d1 && d >= d2 {
        return 0;
    }
    if d < d2 && d1 < d2 {
        return -1;
    }
    if d < d1 && d2 < d1 {
        return 1;
    }
    0
}

/// The inputs of the cone angle overload.
#[derive(Clone, Copy, Debug)]
pub struct DsgPrsConeAngle {
    /// `aCircle`.
    pub circle: Circle3,
    /// `aPosition`.
    pub position: Point3,
    /// `Apex`.
    pub apex: Point3,
    /// `VminCircle`.
    pub v_min_circle: Circle3,
    /// `VmaxCircle`.
    pub v_max_circle: Circle3,
    /// `aArrowSize` (0 for a tenth of the circle radius).
    pub arrow_size: f64,
}

impl DsgPrsAnglePresentation {
    /// `Add(prs, drawer, aVal, text, aCircle, aPosition, Apex, VminCircle, VmaxCircle,
    /// aArrowSize)`: the cone angle. An empty text shows the half angle in the current
    /// "PLANE ANGLE" unit of `units` (the native process-wide UnitsAPI). The arrow length of
    /// the dimension aspect and of the drawer arrow aspect is changed, as natively.
    /// Example: `DsgPrsAnglePresentation::add_cone(&prs, &drawer, 0.0, &t, &cone, &units)?`.
    pub fn add_cone(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        _value: f64,
        text: &ExtendedString,
        cone: &DsgPrsConeAngle,
        units: &Arc<SharedState<UnitsApi>>,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let arrow_size = if cone.arrow_size == 0.0 {
            0.1 * cone.circle.radius()
        } else {
            cone.arrow_size
        };
        set_arrow_lengths(la, drawer, arrow_size);
        let mut my_circle = cone.circle;
        let is_cone_trimmed = cone.v_min_circle.radius() > 0.01;
        if is_cone_trimmed
            && above_in_below_cone(&cone.v_max_circle, &cone.v_min_circle, &my_circle) == 1
        {
            my_circle = cone.v_min_circle;
        }
        let p1 = elclib::value(0.0, &my_circle);
        let p2 = elclib::value(PI, &my_circle);
        let plane = gce::MakePln::through_points(p1, p2, cone.apex)?.value()?;
        let a_vector = Vector3::between(plane.location(), cone.position);
        let mut normal = Vector3::from(plane.axis().direction());
        normal = normal * (a_vector * normal);
        let tmp_pnt = cone.position.translated(-normal);
        let (attachment, opposite) = if tmp_pnt.distance(&p1) < tmp_pnt.distance(&p2) {
            (p1, p2)
        } else {
            (p2, p1)
        };
        let mut a_pnt = attachment;
        let mut vec = Vector3::between(attachment, cone.apex);
        vec.scale(2.0);
        a_pnt.translate(vec);
        let circle2 = gc::MakeCircle::through_points(attachment, opposite, a_pnt)?.value()?;
        let circle2 = circle2
            .get()
            .ok_or_else(|| null_object("DsgPrs_AnglePresentation: no circle"))?
            .circ()?;
        let mut att_param = elclib::parameter(&circle2, attachment)?;
        let mut opp_param = elclib::parameter(&circle2, opposite)?;
        while att_param >= 2.0 * PI {
            att_param -= 2.0 * PI;
        }
        while opp_param >= 2.0 * PI {
            opp_param -= 2.0 * PI;
        }
        let txt = if text.length() == 0 {
            let angle = units
                .write()
                .current_from_ls(opp_param.abs(), b"PLANE ANGLE")?;
            ExtendedString::from_ascii(&AsciiString::from(c_format_real(b"%g", angle).as_str()))
        } else {
            text.clone()
        };
        let frame = ConeFrame {
            circle2,
            attachment,
            opposite,
            att_param,
            opp_param,
            tmp_pnt,
            arrow_size,
            my_circle,
            p1,
            p2,
        };
        draw_cone_angle(prs, &aspect, la, &txt, cone, &frame, is_cone_trimmed)
    }
}

fn set_arrow_lengths(la: &Prs3dDimensionAspect, drawer: &Handle<Prs3dDrawer>, length: f64) {
    if let Some(arrow) = la.arrow_aspect().get() {
        arrow.set_length(length);
    }
    let drawer_arrow = drawer.get().map(|drawer| drawer.arrow_aspect());
    if let Some(arrow) = drawer_arrow.as_ref().and_then(|arrow| arrow.get()) {
        arrow.set_length(length);
    }
}

/// The derived geometry of the cone angle.
struct ConeFrame {
    circle2: Circle3,
    attachment: Point3,
    opposite: Point3,
    att_param: f64,
    opp_param: f64,
    tmp_pnt: Point3,
    arrow_size: f64,
    my_circle: Circle3,
    p1: Point3,
    p2: Point3,
}

fn draw_cone_angle(
    prs: &DsgPrsPresentation,
    aspect: &Handle<Prs3dDimensionAspect>,
    la: &Prs3dDimensionAspect,
    txt: &ExtendedString,
    cone: &DsgPrsConeAngle,
    f: &ConeFrame,
    is_cone_trimmed: bool,
) -> Result<(), Failure> {
    let mut is_arrow_out = true;
    if elclib::parameter(&f.circle2, f.tmp_pnt)? < f.opp_param
        && 2.0 * f.my_circle.radius() > 4.0 * f.arrow_size
    {
        is_arrow_out = false;
    }
    let mut angle = f.opp_param - f.att_param;
    let offset = if is_arrow_out { -PI / 12.0 } else { PI / 12.0 };
    let a_dir = direction_between(
        &elclib::value(f.att_param + offset, &f.circle2),
        &f.attachment,
    )?;
    let a_dir2 = direction_between(
        &elclib::value(f.opp_param - offset, &f.circle2),
        &f.opposite,
    )?;
    while angle > 2.0 * PI {
        angle -= 2.0 * PI;
    }
    let points: Vec<Point3> = (0..=11)
        .map(|i| elclib::value(f.att_param + angle / 11.0 * i as f64, &f.circle2))
        .collect();
    add_array(&current_group(prs)?, &polyline(&points)?);
    for (point, direction) in [(f.attachment, a_dir), (f.opposite, a_dir2)] {
        let ends = DsgPrsSymbolEnds {
            first: point,
            last: point,
            first_direction: direction,
            last_direction: direction,
        };
        dsg_prs_compute_symbol(prs, aspect, &ends, DsgPrsArrowSide::LastAr, true)?;
    }
    let param = elclib::parameter(&f.circle2, f.tmp_pnt)?;
    let text_pnt = elclib::value(param, &f.circle2).translated(Vector3::new(0.0, 0.0, -2.0));
    let group = current_group(prs)?;
    draw_text(&group, la, txt, &text_pnt);
    let mut angle = 2.0 * PI - param;
    if param > f.opp_param {
        while angle > 2.0 * PI {
            angle -= 2.0 * PI;
        }
        let points: Vec<Point3> = (0..=11)
            .rev()
            .map(|i| elclib::value(-angle / 11.0 * i as f64, &f.circle2))
            .collect();
        add_array(&group, &polyline(&points)?);
    }
    cone_generatrices(&group, cone, f, is_cone_trimmed)
}

fn cone_generatrices(
    group: &Handle<dyn Graphic3dGroup>,
    cone: &DsgPrsConeAngle,
    f: &ConeFrame,
    is_cone_trimmed: bool,
) -> Result<(), Failure> {
    let position = above_in_below_cone(&cone.v_max_circle, &cone.v_min_circle, &f.my_circle);
    if position == 1 && !is_cone_trimmed {
        add_array(group, &polyline(&[f.attachment, cone.apex, f.opposite])?);
        return Ok(());
    }
    if position == 0 {
        return Ok(());
    }
    let a_pnt = f.opposite;
    let p11 = elclib::value(0.0, &cone.v_max_circle);
    let p12 = elclib::value(PI, &cone.v_max_circle);
    let near = a_pnt.distance(&f.p1) < a_pnt.distance(&f.p2);
    let (first, second) = if near { (p12, p11) } else { (p11, p12) };
    add_array(
        group,
        &segments(4, &[f.attachment, first, f.opposite, second])?,
    );
    Ok(())
}
