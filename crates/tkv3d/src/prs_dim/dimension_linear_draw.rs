//! `PrsDim_Dimension::DrawLinearDimension`: the label, dimension line, arrows, extensions
//! and flyout lines of a linear dimension.
use super::dimension::{
    PrsDimComputeMode, PrsDimDimensionMembers, PrsDimDimensionSource, label_position as lp,
};
use super::dimension_draw::{PrsDimDimensionDraw, THE_3D_TEXT_MARGIN};
use super::dimension_linear::{
    PrsDimDimensionLinear, PrsDimLinearAdjust, empty_label, line_aspect_of, program,
};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::prs3d::Prs3dDimensionTextVerticalPosition;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::gce::MakeLin;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::*;

const ALL: i32 = PrsDimComputeMode::All as i32;
const LINE: i32 = PrsDimComputeMode::Line as i32;
const TEXT: i32 = PrsDimComputeMode::Text as i32;

fn null_aspect() -> Failure {
    Failure::with_message("Standard_NullObject", "PrsDim_Dimension: null aspect")
}

/// The geometry of a linear dimension line.
struct Layout {
    line: Line3,
    begin: Point3,
    end: Point3,
    first_arrow: (Point3, Direction3, Point3),
    second_arrow: (Point3, Direction3, Point3),
    first_extension: Direction3,
    second_extension: Direction3,
    center_begin: Point3,
    center_end: Point3,
    external: bool,
}

fn segments(
    prs: &Handle<dyn Graphic3dStructureSource>,
    points: &[Point3],
    aspect: &Handle<dyn Graphic3dAspectsSource>,
    stencil: bool,
    group_aspect: bool,
) -> Result<(), Failure> {
    let array = Handle::new(Graphic3dArrayOfSegments::new(
        points.len() as i32,
        0,
        GRAPHIC3D_ARRAY_FLAGS_NONE,
    )?);
    for point in points {
        array.add_vertex_pnt(point)?;
    }
    let group = prs.new_group()?;
    if stencil {
        group.set_stencil_test_options(true);
    }
    if group_aspect {
        group.set_group_primitives_aspect(aspect);
    } else {
        group.set_primitives_aspect(aspect);
    }
    group.add_primitive_array(&array.cast(), true);
    if stencil {
        group.set_stencil_test_options(false);
    }
    Ok(())
}

fn record<D: PrsDimDimensionSource + ?Sized>(dimension: &D, points: &[Point3]) {
    let mut f = dimension.dimension_base().fields.write();
    f.selection_geom.new_curve().extend_from_slice(points);
}

/// The body of `DrawLinearDimension`.
pub(super) fn draw_linear<D: PrsDimDimensionSource + ?Sized>(
    dimension: &D,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
    first: &Point3,
    second: &Point3,
    one_side: bool,
) -> Result<(), Failure> {
    if first.is_equal(second, precision::confusion()) {
        return Err(program("Can not build presentation for equal points."));
    }
    let aspect = dimension.attributes().dimension_aspect();
    let aspect = aspect.get().ok_or_else(null_aspect)?;
    let arrow = aspect.arrow_aspect();
    let arrow = arrow.get().ok_or_else(null_aspect)?;
    let mut extension_size = aspect.extension_size();
    let mut label = dimension.get_value_string()?;
    let is_text_3d = aspect.is_text_3d();
    if is_text_3d {
        label.width +=
            aspect.text_aspect().get().ok_or_else(null_aspect)?.height() * THE_3D_TEXT_MARGIN * 2.0;
    }
    let mut horizontal = aspect.text_horizontal_position();
    if dimension.is_text_position_custom() {
        let (text, flyout, plane, custom) = {
            let f = dimension.dimension_base().fields.read();
            (f.fixed_text_position, f.flyout, f.plane, f.is_plane_custom)
        };
        let mut adjust = PrsDimLinearAdjust {
            extension_size,
            alignment: horizontal,
            flyout,
            plane,
            is_plane_old: custom,
        };
        if !dimension.adjust_parameters_for_linear(&text, first, second, &mut adjust)? {
            return Err(program(
                "Can not adjust plane to the custom label position.",
            ));
        }
        let mut f = dimension.dimension_base().fields.write();
        f.flyout = adjust.flyout;
        f.plane = adjust.plane;
        f.is_plane_custom = adjust.is_plane_old;
        drop(f);
        extension_size = adjust.extension_size;
        horizontal = adjust.alignment;
    }
    let fit = dimension.fit_text_alignment_for_linear(
        first,
        second,
        one_side,
        horizontal,
        label.width,
    )?;
    let layout = layout(
        dimension,
        first,
        second,
        fit.arrows_external,
        arrow.length(),
        arrow.is_zoomable(),
        one_side,
    )?;
    let line_aspect = line_aspect_of(dimension)?;
    let arrow_tail = aspect.arrow_tail_size();
    match fit.label_position & lp::H_MASK {
        lp::H_CENTER => {
            let text_pos = if dimension.is_text_position_custom() {
                dimension.dimension_base().fields.read().fixed_text_position
            } else {
                Point3::from((layout.center_begin.xyz() + layout.center_end.xyz()) * 0.5)
            };
            if mode == ALL || mode == TEXT {
                prs.new_group()?;
                dimension.draw_text(
                    prs,
                    &text_pos,
                    &layout.line.direction(),
                    &label,
                    fit.label_position,
                )?;
            }
            if mode == ALL || mode == LINE {
                let line_break = aspect.text_vertical_position()
                    == Prs3dDimensionTextVerticalPosition::Center
                    && is_text_3d;
                let points = if line_break {
                    let at = elclib::parameter(&layout.line, text_pos)?;
                    let end1 = elclib::value(at - label.width * 0.5, &layout.line);
                    let begin2 = elclib::value(at + label.width * 0.5, &layout.line);
                    record(dimension, &[layout.center_begin, end1]);
                    record(dimension, &[begin2, layout.center_end]);
                    vec![layout.center_begin, end1, begin2, layout.center_end]
                } else {
                    record(dimension, &[layout.center_begin, layout.center_end]);
                    vec![layout.center_begin, layout.center_end]
                };
                let justification = if fit.label_position & lp::V_MASK == lp::BELOW {
                    Graphic3dVerticalTextAlignment::Top
                } else {
                    Graphic3dVerticalTextAlignment::Bottom
                };
                aspect
                    .text_aspect()
                    .get()
                    .ok_or_else(null_aspect)?
                    .set_vertical_justification(justification);
                segments(
                    prs,
                    &points,
                    &line_aspect,
                    !is_text_3d && mode == ALL,
                    false,
                )?;
                prs.new_group()?;
                dimension.draw_arrow(prs, &layout.first_arrow.0, &layout.first_arrow.1)?;
                if !one_side {
                    dimension.draw_arrow(prs, &layout.second_arrow.0, &layout.second_arrow.1)?;
                }
                if layout.external {
                    let empty = empty_label(&label);
                    dimension.draw_extension(
                        prs,
                        arrow_tail,
                        &layout.first_arrow.2,
                        &layout.first_extension,
                        &empty,
                        mode,
                        lp::NONE,
                    )?;
                    if !one_side {
                        dimension.draw_extension(
                            prs,
                            arrow_tail,
                            &layout.second_arrow.2,
                            &layout.second_extension,
                            &empty,
                            mode,
                            lp::NONE,
                        )?;
                    }
                }
            }
        }
        lp::LEFT => {
            let start = if layout.external {
                layout.first_arrow.2
            } else {
                layout.first_arrow.0
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
            if mode == ALL || mode == LINE {
                segments(
                    prs,
                    &[layout.center_begin, layout.center_end],
                    &line_aspect,
                    false,
                    false,
                )?;
                record(dimension, &[layout.center_begin, layout.center_end]);
                prs.new_group()?;
                dimension.draw_arrow(prs, &layout.first_arrow.0, &layout.first_arrow.1)?;
                if !one_side {
                    dimension.draw_arrow(prs, &layout.second_arrow.0, &layout.second_arrow.1)?;
                }
                if layout.external && !one_side {
                    let empty = empty_label(&label);
                    dimension.draw_extension(
                        prs,
                        arrow_tail,
                        &layout.second_arrow.2,
                        &layout.second_extension,
                        &empty,
                        mode,
                        lp::NONE,
                    )?;
                }
            }
        }
        lp::RIGHT => {
            let start = if layout.external {
                layout.second_arrow.2
            } else {
                layout.second_arrow.0
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
            if mode == ALL || mode == LINE {
                segments(
                    prs,
                    &[layout.center_begin, layout.center_end],
                    &line_aspect,
                    false,
                    true,
                )?;
                record(dimension, &[layout.center_begin, layout.center_end]);
                prs.new_group()?;
                dimension.draw_arrow(prs, &layout.second_arrow.0, &layout.second_arrow.1)?;
                if !one_side {
                    dimension.draw_arrow(prs, &layout.first_arrow.0, &layout.first_arrow.1)?;
                }
                if layout.external && !one_side {
                    let empty = empty_label(&label);
                    dimension.draw_extension(
                        prs,
                        arrow_tail,
                        &layout.first_arrow.2,
                        &layout.first_extension,
                        &empty,
                        mode,
                        lp::NONE,
                    )?;
                }
            }
        }
        _ => {}
    }
    if mode == ALL {
        segments(
            prs,
            &[*first, layout.begin, *second, layout.end],
            &line_aspect,
            false,
            true,
        )?;
    }
    dimension
        .dimension_base()
        .fields
        .write()
        .selection_geom
        .is_computed = true;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn layout<D: PrsDimDimensionSource + ?Sized>(
    dimension: &D,
    first: &Point3,
    second: &Point3,
    external: bool,
    arrow_length: f64,
    zoomable: bool,
    one_side: bool,
) -> Result<Layout, Failure> {
    let (begin, end) = dimension.compute_flyout_line_points(first, second)?;
    let line = MakeLin::from_points(begin, end)?.value()?;
    let direction = line.direction();
    let mut first_dir = direction.reversed();
    let mut second_dir = direction;
    if external {
        first_dir.reverse();
        second_dir.reverse();
    }
    let (mut first_end, mut second_end) = (begin, end);
    if zoomable {
        first_end = first_end.translated(Vector3::from(first_dir).scaled(arrow_length).reversed());
        second_end =
            second_end.translated(Vector3::from(second_dir).scaled(arrow_length).reversed());
    }
    Ok(Layout {
        line,
        begin,
        end,
        first_arrow: (begin, first_dir, first_end),
        second_arrow: (end, second_dir, second_end),
        first_extension: direction.reversed(),
        second_extension: direction,
        center_begin: if external { begin } else { first_end },
        center_end: if external || one_side {
            end
        } else {
            second_end
        },
        external,
    })
}
