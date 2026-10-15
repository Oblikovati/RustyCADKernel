//! `PrsDim_AngleDimension` presentation: the dimension arc (with a gap for a centered 3D
//! label), the label, arrows, extensions and flyout segments.
use super::angle_dimension::PrsDimAngleDimension;
use super::angle_dimension_layout::{
    PrsDimAngleAdjust, dimension_aspect_of, margin_width, null_aspect,
};
use super::dimension::{
    PrsDimComputeMode, PrsDimDimensionMembers, PrsDimDimensionSource, label_position as lp,
};
use super::dimension_draw::{PrsDimDimensionDraw, PrsDimLabel};
use super::dimension_linear::empty_label;
use super::{PrsDimTypeOfAngle, PrsDimTypeOfAngleArrowVisibility as Arrow};
use crate::prs3d::Prs3dDimensionTextVerticalPosition;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::gc::MakeArcOfCircle;
use rustycad_tkgeombase::gce::{MakeCirc, Root};
use rustycad_tkgeombase::gcpnts::UniformAbscissa;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Circle3, Direction3, Plane3, Point3, Vector3};
use rustycad_tkservice::graphic3d::*;
use std::f64::consts::PI;

const ALL: i32 = PrsDimComputeMode::All as i32;
const LINE: i32 = PrsDimComputeMode::Line as i32;
const TEXT: i32 = PrsDimComputeMode::Text as i32;

/// The plane of the arc: at the center, normal to the min-angle normal (reversed for an
/// exterior angle).
fn arc_plane(dimension: &PrsDimAngleDimension) -> Result<Plane3, Failure> {
    let mut plane = Plane3::new(
        dimension.center_point(),
        dimension.get_normal_for_min_angle()?,
    )?;
    if dimension.get_type() == PrsDimTypeOfAngle::Exterior {
        let axis = plane.axis();
        plane.set_axis(Axis1::new(axis.location(), axis.direction().reversed()))?;
    }
    Ok(plane)
}

/// The circle of the arc, `None` when it cannot be built.
fn arc_circle(center: &Point3, plane: Plane3, radius: f64) -> Result<Option<Circle3>, Failure> {
    let maker = MakeCirc::from_center_plane(*center, plane, radius)?;
    if !maker.is_done() {
        return Ok(None);
    }
    Ok(Some(maker.value()?))
}

impl PrsDimAngleDimension {
    /// `DrawArc`: the arc between two attach points, sampled about 50 points per half turn,
    /// added to the current group and recorded for the selection.
    /// Example: `dimension.draw_arc(&prs, &a, &b, &center, 15.0, 0)?`.
    pub fn draw_arc(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        first_attach: &Point3,
        second_attach: &Point3,
        center: &Point3,
        radius: f64,
        mode: i32,
    ) -> Result<(), Failure> {
        let plane = arc_plane(self)?;
        let Some(circle) = arc_circle(center, plane, radius)? else {
            return Ok(());
        };
        let arc = MakeArcOfCircle::from_points(circle, *first_attach, *second_attach, true)?;
        if !arc.is_done() {
            return Ok(());
        }
        let curve: Handle<dyn GeomCurve> = arc.value()?.cast();
        let trimmed = curve.get().ok_or_else(null_aspect)?;
        let adaptor = GeomAdaptorCurve::with_range(
            &curve,
            trimmed.first_parameter(),
            trimmed.last_parameter(),
        )?;
        let mut angle = Vector3::between(*center, *first_attach)
            .angle(Vector3::between(*center, *second_attach))?;
        if self.get_type() == PrsDimTypeOfAngle::Exterior {
            angle = 2.0 * PI - angle;
        }
        let nb_points = 4.max((50.0 * angle / PI) as i32);
        let sampler = UniformAbscissa::with_nb_points(&adaptor, nb_points, -1.0)?;
        if !sampler.is_done() {
            return Ok(());
        }
        let polyline = Handle::new(Graphic3dArrayOfPolylines::new(
            nb_points,
            0,
            0,
            GRAPHIC3D_ARRAY_FLAGS_NONE,
        )?);
        let mut points = Vec::new();
        for index in 1..=sampler.nb_points()? {
            let point = adaptor.value(sampler.parameter(index)?)?;
            polyline.add_vertex_pnt(&point)?;
            points.push(point);
        }
        self.base
            .fields
            .write()
            .selection_geom
            .new_curve()
            .extend(points);
        let aspect = dimension_aspect_of(self)?;
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let stencil = !aspect.is_text_3d() && mode == ALL;
        let group = prs.current_group()?;
        if stencil {
            group.set_stencil_test_options(true);
        }
        group.set_primitives_aspect(&aspect.line_aspect().aspect().cast());
        group.add_primitive_array(&polyline.cast(), true);
        if stencil {
            group.set_stencil_test_options(false);
        }
        Ok(())
    }

    /// `DrawArcWithText`: the label at the middle of the arc and the arc, broken around a
    /// centered 3D label.
    /// Example: `dimension.draw_arc_with_text(&prs, &a, &b, &center, &label, mode, position)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc_with_text(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        first_attach: &Point3,
        second_attach: &Point3,
        center: &Point3,
        label: &PrsDimLabel,
        mode: i32,
        label_position: i32,
    ) -> Result<(), Failure> {
        let plane = Plane3::new(self.center_point(), self.get_normal_for_min_angle()?)?;
        let radius = first_attach.distance(&self.center_point());
        let Some(circle) = arc_circle(center, plane, radius)? else {
            return Ok(());
        };
        let (begin, end) = elclib::adjust_periodic(
            0.0,
            PI * 2.0,
            precision::p_confusion(),
            elclib::parameter(&circle, *first_attach)?,
            elclib::parameter(&circle, *second_attach)?,
        );
        let middle = (begin + end) * 0.5;
        if mode == ALL || mode == TEXT {
            let position = elclib::value(middle, &circle);
            let direction = make_dir(first_attach, second_attach)?;
            self.draw_text(prs, &position, &direction, label, label_position)?;
        }
        if mode != ALL && mode != LINE {
            return Ok(());
        }
        let aspect = dimension_aspect_of(self)?;
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let line_break = aspect.text_vertical_position()
            == Prs3dDimensionTextVerticalPosition::Center
            && aspect.is_text_3d();
        if !line_break {
            return self.draw_arc(prs, first_attach, second_attach, center, radius, mode);
        }
        let sector = label.width / radius;
        let text_begin = middle - sector * 0.5;
        let text_end = middle + sector * 0.5;
        let text_begin_point = elclib::value(text_begin, &circle);
        let text_end_point = elclib::value(text_end, &circle);
        if text_begin > begin {
            self.draw_arc(prs, first_attach, &text_begin_point, center, radius, mode)?;
        }
        if text_end < end {
            self.draw_arc(prs, &text_end_point, second_attach, center, radius, mode)?;
        }
        Ok(())
    }
}

/// `gce_MakeDir(P1, P2)` converted to a direction (StdFail_NotDone for equal points).
fn make_dir(from: &Point3, to: &Point3) -> Result<Direction3, Failure> {
    Direction3::from_xyz(to.xyz() - from.xyz())
        .map_err(|_| Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result"))
}

/// The arrows of the angle: attach points, arrow vectors, arrow ends and extension directions.
struct ArrowLayout {
    first_attach: Point3,
    second_attach: Point3,
    first_vec: Vector3,
    second_vec: Vector3,
    first_end: Point3,
    second_end: Point3,
    first_extension: Direction3,
    second_extension: Direction3,
}

fn arrow_layout(
    dimension: &PrsDimAngleDimension,
    arrow_length: f64,
    external: bool,
    zoomable: bool,
) -> Result<ArrowLayout, Failure> {
    let center = dimension.center_point();
    let (first_attach, second_attach) = dimension.attach_points(dimension.get_flyout())?;
    let working = Vector3::from(dimension.get_normal_for_min_angle()?);
    let first_extension = Direction3::from_vector(
        working
            .reversed()
            .crossed(Vector3::between(center, first_attach)),
    )?;
    let second_extension =
        Direction3::from_vector(working.crossed(Vector3::between(center, second_attach)))?;
    let mut first_vec = Vector3::from(first_extension) * arrow_length;
    let mut second_vec = Vector3::from(second_extension) * arrow_length;
    if external {
        first_vec.reverse();
        second_vec.reverse();
    }
    let (mut first_end, mut second_end) = (first_attach, second_attach);
    if zoomable {
        first_end = first_end.translated(first_vec.reversed());
        second_end = second_end.translated(second_vec.reversed());
    }
    Ok(ArrowLayout {
        first_attach,
        second_attach,
        first_vec,
        second_vec,
        first_end,
        second_end,
        first_extension,
        second_extension,
    })
}

/// The body of `PrsDim_AngleDimension::Compute`.
pub(super) fn compute_angle(
    dimension: &PrsDimAngleDimension,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    dimension.base.fields.write().selection_geom.clear(mode);
    if !dimension.is_valid() {
        return Ok(());
    }
    let aspect = dimension_aspect_of(dimension)?;
    let aspect = aspect.get().ok_or_else(null_aspect)?;
    prs.current_group()?
        .set_primitives_aspect(&aspect.line_aspect().aspect().cast());
    let arrow = aspect.arrow_aspect();
    let arrow = arrow.get().ok_or_else(null_aspect)?;
    let mut label = dimension.get_value_string()?;
    label.width = margin_width(aspect, label.width)?;
    let mut extension_size = aspect.extension_size();
    let mut horizontal = aspect.text_horizontal_position();
    if dimension.is_text_position_custom() {
        let (text, flyout) = {
            let f = dimension.base.fields.read();
            (f.fixed_text_position, f.flyout)
        };
        let mut adjust = PrsDimAngleAdjust {
            extension_size,
            alignment: horizontal,
            flyout,
        };
        dimension.adjust_parameters(&text, &mut adjust)?;
        dimension.base.fields.write().flyout = adjust.flyout;
        extension_size = adjust.extension_size;
        horizontal = adjust.alignment;
    }
    let fit = dimension.fit_text_alignment(horizontal, label.width)?;
    let external = fit.arrows_external;
    let layout = arrow_layout(dimension, arrow.length(), external, arrow.is_zoomable())?;
    let center = dimension.center_point();
    prs.new_group()?;
    let h_position = fit.label_position & lp::H_MASK;
    let show_first = dimension.is_arrow_visible(Arrow::First);
    let show_second = dimension.is_arrow_visible(Arrow::Second);
    let arc_first = if external || !show_first {
        layout.first_attach
    } else {
        layout.first_end
    };
    let arc_second = if external || !show_second {
        layout.second_attach
    } else {
        layout.second_end
    };
    let radius = dimension.get_flyout().abs();
    let lines = mode == ALL || mode == LINE;
    match h_position {
        lp::H_CENTER => {
            let line_break = aspect.text_vertical_position()
                == Prs3dDimensionTextVerticalPosition::Center
                && aspect.is_text_3d();
            if line_break {
                dimension.draw_arc_with_text(
                    prs,
                    &layout.first_attach,
                    &layout.second_attach,
                    &center,
                    &label,
                    mode,
                    fit.label_position,
                )?;
            } else {
                if mode == ALL || mode == TEXT {
                    let position = if dimension.is_text_position_custom() {
                        dimension.base.fields.read().fixed_text_position
                    } else {
                        dimension.get_center_on_arc(
                            &layout.first_attach,
                            &layout.second_attach,
                            &center,
                        )?
                    };
                    let direction = Direction3::from_vector(Vector3::between(
                        layout.first_attach,
                        layout.second_attach,
                    ))?;
                    dimension.draw_text(prs, &position, &direction, &label, fit.label_position)?;
                }
                if lines {
                    dimension.draw_arc(prs, &arc_first, &arc_second, &center, radius, mode)?;
                }
            }
        }
        lp::LEFT => {
            let start = if external && show_first {
                layout.first_end
            } else {
                layout.first_attach
            };
            dimension.draw_extension(
                prs,
                extension_size,
                &start,
                &layout.first_extension,
                &label,
                mode,
                fit.label_position,
            )?;
        }
        lp::RIGHT => {
            let start = if external && show_second {
                layout.second_end
            } else {
                layout.second_attach
            };
            dimension.draw_extension(
                prs,
                extension_size,
                &start,
                &layout.second_extension,
                &label,
                mode,
                fit.label_position,
            )?;
        }
        _ => {}
    }
    if lines && h_position != lp::H_CENTER {
        prs.new_group()?;
        dimension.draw_arc(prs, &arc_first, &arc_second, &center, radius, mode)?;
    }
    if lines {
        prs.new_group()?;
        if show_first {
            let direction = Direction3::from_vector(layout.first_vec)?;
            dimension.draw_arrow(prs, &layout.first_attach, &direction)?;
        }
        if show_second {
            let direction = Direction3::from_vector(layout.second_vec)?;
            dimension.draw_arrow(prs, &layout.second_attach, &direction)?;
        }
    }
    if lines && external {
        prs.new_group()?;
        let empty = empty_label(&label);
        let tail = aspect.arrow_tail_size();
        if h_position != lp::LEFT && show_first {
            dimension.draw_extension(
                prs,
                tail,
                &layout.first_end,
                &layout.first_extension,
                &empty,
                mode,
                lp::NONE,
            )?;
        }
        if h_position != lp::RIGHT && show_second {
            dimension.draw_extension(
                prs,
                tail,
                &layout.second_end,
                &layout.second_extension,
                &empty,
                mode,
                lp::NONE,
            )?;
        }
    }
    if mode == ALL {
        let group = prs.new_group()?;
        let segments = Handle::new(Graphic3dArrayOfSegments::new(
            4,
            0,
            GRAPHIC3D_ARRAY_FLAGS_NONE,
        )?);
        for point in [center, layout.first_attach, center, layout.second_attach] {
            segments.add_vertex_pnt(&point)?;
        }
        group.set_primitives_aspect(&aspect.line_aspect().aspect().cast());
        group.add_primitive_array(&segments.cast(), true);
    }
    dimension.base.fields.write().selection_geom.is_computed = true;
    Ok(())
}
