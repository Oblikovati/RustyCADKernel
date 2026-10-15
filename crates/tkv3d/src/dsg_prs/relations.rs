//! The identity, mid point and equal distance relation presentations.
use super::support::*;
use super::symbol::{DsgPrsArrowSide, DsgPrsSymbolEnds, dsg_prs_compute_symbol};
use crate::prs3d::{Prs3dDimensionAspect, Prs3dDrawer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis2, Circle3, Direction3, Ellipse3, Line3, Point3, StandardDirection3, Vector3,
};
use rustycad_tkservice::graphic3d::{Graphic3dArrayOfPolylines, Graphic3dGroup};
use std::f64::consts::PI;

fn aspect_ref(aspect: &Handle<Prs3dDimensionAspect>) -> Result<&Prs3dDimensionAspect, Failure> {
    aspect
        .get()
        .ok_or_else(|| null_object("DsgPrs: the dimension aspect is null"))
}

/// `DsgPrs_IdenticPresentation`.
pub struct DsgPrsIdenticPresentation;

// The angular span from the first to the second attachment and its sampling.
fn arc_sampling(p_first: f64, p_second: f64) -> (i32, f64) {
    let mut alpha = p_second - p_first;
    if alpha < 0.0 {
        alpha += 2.0 * PI;
    }
    let nbp = 4.max((50.0 * alpha / PI) as i32);
    (nbp, alpha / (nbp - 1) as f64)
}

// The arc polyline, preceded by a two-point bound from the offset point when requested.
fn arc_with_leader(
    leader: Option<(Point3, Point3)>,
    nbp: i32,
    point_at: impl Fn(i32) -> Point3,
) -> Result<Handle<Graphic3dArrayOfPolylines>, Failure> {
    let prims = match leader {
        Some((from, to)) => {
            let prims = Graphic3dArrayOfPolylines::new(nbp + 2, 2, 0, 0)?;
            prims.add_bound(2)?;
            prims.add_vertex_pnt(&from)?;
            prims.add_vertex_pnt(&to)?;
            prims.add_bound(nbp)?;
            prims
        }
        None => Graphic3dArrayOfPolylines::new(nbp, 0, 0, 0)?,
    };
    for i in 1..=nbp {
        prims.add_vertex_pnt(&point_at(i))?;
    }
    Ok(Handle::new(prims))
}

impl DsgPrsIdenticPresentation {
    /// `Add(prs, drawer, text, PntAttach, PntOffset)`: a leader, a ring marker and the text.
    /// Example: `DsgPrsIdenticPresentation::add(&prs, &drawer, &t, &a, &o)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        attach: &Point3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        add_array(
            &current_line_group(prs, la)?,
            &segments(2, &[*attach, *offset])?,
        );
        add_ring_point(prs, la, attach)?;
        draw_text(&current_group(prs)?, la, text, offset);
        Ok(())
    }

    /// `Add(prs, drawer, text, FAttach, SAttach, PntOffset)`: the segment and a leader from
    /// the offset point to its projection on the segment line.
    /// Example: `DsgPrsIdenticPresentation::add_segment(&prs, &drawer, &t, &a, &b, &o)?`.
    pub fn add_segment(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        f_attach: &Point3,
        s_attach: &Point3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let v1 = Vector3::between(*f_attach, *s_attach);
        let v2 = Vector3::between(*s_attach, *offset);
        let last = if !v1.is_parallel(v2, angular())? {
            project_on_line(&Line3::new(*f_attach, Direction3::from_vector(v1)?), offset)?
        } else {
            *s_attach
        };
        add_array(
            &group,
            &segments(4, &[*f_attach, *s_attach, *offset, last])?,
        );
        draw_text(&current_group(prs)?, la, text, offset);
        Ok(())
    }

    /// `Add(prs, drawer, text, Ax2, Center, FAttach, SAttach, PntOffset)`: the arc between the
    /// attachments and a leader to the circle when the offset is off it.
    /// Example: `DsgPrsIdenticPresentation::add_arc(&prs, &drawer, &t, &ax, &c, &a, &b, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_arc(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        axis: &Axis2,
        center: &Point3,
        f_attach: &Point3,
        s_attach: &Point3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let mut ax = *axis;
        ax.set_location(*center);
        let rad = center.distance(f_attach);
        let cc = Circle3::new(ax, rad)?;
        let p_f = elclib::parameter(&cc, *f_attach)?;
        let (nbp, dteta) = arc_sampling(p_f, elclib::parameter(&cc, *s_attach)?);
        let leader = if (offset.distance(center) - rad).abs() >= confusion() {
            Some((
                *offset,
                elclib::value(elclib::parameter(&cc, *offset)?, &cc),
            ))
        } else {
            None
        };
        add_array(
            &group,
            &arc_with_leader(leader, nbp, |i| {
                elclib::value(p_f + dteta * (i - 1) as f64, &cc)
            })?,
        );
        draw_text(&current_group(prs)?, la, text, offset);
        Ok(())
    }

    /// `Add(..., Ax2, Center, FAttach, SAttach, PntOffset, PntOnCirc)`: as `add_arc` with a
    /// given leader end. Example: `DsgPrsIdenticPresentation::add_arc_point(&prs, &drawer, &t, &ax, &c, &a, &b, &o, &p)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_arc_point(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        axis: &Axis2,
        center: &Point3,
        f_attach: &Point3,
        s_attach: &Point3,
        offset: &Point3,
        on_circ: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let mut ax = *axis;
        ax.set_location(*center);
        let cc = Circle3::new(ax, center.distance(f_attach))?;
        let p_f = elclib::parameter(&cc, *f_attach)?;
        let (nbp, dteta) = arc_sampling(p_f, elclib::parameter(&cc, *s_attach)?);
        let leader = (offset.distance(on_circ) >= confusion()).then_some((*offset, *on_circ));
        add_array(
            &group,
            &arc_with_leader(leader, nbp, |i| {
                elclib::value(p_f + dteta * (i - 1) as f64, &cc)
            })?,
        );
        draw_text(&current_group(prs)?, la, text, offset);
        Ok(())
    }

    /// `Add(..., anEllipse, FAttach, SAttach, PntOffset, PntOnElli)`: the ellipse arc variant.
    /// Example: `DsgPrsIdenticPresentation::add_ellipse(&prs, &drawer, &t, &e, &a, &b, &o, &p)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_ellipse(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        ellipse: &Ellipse3,
        f_attach: &Point3,
        s_attach: &Point3,
        offset: &Point3,
        on_elli: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let p_f = elclib::parameter(ellipse, *f_attach)?;
        let (nbp, dteta) = arc_sampling(p_f, elclib::parameter(ellipse, *s_attach)?);
        let leader = (!on_elli.is_equal(offset, confusion())).then_some((*offset, *on_elli));
        add_array(
            &group,
            &arc_with_leader(leader, nbp, |i| {
                elclib::value(p_f + dteta * (i - 1) as f64, ellipse)
            })?,
        );
        draw_text(&current_group(prs)?, la, text, offset);
        Ok(())
    }
}

/// `DsgPrs_MidPointPresentation`.
pub struct DsgPrsMidPointPresentation;

// The small circle of the mid point mark and the " (+)" label, then the leader end.
fn mid_point_mark(
    group: &Handle<dyn Graphic3dGroup>,
    la: &Prs3dDimensionAspect,
    circle_m: &Circle3,
    mid: &Point3,
    position: &Point3,
    rad: f64,
) -> Result<(), Failure> {
    let nbp = 100;
    let dteta = (2.0 * PI) / (nbp - 1) as f64;
    let prims = Graphic3dArrayOfPolylines::new(nbp + 2, 2, 0, 0)?;
    prims.add_bound(nbp)?;
    for i in 1..=nbp {
        prims.add_vertex_pnt(&elclib::value(dteta * (i - 1) as f64, circle_m))?;
    }
    prims.add_bound(2)?;
    let start = if position.is_equal(mid, rad) {
        *mid
    } else {
        elclib::value(elclib::parameter(circle_m, *position)?, circle_m)
    };
    prims.add_vertex_pnt(&start)?;
    prims.add_vertex_pnt(position)?;
    add_array(group, &Handle::new(prims));
    draw_text(group, la, &ExtendedString::new(" (+)"), position);
    Ok(())
}

// The segment from the mark circle to the attach point.
fn mid_point_leader(
    group: &Handle<dyn Graphic3dGroup>,
    circle_m: &Circle3,
    attach: &Point3,
) -> Result<(), Failure> {
    let start = elclib::value(elclib::parameter(circle_m, *attach)?, circle_m);
    add_array(group, &segments(2, &[start, *attach])?);
    Ok(())
}

fn mark_circle(position: Axis2, mid: &Point3, rad: f64) -> Result<Circle3, Failure> {
    let mut ax = position;
    ax.set_location(*mid);
    Circle3::new(ax, rad)
}

fn mark_radius(attach: &Point3, mid: &Point3, point1: &Point3, point2: &Point3) -> f64 {
    let rad = attach.distance(mid) / 20.0;
    if rad <= confusion() {
        point1.distance(point2) / 20.0
    } else {
        rad
    }
}

impl DsgPrsMidPointPresentation {
    /// `Add(prs, drawer, Axe, MidPoint, Position, AttachPoint, first)`.
    /// Example: `DsgPrsMidPointPresentation::add(&prs, &drawer, &ax, &m, &p, &a, true)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        axis: &Axis2,
        mid: &Point3,
        position: &Point3,
        attach: &Point3,
        first: bool,
    ) -> Result<(), Failure> {
        let rad = attach.distance(mid) / 20.0;
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let circle_m = mark_circle(*axis, mid, rad)?;
        if first {
            mid_point_mark(&new_line_group(prs, la)?, la, &circle_m, mid, position, rad)?;
        }
        if !attach.is_equal(mid, confusion()) {
            if !first {
                new_line_group(prs, la)?;
            }
            mid_point_leader(&current_group(prs)?, &circle_m, attach)?;
        }
        Ok(())
    }

    /// `Add(prs, drawer, Axe, MidPoint, Position, AttachPoint, Point1, Point2, first)`: with the
    /// segment between the two points. Example: `DsgPrsMidPointPresentation::add_segment(&prs, &drawer, &ax, &m, &p, &a, &p1, &p2, true)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_segment(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        axis: &Axis2,
        mid: &Point3,
        position: &Point3,
        attach: &Point3,
        point1: &Point3,
        point2: &Point3,
        first: bool,
    ) -> Result<(), Failure> {
        let rad = mark_radius(attach, mid, point1, point2);
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let circle_m = mark_circle(*axis, mid, rad)?;
        let group = new_line_group(prs, la)?;
        add_array(&group, &segments(2, &[*point1, *point2])?);
        Self::finish(&group, la, &circle_m, mid, position, attach, rad, first)
    }

    /// `Add(prs, drawer, aCircle, MidPoint, Position, AttachPoint, Point1, Point2, first)`: with
    /// the circle arc between the points. Example: `DsgPrsMidPointPresentation::add_circle(&prs, &drawer, &c, &m, &p, &a, &p1, &p2, true)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_circle(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        circle: &Circle3,
        mid: &Point3,
        position: &Point3,
        attach: &Point3,
        point1: &Point3,
        point2: &Point3,
        first: bool,
    ) -> Result<(), Failure> {
        let rad = mark_radius(attach, mid, point1, point2);
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let circle_m = mark_circle(circle.position(), mid, rad)?;
        let group = new_line_group(prs, la)?;
        let pf = elclib::parameter(circle, *point1)?;
        let (nbp, dteta) = arc_sampling(pf, elclib::parameter(circle, *point2)?);
        add_array(
            &group,
            &arc_with_leader(None, nbp, |i| {
                elclib::value(pf + dteta * (i - 1) as f64, circle)
            })?,
        );
        Self::finish(&group, la, &circle_m, mid, position, attach, rad, first)
    }

    /// `Add(prs, drawer, anElips, MidPoint, Position, AttachPoint, Point1, Point2, first)`: with
    /// the ellipse arc between the points. Example: `DsgPrsMidPointPresentation::add_ellipse(&prs, &drawer, &e, &m, &p, &a, &p1, &p2, true)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_ellipse(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        ellipse: &Ellipse3,
        mid: &Point3,
        position: &Point3,
        attach: &Point3,
        point1: &Point3,
        point2: &Point3,
        first: bool,
    ) -> Result<(), Failure> {
        let rad = mark_radius(attach, mid, point1, point2);
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let circle_m = mark_circle(ellipse.position(), mid, rad)?;
        let group = new_line_group(prs, la)?;
        let pf = elclib::parameter(ellipse, *point1)?;
        let (nbp, dteta) = arc_sampling(pf, elclib::parameter(ellipse, *point2)?);
        add_array(
            &group,
            &arc_with_leader(None, nbp, |i| {
                elclib::value(pf + dteta * (i - 1) as f64, ellipse)
            })?,
        );
        Self::finish(&group, la, &circle_m, mid, position, attach, rad, first)
    }

    #[allow(clippy::too_many_arguments)] // The shared tail of the three native overloads.
    fn finish(
        group: &Handle<dyn Graphic3dGroup>,
        la: &Prs3dDimensionAspect,
        circle_m: &Circle3,
        mid: &Point3,
        position: &Point3,
        attach: &Point3,
        rad: f64,
        first: bool,
    ) -> Result<(), Failure> {
        if first {
            mid_point_mark(group, la, circle_m, mid, position, rad)?;
        }
        if !attach.is_equal(mid, confusion()) {
            mid_point_leader(group, circle_m, attach)?;
        }
        Ok(())
    }
}

/// `DsgPrs_EqualDistancePresentation`.
pub struct DsgPrsEqualDistancePresentation;

impl DsgPrsEqualDistancePresentation {
    /// `Add(prs, drawer, Point1..Point4, Plane)`: the segment between both pair middles with
    /// point ends and an "==" mark beside it.
    /// Example: `DsgPrsEqualDistancePresentation::add(&prs, &drawer, &p1, &p2, &p3, &p4, &plane)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        point1: &Point3,
        point2: &Point3,
        point3: &Point3,
        point4: &Point3,
        plane: &Handle<GeomPlane>,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let middle12 = Point3::from((point1.xyz() + point2.xyz()) * 0.5);
        let middle34 = Point3::from((point3.xyz() + point4.xyz()) * 0.5);
        add_array(&group, &segments(2, &[middle12, middle34])?);
        let z = Direction3::from(StandardDirection3::Z);
        let ends = DsgPrsSymbolEnds {
            first: middle12,
            last: middle34,
            first_direction: z,
            last_direction: z,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, DsgPrsArrowSide::BothPt, true)?;
        let plane = plane
            .get()
            .ok_or_else(|| null_object("DsgPrs_EqualDistancePresentation: the plane is null"))?
            .pln();
        let middle = Point3::from((middle12.xyz() + middle34.xyz()) * 0.5);
        let text_pos = equal_distance_text_position(
            &middle, &middle12, &middle34, point1, point2, point3, &plane,
        )?;
        draw_text(
            &current_group(prs)?,
            la,
            &ExtendedString::new("=="),
            &text_pos,
        );
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)] // The native local variables of the text placement.
fn equal_distance_text_position(
    middle: &Point3,
    middle12: &Point3,
    middle34: &Point3,
    point1: &Point3,
    point2: &Point3,
    point3: &Point3,
    plane: &rustycad_tkmath::gp::Plane3,
) -> Result<Point3, Failure> {
    let dist = middle12.distance(middle34);
    if dist > confusion() {
        let mut small_dist = dist * 0.05;
        if small_dist <= confusion() {
            small_dist = dist;
        }
        let line_dir = make_dir(middle12, middle34)?;
        let ort_dir = plane.axis().direction().crossed(line_dir)?;
        return Ok(middle.translated(Vector3::from(ort_dir) * small_dist));
    }
    let vec1 = Vector3::between(*middle, *point1);
    let ort_vec = if vec1.square_magnitude() > confusion() * confusion() {
        let angle = Vector3::between(*middle, *point1).angle(Vector3::between(*middle, *point3))?;
        let mid_pnt = point1.rotated(plane.axis(), angle * 0.5)?;
        let ort_dir = make_dir(middle, &mid_pnt)?;
        let _line_dir = ort_dir.crossed(plane.axis().direction())?;
        let distance = point1.distance(point2);
        let mut small_dist = distance * 0.05;
        if small_dist <= confusion() {
            small_dist = distance;
        }
        Vector3::from(ort_dir) * small_dist
    } else {
        Vector3::from(plane.x_axis().direction()) * 5.0
    };
    Ok(middle.translated(ort_vec))
}

impl DsgPrsEqualDistancePresentation {
    /// `AddInterval`: the interval between two points along a direction at the position;
    /// returns the two extreme points. Example: `DsgPrsEqualDistancePresentation::add_interval(&prs, &drawer, &a, &b, &d, &p, DsgPrsArrowSide::BothAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_interval(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        point1: &Point3,
        point2: &Point3,
        direction: &Direction3,
        position: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(Point3, Point3), Failure> {
        let mut extremes = (Point3::default(), Point3::default());
        Self::add_interval_into(
            prs,
            drawer,
            point1,
            point2,
            direction,
            position,
            side,
            &mut extremes,
        )?;
        Ok(extremes)
    }

    /// `AddInterval` with the native output arguments: the two extreme points are written
    /// before the arrows, so they are kept when the arrow direction raises.
    /// Example: `DsgPrsEqualDistancePresentation::add_interval_into(&prs, &drawer, &a, &b, &d, &p, side, &mut ends)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_interval_into(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        point1: &Point3,
        point2: &Point3,
        direction: &Direction3,
        position: &Point3,
        side: DsgPrsArrowSide,
        extremes: &mut (Point3, Point3),
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let proj1 = project_on_line(&Line3::new(*point1, *direction), position)?;
        extremes.0 = proj1;
        let proj2 = project_on_line(&Line3::new(*point2, *direction), position)?;
        extremes.1 = proj2;
        add_array(&group, &polyline(&[*point1, proj1, proj2, *point2])?);
        let dir = Direction3::from_xyz(proj2.xyz() - proj1.xyz())?;
        let ends = DsgPrsSymbolEnds {
            first: proj1,
            last: proj2,
            first_direction: dir.reversed(),
            last_direction: dir,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }

    /// `AddIntervalBetweenTwoArcs`: the segment joining two arc ends and both arcs.
    /// Example: `DsgPrsEqualDistancePresentation::add_interval_between_two_arcs(&prs, &drawer, &c1, &c2, &p1, &p2, &p3, &p4, DsgPrsArrowSide::BothAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_interval_between_two_arcs(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        circ1: &Circle3,
        circ2: &Circle3,
        point1: &Point3,
        point2: &Point3,
        point3: &Point3,
        point4: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let (par11, par12) = arc_parameters(circ1, point1, point2)?;
        let (par21, par22) = arc_parameters(circ2, point3, point4)?;
        add_array(&group, &segments(2, &[*point2, *point4])?);
        add_two_arc_polyline(&group, circ1, par11, par12, point2)?;
        add_two_arc_polyline(&group, circ2, par21, par22, point4)?;
        let dir_of_arrow = if point4.distance(point2) > confusion() {
            Direction3::from_xyz(point4.xyz() - point2.xyz())?
        } else {
            let center = circ1.location();
            if point4.distance(&center) < confusion() {
                return Ok(());
            }
            Direction3::from_xyz(point4.xyz() - center.xyz())?
        };
        let ends = DsgPrsSymbolEnds {
            first: *point2,
            last: *point4,
            first_direction: dir_of_arrow.reversed(),
            last_direction: dir_of_arrow,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }
}

fn arc_parameters(
    circle: &Circle3,
    first: &Point3,
    second: &Point3,
) -> Result<(f64, f64), Failure> {
    if circle.radius() > confusion() {
        Ok((
            elclib::parameter(circle, *first)?,
            elclib::parameter(circle, *second)?,
        ))
    } else {
        Ok((PI, PI))
    }
}

fn add_two_arc_polyline(
    group: &Handle<dyn Graphic3dGroup>,
    circle: &Circle3,
    par1: f64,
    mut par2: f64,
    end: &Point3,
) -> Result<(), Failure> {
    if par2 < par1 {
        par2 += 2.0 * PI;
    }
    if (par2 - par1).abs() <= confusion() {
        return Ok(());
    }
    let node_nb = ((par2 - par1).abs() * 50.0 / PI + 0.5).max(4.0) as i32;
    let delta = (par2 - par1) / node_nb as f64;
    let mut current = par1;
    let mut points = Vec::with_capacity(node_nb as usize + 1);
    for _ in 1..=node_nb {
        points.push(elclib::value(current, circle));
        current += delta;
    }
    points.push(*end);
    let prims = Graphic3dArrayOfPolylines::new(node_nb + 1, 0, 0, 0)?;
    for point in &points {
        prims.add_vertex_pnt(point)?;
    }
    add_array(group, &Handle::new(prims));
    Ok(())
}
