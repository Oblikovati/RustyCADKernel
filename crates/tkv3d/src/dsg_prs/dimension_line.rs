//! The length, parallelism and offset presentations: a dimension line between two
//! projections of the attachment points, extended to the offset point.
use super::package::{DsgPrs, DsgPrsCurvilinearLength, DsgPrsPlanarLength};
use super::support::*;
use super::symbol::{DsgPrsArrowSide, DsgPrsSymbolEnds, dsg_prs_compute_symbol};
use crate::prs3d::{Prs3dDimensionAspect, Prs3dDrawer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point3, Vector3};
use rustycad_tkservice::aspect::{AspectTypeOfLine, AspectTypeOfMarker};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfSegments, Graphic3dAspectLine3d, Graphic3dAspectMarker3d,
    Graphic3dAspectsMembers as _,
};
use std::f64::consts::PI;

/// The extent of a dimension line `L3` over two points and the offset point.
pub(super) struct DsgPrsLineExtent {
    pub(super) point_min: Point3,
    pub(super) point_max: Point3,
    pub(super) offset_projection: Point3,
    pub(super) dist: f64,
    pub(super) outside: bool,
}

/// The parameter range of `p1`, `p2` and `offset` on `line` (the `parmin`/`parmax` block).
pub(super) fn line_extent(
    line: &Line3,
    p1: &Point3,
    p2: &Point3,
    offset: &Point3,
) -> Result<DsgPrsLineExtent, Failure> {
    let mut parmin = elclib::parameter(line, *p1)?;
    let mut parmax = parmin;
    let mut parcur = elclib::parameter(line, *p2)?;
    let dist = (parmin - parcur).abs();
    if parcur < parmin {
        parmin = parcur;
    }
    if parcur > parmax {
        parmax = parcur;
    }
    parcur = elclib::parameter(line, *offset)?;
    let offset_projection = elclib::value(parcur, line);
    let mut outside = false;
    if parcur < parmin {
        parmin = parcur;
        outside = true;
    }
    if parcur > parmax {
        parmax = parcur;
        outside = true;
    }
    Ok(DsgPrsLineExtent {
        point_min: elclib::value(parmin, line),
        point_max: elclib::value(parmax, line),
        offset_projection,
        dist,
        outside,
    })
}

/// The first arrow direction: inward unless outside or too short for two arrows.
pub(super) fn arrow_direction(
    line: &Line3,
    extent: &DsgPrsLineExtent,
    aspect: &Prs3dDimensionAspect,
) -> Direction3 {
    let length = arrow_length(aspect);
    let outside = extent.outside || extent.dist < (length + length);
    let direction = line.direction().reversed();
    if outside {
        direction.reversed()
    } else {
        direction
    }
}

/// The projections of two attachment points on their lines through the offset point.
fn projections(
    a1: &Point3,
    a2: &Point3,
    d1: &Direction3,
    d2: &Direction3,
    offset: &Point3,
) -> Result<(Point3, Point3), Failure> {
    let proj1 = project_on_line(&Line3::new(*a1, *d1), offset)?;
    let proj2 = project_on_line(&Line3::new(*a2, *d2), offset)?;
    Ok((proj1, proj2))
}

/// `L3`: the line through the projections, or along the direction when they coincide.
fn projections_line(
    proj1: &Point3,
    proj2: &Point3,
    direction: &Direction3,
) -> Result<Line3, Failure> {
    if proj1.is_equal(proj2, confusion()) {
        Ok(Line3::new(*proj1, *direction))
    } else {
        make_lin(proj1, proj2)
    }
}

/// The dimension segments: the line, then both attachment-to-projection segments.
fn dimension_segments(
    extent: &DsgPrsLineExtent,
    attach: [(&Point3, &Point3); 2],
) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    let (a1, p1) = attach[0];
    let (a2, p2) = attach[1];
    segments(6, &[extent.point_min, extent.point_max, *a1, *p1, *a2, *p2])
}

fn aspect_ref(aspect: &Handle<Prs3dDimensionAspect>) -> Result<&Prs3dDimensionAspect, Failure> {
    aspect
        .get()
        .ok_or_else(|| null_object("DsgPrs: the dimension aspect is null"))
}

/// `DsgPrs_LengthPresentation`.
pub struct DsgPrsLengthPresentation;

impl DsgPrsLengthPresentation {
    /// `Add(prs, drawer, text, A1, A2, dir, offset)`: arrows in their own groups, then the
    /// text and the segments. Example: `DsgPrsLengthPresentation::add(&prs, &drawer, &t, &a1, &a2, &d, &o)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        a1: &Point3,
        a2: &Point3,
        direction: &Direction3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        current_line_group(prs, la)?;
        let (proj1, proj2) = projections(a1, a2, direction, direction, offset)?;
        let l3 = projections_line(&proj1, &proj2, direction)?;
        let extent = line_extent(&l3, &proj1, &proj2, offset)?;
        let arrdir = arrow_direction(&l3, &extent, la);
        draw_arrow(&current_group(prs)?, la, &proj1, &arrdir)?;
        draw_arrow(&new_line_group(prs, la)?, la, &proj2, &arrdir.reversed())?;
        let group = new_group(prs)?;
        draw_text(&group, la, text, &extent.offset_projection);
        set_line_aspect(&group, la);
        add_array(
            &group,
            &dimension_segments(&extent, [(a1, &proj1), (a2, &proj2)])?,
        );
        Ok(())
    }

    /// `Add(prs, drawer, text, A1, A2, dir, offset, ArrowSide)`: one group with the
    /// segments and text, and computed end symbols.
    /// Example: `DsgPrsLengthPresentation::add_with_arrow(&prs, &drawer, &t, &a1, &a2, &d, &o, DsgPrsArrowSide::BothAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_with_arrow(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        a1: &Point3,
        a2: &Point3,
        direction: &Direction3,
        offset: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let (proj1, proj2) = projections(a1, a2, direction, direction, offset)?;
        let l3 = projections_line(&proj1, &proj2, direction)?;
        let extent = line_extent(&l3, &proj1, &proj2, offset)?;
        let arrdir = arrow_direction(&l3, &extent, la);
        add_array(
            &group,
            &dimension_segments(&extent, [(a1, &proj1), (a2, &proj2)])?,
        );
        draw_text(&current_group(prs)?, la, text, &extent.offset_projection);
        let ends = DsgPrsSymbolEnds {
            first: proj1,
            last: proj2,
            first_direction: arrdir,
            last_direction: arrdir.reversed(),
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }

    /// `Add(prs, drawer, text, A1, A2, PlaneOfFaces, dir, offset, ArrowSide)`: the planar
    /// faces layout of `DsgPrs::ComputePlanarFacesLengthPresentation`.
    /// Example: `DsgPrsLengthPresentation::add_planar(&prs, &drawer, &t, &a1, &a2, &plane, &d, &o, DsgPrsArrowSide::BothAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_planar(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        a1: &Point3,
        a2: &Point3,
        plane: &Plane3,
        direction: &Direction3,
        offset: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        current_line_group(prs, la)?;
        let length = arrow_length(la);
        let mut ends = DsgPrsPlanarLength::default();
        DsgPrs::compute_planar_faces_length_presentation(
            length, length, a1, a2, direction, offset, plane, &mut ends,
        )?;
        let length_line = Line3::new(*offset, ends.dir_of_arrow1);
        let par1 = elclib::parameter(&length_line, ends.end_of_arrow1)?;
        let par2 = elclib::parameter(&length_line, ends.end_of_arrow2)?;
        let (first, last) = if (par1 > 0.0 && par2 > 0.0) || (par1 < 0.0 && par2 < 0.0) {
            (
                *offset,
                if par1.abs() > par2.abs() {
                    ends.end_of_arrow1
                } else {
                    ends.end_of_arrow2
                },
            )
        } else {
            (ends.end_of_arrow1, ends.end_of_arrow2)
        };
        let prims = segments(6, &[first, last])?;
        let symbol = DsgPrsSymbolEnds {
            first: ends.end_of_arrow1,
            last: ends.end_of_arrow2,
            first_direction: ends.dir_of_arrow1,
            last_direction: ends.dir_of_arrow1.reversed(),
        };
        dsg_prs_compute_symbol(prs, &aspect, &symbol, side, true)?;
        let group = current_group(prs)?;
        draw_text(&group, la, text, offset);
        for point in [a1, &ends.end_of_arrow1, a2, &ends.end_of_arrow2] {
            prims.add_vertex_pnt(point)?;
        }
        add_array(&group, &prims);
        Ok(())
    }

    /// `Add(prs, drawer, text, SecondSurf, A1, A2, dir, offset, ArrowSide)`: the curvilinear
    /// faces layout with the iso-curve arcs on the second surface.
    /// Example: `DsgPrsLengthPresentation::add_curvilinear(&prs, &drawer, &t, &surface, &a1, &a2, &d, &o, DsgPrsArrowSide::BothAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_curvilinear(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        second_surf: &Handle<dyn GeomSurface>,
        a1: &Point3,
        a2: &Point3,
        direction: &Direction3,
        offset: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let length = arrow_length(la);
        let mut layout = DsgPrsCurvilinearLength::default();
        DsgPrs::compute_curvilinear_faces_length_presentation(
            length,
            length,
            second_surf,
            a1,
            a2,
            direction,
            &mut layout,
        )?;
        let length_line = Line3::new(*offset, layout.dir_of_arrow1);
        let par1 = elclib::parameter(&length_line, *a1)?;
        let par2 = elclib::parameter(&length_line, layout.end_of_arrow2)?;
        let (first, last) = if (par1 > 0.0 && par2 > 0.0) || (par1 < 0.0 && par2 < 0.0) {
            (
                *offset,
                if par1.abs() > par2.abs() {
                    *a1
                } else {
                    layout.end_of_arrow2
                },
            )
        } else {
            (*a1, layout.end_of_arrow2)
        };
        add_array(&group, &segments(2, &[first, last])?);
        let symbol = DsgPrsSymbolEnds {
            first: *a1,
            last: layout.end_of_arrow2,
            first_direction: layout.dir_of_arrow1,
            last_direction: layout.dir_of_arrow1.reversed(),
        };
        dsg_prs_compute_symbol(prs, &aspect, &symbol, side, true)?;
        let group = current_group(prs)?;
        draw_text(&group, la, text, offset);
        iso_arc(&group, &layout.v_curve, layout.first_u, layout.delta_u)?;
        iso_arc(&group, &layout.u_curve, layout.first_v, layout.delta_v)
    }

    /// `Add(prs, drawer, Pt1, Pt2, ArrowSide)`: a segment with arrows along it.
    /// Example: `DsgPrsLengthPresentation::add_segment(&prs, &drawer, &p1, &p2, DsgPrsArrowSide::BothAr)?`.
    pub fn add_segment(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        pt1: &Point3,
        pt2: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        add_array(&current_group(prs)?, &segments(2, &[*pt1, *pt2])?);
        if !matches!(
            side,
            DsgPrsArrowSide::LastAr | DsgPrsArrowSide::FirstAr | DsgPrsArrowSide::BothAr
        ) {
            return Ok(());
        }
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let arrow = |point: &Point3, direction: Direction3| {
            draw_arrow(&current_group(prs)?, la, point, &direction)
        };
        match side {
            DsgPrsArrowSide::LastAr => arrow(pt2, direction_between(pt1, pt2)?),
            DsgPrsArrowSide::FirstAr => arrow(pt1, direction_between(pt2, pt1)?),
            DsgPrsArrowSide::BothAr => {
                let v = Vector3::between(*pt1, *pt2);
                arrow(pt2, Direction3::from_vector(v)?)?;
                arrow(pt1, Direction3::from_vector(v.reversed())?)
            }
            _ => Ok(()),
        }
    }
}

// A polyline along an iso curve when the parameter span is a usable angle.
fn iso_arc(
    group: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dGroup>,
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    mut first: f64,
    delta_param: f64,
) -> Result<(), Failure> {
    let alpha = delta_param.abs();
    if !(alpha > angular() && alpha < rustycad_tkernel::precision::infinite()) {
        return Ok(());
    }
    let Some(curve) = curve.get() else {
        return Err(null_object(
            "DsgPrs_LengthPresentation: the iso curve is null",
        ));
    };
    let node_number = 4.max((50.0 * alpha / PI) as i32);
    let delta = delta_param / (node_number - 1) as f64;
    let mut points = Vec::with_capacity(node_number as usize);
    for _ in 1..=node_number {
        points.push(curve.value(first)?);
        first += delta;
    }
    add_array(group, &polyline(&points)?);
    Ok(())
}

/// `DsgPrs_ParalPresentation`.
pub struct DsgPrsParalPresentation;

impl DsgPrsParalPresentation {
    /// `Add(prs, drawer, text, A1, A2, dir, offset)`.
    /// Example: `DsgPrsParalPresentation::add(&prs, &drawer, &t, &a1, &a2, &d, &o)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        a1: &Point3,
        a2: &Point3,
        direction: &Direction3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        current_line_group(prs, la)?;
        let (proj1, proj2) = projections(a1, a2, direction, direction, offset)?;
        let l3 = make_lin(&proj1, &proj2)?;
        let extent = line_extent(&l3, &proj1, &proj2, offset)?;
        let arrdir = arrow_direction(&l3, &extent, la);
        draw_arrow(&new_line_group(prs, la)?, la, &proj1, &arrdir)?;
        draw_arrow(&new_line_group(prs, la)?, la, &proj2, &arrdir.reversed())?;
        let group = new_group(prs)?;
        draw_text(&group, la, text, &extent.offset_projection);
        set_line_aspect(&group, la);
        add_array(
            &group,
            &dimension_segments(&extent, [(a1, &proj1), (a2, &proj2)])?,
        );
        Ok(())
    }

    /// `Add(prs, drawer, text, A1, A2, dir, offset, ArrowSide)`.
    /// Example: `DsgPrsParalPresentation::add_with_arrow(&prs, &drawer, &t, &a1, &a2, &d, &o, DsgPrsArrowSide::BothAr)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_with_arrow(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        a1: &Point3,
        a2: &Point3,
        direction: &Direction3,
        offset: &Point3,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let (proj1, proj2) = projections(a1, a2, direction, direction, offset)?;
        let l3 = make_lin(&proj1, &proj2)?;
        let extent = line_extent(&l3, &proj1, &proj2, offset)?;
        let arrdir = arrow_direction(&l3, &extent, la);
        add_array(
            &group,
            &dimension_segments(&extent, [(a1, &proj1), (a2, &proj2)])?,
        );
        draw_text(&current_group(prs)?, la, text, &extent.offset_projection);
        let ends = DsgPrsSymbolEnds {
            first: proj1,
            last: proj2,
            first_direction: arrdir,
            last_direction: arrdir.reversed(),
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }
}

/// `DsgPrs_OffsetPresentation`.
pub struct DsgPrsOffsetPresentation;

impl DsgPrsOffsetPresentation {
    /// `Add`: an offset dimension, or two opposite arrows at the offset projection when the
    /// projections coincide (null dimension).
    /// Example: `DsgPrsOffsetPresentation::add(&prs, &drawer, &t, &a1, &a2, &d1, &d2, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        a1: &Point3,
        a2: &Point3,
        direction: &Direction3,
        direction2: &Direction3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        current_line_group(prs, la)?;
        let (proj1, proj2) = projections(a1, a2, direction, direction2, offset)?;
        let null_dimension = proj1.is_equal(&proj2, confusion() * 100.0);
        let (l3, l4) = if !null_dimension {
            (make_lin(&proj1, &proj2)?, Line3::default())
        } else {
            let d4 = Direction3::from_vector(Vector3::between(proj1, *offset))?;
            (Line3::new(proj1, *direction), Line3::new(proj1, d4))
        };
        let extent = line_extent(&l3, &proj1, &proj2, offset)?;
        let prims = segments(6, &[extent.point_min, extent.point_max])?;
        let group = new_line_group(prs, la)?;
        if null_dimension {
            draw_arrow(&group, la, &extent.offset_projection, &l4.direction())?;
            draw_arrow(
                &group,
                la,
                &extent.offset_projection,
                &l4.direction().reversed(),
            )?;
        } else {
            let arrdir = arrow_direction(&l3, &extent, la);
            draw_arrow(&group, la, &proj1, &arrdir)?;
            add_ring_point(prs, la, &proj2)?;
            draw_text(&new_group(prs)?, la, text, &extent.offset_projection);
        }
        let group = new_line_group(prs, la)?;
        for point in [a1, &proj1, a2, &proj2] {
            prims.add_vertex_pnt(point)?;
        }
        add_array(&group, &prims);
        Ok(())
    }

    /// `AddAxes`: dot-dash axis lines (the second 4 wide) and ring markers at the second
    /// projection. Example: `DsgPrsOffsetPresentation::add_axes(&prs, &drawer, &t, &a1, &a2, &d1, &d2, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_axes(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        _text: &ExtendedString,
        a1: &Point3,
        a2: &Point3,
        direction: &Direction3,
        _direction2: &Direction3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let (proj1, proj2) = projections(a1, a2, direction, direction, offset)?;
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let line = la
            .line_aspect()
            .get()
            .map(|line| line.aspect())
            .ok_or_else(|| null_object("DsgPrs: null line aspect"))?;
        let line = line
            .get()
            .ok_or_else(|| null_object("DsgPrs: null line aspect"))?;
        let (color, kind, width) = (
            line.interior_color(),
            line.line_kind(),
            f64::from(line.width()),
        );
        let axis_aspect = Graphic3dAspectLine3d::new(color, kind, width)?;
        axis_aspect.set_line_kind(AspectTypeOfLine::Dotdash);
        let group = new_group(prs)?;
        set_aspect(&group, &Handle::new(axis_aspect).cast());
        add_array(&group, &segments(2, &[*a1, proj1])?);
        let group = new_group(prs)?;
        let axis2_aspect = Graphic3dAspectLine3d::new(color, kind, width)?;
        axis2_aspect.set_line_kind(AspectTypeOfLine::Dotdash);
        axis2_aspect.set_width(4.0)?;
        set_aspect(&group, &Handle::new(axis2_aspect).cast());
        add_array(&group, &segments(2, &[*a2, proj2])?);
        let marker_points = points(&[proj2])?;
        for scale in [4.0, 2.0] {
            let group = new_group(prs)?;
            let marker = Graphic3dAspectMarker3d::default();
            marker.set_marker_kind(AspectTypeOfMarker::O);
            marker.set_scale(scale)?;
            marker.set_interior_color(color);
            set_aspect(&group, &Handle::new(marker).cast());
            add_array(&group, &marker_points);
        }
        Ok(())
    }
}

fn set_aspect(
    group: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dGroup>,
    aspect: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dAspectsSource>,
) {
    if let Some(group) = group.get() {
        group.set_primitives_aspect(aspect);
    }
}
