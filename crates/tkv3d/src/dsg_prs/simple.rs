//! The short DsgPrs builders: chamfer, concentric, datum, equal radius, fix, perpendicular
//! and the tangent, symbol and shape direction helpers' shared steps.
use super::DsgPrsXYZAxisPresentation;
use super::support::*;
use super::symbol::{DsgPrsArrowSide, DsgPrsSymbolEnds, dsg_prs_compute_symbol};
use crate::prs3d::{Prs3dDatumAttribute, Prs3dDatumAxes, Prs3dDatumParts, Prs3dDrawer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Vector3};
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::{Graphic3dArrayOfPolylines, Graphic3dArrayOfSegments};
use std::f64::consts::PI;

/// `DsgPrs_Chamf2dPresentation`.
pub struct DsgPrsChamf2dPresentation;

impl DsgPrsChamf2dPresentation {
    /// `Add(prs, drawer, PntAttach, PntEnd, Text)`: the chamfer line, an arrow at the attach
    /// point and the text. Example: `DsgPrsChamf2dPresentation::add(&prs, &drawer, &a, &b, &text)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        attach: &Point3,
        end: &Point3,
        text: &ExtendedString,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: null dimension aspect"))?;
        let group = current_line_group(prs, la)?;
        add_array(&group, &segments(2, &[*attach, *end])?);
        let arrow_dir = Direction3::from_xyz(attach.xyz() - end.xyz())?;
        draw_arrow(&current_group(prs)?, la, attach, &arrow_dir)?;
        draw_text(&current_group(prs)?, la, text, end);
        Ok(())
    }

    /// `Add(prs, drawer, PntAttach, PntEnd, Text, ArrowSide)`: with a computed end symbol.
    /// Example: `DsgPrsChamf2dPresentation::add_with_arrow(&prs, &drawer, &a, &b, &text, DsgPrsArrowSide::LastAr)?`.
    pub fn add_with_arrow(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        attach: &Point3,
        end: &Point3,
        text: &ExtendedString,
        side: DsgPrsArrowSide,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: null dimension aspect"))?;
        let group = current_line_group(prs, la)?;
        add_array(&group, &segments(2, &[*attach, *end])?);
        draw_text(&current_group(prs)?, la, text, end);
        let arrow_dir = Direction3::from_xyz(attach.xyz() - end.xyz())?;
        let ends = DsgPrsSymbolEnds {
            first: *end,
            last: *attach,
            first_direction: arrow_dir.reversed(),
            last_direction: arrow_dir,
        };
        dsg_prs_compute_symbol(prs, &aspect, &ends, side, true)
    }
}

/// `DsgPrs_ConcentricPresentation`.
pub struct DsgPrsConcentricPresentation;

impl DsgPrsConcentricPresentation {
    /// `Add`: two concentric circles (radius and half radius) and a cross through the center.
    /// Example: `DsgPrsConcentricPresentation::add(&prs, &drawer, &center, 5.0, &normal, &point)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        center: &Point3,
        radius: f64,
        normal: &Direction3,
        point: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: null dimension aspect"))?;
        let mut circ = Circle3::new(Axis2::new(*center, *normal)?, radius)?;
        let nbp = 50;
        let dteta = (2.0 * PI) / nbp as f64;
        let group = current_line_group(prs, la)?;
        let prims = Graphic3dArrayOfPolylines::new(2 * nbp + 6, 4, 0, 0)?;
        add_circle_bound(&prims, &circ, nbp, dteta)?;
        circ.set_radius(0.5 * radius)?;
        add_circle_bound(&prims, &circ, nbp, dteta)?;
        let vecnorm = Direction3::from_xyz(point.xyz() - center.xyz())?;
        let mut vec = Vector3::from(vecnorm) * radius;
        add_bound_pair(&prims, center.translated(vec), center.translated(-vec))?;
        vec.cross(Vector3::from(*normal));
        let vecnorm = Direction3::new(vec.x(), vec.y(), vec.z())?;
        let vec = Vector3::from(vecnorm) * radius;
        add_bound_pair(&prims, center.translated(vec), center.translated(-vec))?;
        add_array(&group, &Handle::new(prims));
        Ok(())
    }
}

fn add_circle_bound(
    prims: &Graphic3dArrayOfPolylines,
    circ: &Circle3,
    nbp: i32,
    dteta: f64,
) -> Result<(), Failure> {
    let pt1 = elclib::value(0.0, circ);
    prims.add_bound(nbp + 1)?;
    prims.add_vertex_pnt(&pt1)?;
    let mut ucur = dteta;
    for _ in 2..=nbp {
        prims.add_vertex_pnt(&elclib::value(ucur, circ))?;
        ucur += dteta;
    }
    prims.add_vertex_pnt(&pt1)?;
    Ok(())
}

fn add_bound_pair(
    prims: &Graphic3dArrayOfPolylines,
    p1: Point3,
    p2: Point3,
) -> Result<(), Failure> {
    prims.add_bound(2)?;
    prims.add_vertex_pnt(&p1)?;
    prims.add_vertex_pnt(&p2)?;
    Ok(())
}

/// `DsgPrs_DatumPrs`.
pub struct DsgPrsDatumPrs;

impl DsgPrsDatumPrs {
    /// `Add(prs, datum, drawer)`: the requested datum axes with arrows and labels.
    /// Example: `DsgPrsDatumPrs::add(&prs, &Axis2::default(), &drawer)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        datum: &Axis2,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let Some(drawer_ref) = drawer.get() else {
            return Err(null_object("DsgPrs_DatumPrs: the drawer is null"));
        };
        let datum_aspect = drawer_ref.datum_aspect();
        let Some(aspect) = datum_aspect.get() else {
            return Err(null_object("DsgPrs_DatumPrs: the datum aspect is null"));
        };
        current_group(prs)?;
        let origin = datum.location();
        let axes = aspect.datum_axes();
        let labels = aspect.to_draw_labels();
        let parts = [
            (
                Prs3dDatumAxes::XAxis,
                Prs3dDatumAttribute::XAxisLength,
                Prs3dDatumParts::XAxis,
                datum.x_direction(),
                "X",
            ),
            (
                Prs3dDatumAxes::YAxis,
                Prs3dDatumAttribute::YAxisLength,
                Prs3dDatumParts::YAxis,
                datum.y_direction(),
                "Y",
            ),
            (
                Prs3dDatumAxes::ZAxis,
                Prs3dDatumAttribute::ZAxisLength,
                Prs3dDatumParts::ZAxis,
                datum.direction(),
                "Z",
            ),
        ];
        for (axis, attribute, part, direction, label) in parts {
            if (axes as i32 & axis as i32) == 0 {
                continue;
            }
            let length = aspect.attribute(attribute);
            let end = Point3::from(origin.xyz() + direction.xyz() * length);
            let label = if labels { label } else { "" };
            DsgPrsXYZAxisPresentation::add_with_aspects(
                prs,
                &aspect.line_aspect(part),
                &aspect.arrow_aspect(),
                &drawer_ref.text_aspect(),
                &direction,
                length,
                label,
                &origin,
                &end,
            )?;
        }
        Ok(())
    }
}

/// `DsgPrs_EqualRadiusPresentation`.
pub struct DsgPrsEqualRadiusPresentation;

impl DsgPrsEqualRadiusPresentation {
    /// `Add`: both radii with point/arrow ends and an "==" mark beside the center line.
    /// Example: `DsgPrsEqualRadiusPresentation::add(&prs, &drawer, &c1, &c2, &p1, &p2, &plane)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        first_center: &Point3,
        second_center: &Point3,
        first_point: &Point3,
        second_point: &Point3,
        plane: &Handle<rustycad_tkg3d::geom::GeomPlane>,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: null dimension aspect"))?;
        let group = current_line_group(prs, la)?;
        add_array(
            &group,
            &polyline(&[*first_point, *first_center, *second_center, *second_point])?,
        );
        let first_dir = make_dir(first_center, first_point)?;
        let second_dir = make_dir(second_center, second_point)?;
        for (center, point, dir) in [
            (first_center, first_point, first_dir),
            (second_center, second_point, second_dir),
        ] {
            let ends = DsgPrsSymbolEnds {
                first: *center,
                last: *point,
                first_direction: dir.reversed(),
                last_direction: dir,
            };
            dsg_prs_compute_symbol(prs, &aspect, &ends, DsgPrsArrowSide::FirstPtLastAr, true)?;
        }
        let text_pos = equal_radius_text_position(
            first_center,
            second_center,
            first_point,
            second_point,
            plane,
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

fn equal_radius_text_position(
    first_center: &Point3,
    second_center: &Point3,
    first_point: &Point3,
    second_point: &Point3,
    plane: &Handle<rustycad_tkg3d::geom::GeomPlane>,
) -> Result<Point3, Failure> {
    let middle = Point3::from((first_center.xyz() + second_center.xyz()) * 0.5);
    let dist = first_center.distance(second_center);
    if dist > confusion() {
        let mut small_dist = dist * 0.05;
        if small_dist <= confusion() {
            small_dist = dist;
        }
        let line_dir = make_dir(first_center, second_center)?;
        let plane = plane
            .get()
            .ok_or_else(|| null_object("DsgPrs: the plane is null"))?;
        let ort_dir = plane.pln().axis().direction().crossed(line_dir)?;
        return Ok(middle.translated(Vector3::from(ort_dir) * small_dist));
    }
    let rad = first_center
        .distance(first_point)
        .max(second_center.distance(second_point));
    let mut small_dist = rad * 0.05;
    if small_dist <= confusion() {
        small_dist = rad;
    }
    Ok(first_center.translated(Vector3::new(small_dist, small_dist, small_dist)))
}

/// `DsgPrs_FixPresentation`.
pub struct DsgPrsFixPresentation;

impl DsgPrsFixPresentation {
    /// `Add`: the attach segment, the fix symbol (a base and three hatches) and a ring
    /// marker at the attach point. Example: `DsgPrsFixPresentation::add(&prs, &drawer, &a, &b, &n, 2.0)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        attach: &Point3,
        end: &Point3,
        norm_pln: &Direction3,
        symb_size: f64,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: null dimension aspect"))?;
        let group = current_line_group(prs, la)?;
        let prims = Graphic3dArrayOfSegments::new(10, 0, 0)?;
        prims.add_vertex_pnt(attach)?;
        prims.add_vertex_pnt(end)?;
        let mut dirac = Vector3::between(*attach, *end);
        dirac.normalize()?;
        let mut norac = dirac.crossed(Vector3::from(*norm_pln));
        norac.rotate(Axis1::new(*end, *norm_pln), PI / 8.0)?;
        norac *= symb_size / 2.0;
        prims.add_vertex_pnt(&end.translated(norac))?;
        prims.add_vertex_pnt(&end.translated(-norac))?;
        norac *= 0.8;
        let (p1, p2) = (end.translated(norac), end.translated(-norac));
        dirac *= symb_size / 2.0;
        let middle = Point3::from((p1.xyz() + p2.xyz()) * 0.5);
        for first in [p1, p2, middle] {
            prims.add_vertex_pnt(&first)?;
            prims.add_vertex_pnt(&first.translated(dirac).translated(norac))?;
        }
        add_array(&group, &Handle::new(prims));
        add_ring_point(prs, la, attach)
    }
}

/// `DsgPrs_PerpenPresentation`.
pub struct DsgPrsPerpenPresentation;

impl DsgPrsPerpenPresentation {
    /// `Add`: the right-angle symbol at the offset point and dotted extensions; note that the
    /// dimension line aspect is switched to solid, then dot, as natively.
    /// Example: `DsgPrsPerpenPresentation::add(&prs, &drawer, &a1, &a2, &p1, &p2, &offset, false, true)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        p_ax1: &Point3,
        p_ax2: &Point3,
        pnt1: &Point3,
        pnt2: &Point3,
        offset: &Point3,
        int_out1: bool,
        int_out2: bool,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: null dimension aspect"))?;
        set_line_type(la, AspectTypeOfLine::Solid);
        let group = current_line_group(prs, la)?;
        let prims = Graphic3dArrayOfPolylines::new(6, 2, 0, 0)?;
        prims.add_bound(3)?;
        for point in [offset, p_ax1, p_ax2] {
            prims.add_vertex_pnt(point)?;
        }
        let vec1 = Vector3::from(make_dir(offset, p_ax1)?) * (0.2 * offset.distance(p_ax1));
        let vec2 = Vector3::from(make_dir(offset, p_ax2)?) * (0.2 * offset.distance(p_ax2));
        let (p_ax11, p_ax22) = (offset.translated(vec1), offset.translated(vec2));
        prims.add_bound(3)?;
        for point in [p_ax11, p_ax22.translated(vec1), p_ax22] {
            prims.add_vertex_pnt(&point)?;
        }
        add_array(&group, &Handle::new(prims));
        if !int_out1 && !int_out2 {
            return Ok(());
        }
        set_line_type(la, AspectTypeOfLine::Dot);
        let group = new_line_group(prs, la)?;
        for (out, from, to) in [(int_out1, p_ax1, pnt1), (int_out2, p_ax2, pnt2)] {
            if out {
                add_array(&group, &segments(2, &[*from, *to])?);
            }
        }
        Ok(())
    }
}

fn set_line_type(aspect: &crate::prs3d::Prs3dDimensionAspect, kind: AspectTypeOfLine) {
    if let Some(line) = aspect.line_aspect().get() {
        line.set_type_of_line(kind);
    }
}
