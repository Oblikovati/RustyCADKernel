//! `PrsDim_Dimension` drawing primitives: the value string, arrows, labels (2D text or 3D
//! glyph shapes) and extension lines.
use super::PrsDimDisplaySpecialSymbol;
use super::dimension::{
    PrsDimComputeMode, PrsDimDimensionMembers, PrsDimDimensionSource, PrsDimValueType,
    label_position as lp,
};
use crate::ais::AisInteractiveObjectMembers;
use crate::brep_font::BRepFontBuilderTextPlan;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::prs3d::{Prs3dArrow, Prs3dLineAspect, Prs3dShadingAspect, Prs3dText};
use crate::std_prs::{StdPrsBRepFont, StdPrsShadedShape, StdPrsVolume, StdPrsWfShape};
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::c_string::{FormatArg, sprintf_into};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Axis2, Axis3, Direction3, Line3, Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::aspect::{
    AspectInteriorStyle, AspectPolygonOffsetMode, AspectTypeOfDisplayText, AspectTypeOfLine,
};
use rustycad_tkservice::font::{FontFTFont, FontFTFontParams, FontStrictLevel};
use rustycad_tkservice::graphic3d::*;

/// `THE_3D_TEXT_MARGIN`: the margin of 3D labels in font heights.
pub const THE_3D_TEXT_MARGIN: f64 = 0.1;

/// The label of a value: its text, its width and the 3D font and plan.
#[derive(Clone, Default)]
pub struct PrsDimLabel {
    /// The text.
    pub text: ExtendedString,
    /// The width.
    pub width: f64,
    /// The font of a 3D label.
    pub font: Handle<StdPrsBRepFont>,
    /// The text plan of a 3D label.
    pub plan: Option<BRepFontBuilderTextPlan>,
}

fn null_aspect() -> Failure {
    Failure::with_message("Standard_NullObject", "PrsDim_Dimension: null aspect")
}

/// `Sprintf(buffer, format, value)` as text.
pub(super) fn c_format_real(format: &[u8], value: f64) -> String {
    let mut buffer = Vec::new();
    let _ = sprintf_into(&mut buffer, format, &[FormatArg::Real(value)]);
    buffer.pop();
    String::from_utf8_lossy(&buffer).into_owned()
}

/// The non-virtual drawing members of every dimension.
pub trait PrsDimDimensionDraw: PrsDimDimensionSource {
    /// `GetValueString`: the formatted (or custom) value with its units and special symbol,
    /// with its width and, for 3D text, the font and plan; `dimension.get_value_string()?`.
    fn get_value_string(&self) -> Result<PrsDimLabel, Failure> {
        let aspect = self.attributes().dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let (kind, custom, display, symbol, cache) = {
            let f = self.dimension_base().fields.read();
            (
                f.value_type,
                f.custom_string_value.clone(),
                f.display_special_symbol,
                f.special_symbol,
                f.brep_font_cache.clone(),
            )
        };
        let mut text = if kind == PrsDimValueType::CustomText {
            custom
        } else {
            let value = self.value_to_display_units()?;
            let format = aspect.value_string_format();
            ExtendedString::from_ascii(&AsciiString::from(
                c_format_real(format.as_bytes(), value).as_str(),
            ))
        };
        if aspect.is_units_displayed() {
            text += ExtendedString::from_ascii(&AsciiString::from(" "));
            text += ExtendedString::from_ascii(&self.get_display_units());
        }
        match display {
            PrsDimDisplaySpecialSymbol::Before => text.insert_char(1, symbol)?,
            PrsDimDisplaySpecialSymbol::After => {
                let length = text.length();
                text.insert_char(length + 1, symbol)?;
            }
            PrsDimDisplaySpecialSymbol::No => {}
        }
        let text_aspect = aspect.text_aspect();
        let text_aspect = text_aspect.get().ok_or_else(null_aspect)?;
        let graphic = text_aspect.aspect();
        let graphic = graphic.get().ok_or_else(null_aspect)?;
        let utf8 = text.to_utf8();
        let mut label = PrsDimLabel {
            text,
            ..PrsDimLabel::default()
        };
        if aspect.is_text_3d() {
            let font = match cache.get() {
                None => StdPrsBRepFont::find_and_create(
                    &graphic.font(),
                    graphic.text_font_aspect(),
                    text_aspect.height(),
                    FontStrictLevel::Any,
                ),
                Some(cache) => cache.find_font(
                    &graphic.font(),
                    graphic.text_font_aspect(),
                    text_aspect.height(),
                    FontStrictLevel::Any,
                ),
            };
            if let Some(value) = font.get() {
                label.plan = value.plan_text(
                    &String8::from_unicode_units(label.text.as_utf16(), -1),
                    Graphic3dHorizontalTextAlignment::Left,
                    Graphic3dVerticalTextAlignment::Bottom,
                );
                if let Some(plan) = &label.plan {
                    label.width = plan.width();
                }
            }
            label.font = font;
            return Ok(label);
        }
        let context = self.get_context();
        let context = context.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "PrsDim_Dimension: no context for 2D text",
            )
        })?;
        let rendering = context.current_viewer().default_rendering_params();
        let params = FontFTFontParams {
            point_size: text_aspect.height() as u32,
            resolution: rendering.resolution,
            font_hinting: rendering.font_hinting,
            ..FontFTFontParams::default()
        };
        let font = FontFTFont::find_and_create(
            &graphic.font(),
            graphic.text_font_aspect(),
            &params,
            FontStrictLevel::Any,
        );
        if let Some(font) = font.get() {
            let chars: Vec<u32> = utf8.chars().map(u32::from).collect();
            for (index, current) in chars.iter().enumerate() {
                let next = chars.get(index + 1).copied().unwrap_or(0);
                label.width += f64::from(font.advance_x_of(*current, next));
            }
        }
        Ok(label)
    }

    /// `DrawArrow`: a 3D arrow cone, or a flat triangle (zoom persistent when not zoomable),
    /// recorded for the selection; `dimension.draw_arrow(&prs, &tip, &direction)?`.
    fn draw_arrow(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        location: &Point3,
        direction: &Direction3,
    ) -> Result<(), Failure> {
        let group = prs.new_group()?;
        let aspect = self.attributes().dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let arrow = aspect.arrow_aspect();
        let arrow = arrow.get().ok_or_else(null_aspect)?;
        let (length, angle, zoomable) = (arrow.length(), arrow.angle(), arrow.is_zoomable());
        if aspect.is_arrows_3d() {
            Prs3dArrow::draw(&group, location, direction, angle, length)?;
            group.set_group_primitives_aspect(&arrow.aspect().cast());
        } else {
            let tip = if zoomable { *location } else { gp::origin() };
            let plane = self.get_plane().axis().direction();
            let (left, right) = points_for_arrow(&tip, direction, &plane, length, angle)?;
            let triangle = Handle::new(Graphic3dArrayOfTriangles::new(
                3,
                0,
                GRAPHIC3D_ARRAY_FLAGS_NONE,
            )?);
            for point in [left, tip, right] {
                triangle.add_vertex_pnt(&point)?;
            }
            let style = Graphic3dAspectFillArea3d::default();
            style.set_interior_style(AspectInteriorStyle::Solid);
            let color = arrow
                .aspect()
                .get()
                .ok_or_else(null_aspect)?
                .interior_color();
            style.set_interior_color(color);
            style.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
            style.set_polygon_offset(Graphic3dPolygonOffset {
                mode: AspectPolygonOffsetMode::OFF,
                factor: 0.0,
                units: 0.0,
            });
            group.set_primitives_aspect(&Handle::new(style).cast());
            group.add_primitive_array(&triangle.cast(), true);
            if !zoomable {
                let pers = Graphic3dTransformPers::with_point(
                    Graphic3dTransModeFlags::ZOOM_PERS,
                    location,
                )?;
                group.set_transform_persistence(&Handle::new(pers).cast());
            }
        }
        let mut f = self.dimension_base().fields.write();
        let recorded = f.selection_geom.new_arrow();
        recorded.position = *location;
        recorded.direction = *direction;
        Ok(())
    }

    /// `drawText`: a 3D glyph label (shaded or wireframe, flipping with the view) placed by
    /// the label position, or a 2D dimension text; recorded for the selection.
    /// Example: `dimension.draw_text(&prs, &pos, &dir, &label, position)?`.
    fn draw_text(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        position: &Point3,
        direction: &Direction3,
        label: &PrsDimLabel,
        label_position: i32,
    ) -> Result<(), Failure> {
        let group = prs.new_group()?;
        let drawer = self.attributes();
        let aspect = drawer.dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let text_aspect = aspect.text_aspect();
        if !aspect.is_text_3d() {
            let value = text_aspect.get().ok_or_else(null_aspect)?;
            value
                .aspect()
                .get()
                .ok_or_else(null_aspect)?
                .set_text_display_type(AspectTypeOfDisplayText::Dimension);
            Prs3dText::draw(&group, &text_aspect, &label.text, position);
            let mut f = self.dimension_base().fields.write();
            f.selection_geom.text_pos = *position;
            f.selection_geom.text_dir = *direction;
            f.selection_geom.text_width = 0.0;
            f.selection_geom.text_height = 0.0;
            return Ok(());
        }
        let value = text_aspect.get().ok_or_else(null_aspect)?;
        let color = value
            .aspect()
            .get()
            .ok_or_else(null_aspect)?
            .interior_color();
        let font_height = value.height();
        let (Some(font), Some(plan)) = (label.font.get(), label.plan.as_ref()) else {
            return Ok(());
        };
        let mut shape = font.render_plan(plan);
        if shape.is_null() {
            return Ok(());
        }
        let (width, height) = (plan.width(), plan.height());
        let h_position = label_position & lp::H_MASK;
        let v_position = label_position & lp::V_MASK;
        let text_dir = if h_position == lp::LEFT {
            direction.reversed()
        } else {
            *direction
        };
        let margin = font_height * THE_3D_TEXT_MARGIN;
        let h_offset = match h_position {
            lp::RIGHT => width / 2.0 + margin,
            lp::LEFT => -width / 2.0 - margin,
            _ => 0.0,
        };
        let v_offset = match v_position {
            lp::ABOVE => height / 2.0 + margin,
            lp::BELOW => -height / 2.0 - margin,
            _ => 0.0,
        };
        let (lower, upper) = (plan.lower_left(), plan.upper_right());
        let shape_h = h_offset - (lower.x() + upper.x()) * 0.5;
        let shape_v = v_offset - (lower.y() + upper.y()) * 0.5;
        let mut offset = Transform3::default();
        offset.set_translation_between(gp::origin(), Point3::new(shape_h, shape_v, 0.0));
        shape.move_by(&Location::new(offset), true)?;
        let normal = self.get_plane().axis().direction();
        let system = Axis3::new(*position, normal, text_dir)?;
        let mut plane_trsf = Transform3::default();
        plane_trsf.set_transformation_between(system, Axis3::from_axis2(gp::xoy()));
        shape.move_by(&Location::new(plane_trsf), true)?;
        let mut center_offset = Transform3::default();
        center_offset.set_translation_between(gp::origin(), Point3::new(h_offset, v_offset, 0.0));
        let center = gp::origin()
            .transformed(center_offset)
            .transformed(plane_trsf);
        group.set_flipping_options(true, &Axis2::with_x_direction(center, normal, text_dir)?);
        if aspect.is_text_shaded() {
            if !drawer.has_own_shading_aspect() {
                drawer.set_shading_aspect(Handle::new(Prs3dShadingAspect::new()?));
            }
            let fill = drawer.shading_aspect().aspect();
            fill.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
            fill.set_interior_color(color);
            fill.set_back_interior_color(color);
            StdPrsShadedShape::add(
                prs,
                &shape,
                &drawer,
                StdPrsVolume::Autodetection,
                &Handle::default(),
            )?;
        } else {
            if !drawer.has_own_free_boundary_aspect() {
                drawer.set_free_boundary_aspect(Handle::new(Prs3dLineAspect::new(
                    color,
                    AspectTypeOfLine::Solid,
                    1.0,
                )?));
            }
            drawer
                .free_boundary_aspect()
                .aspect()
                .get()
                .ok_or_else(null_aspect)?
                .set_interior_color(color);
            let edges = StdPrsWfShape::add_all_edges(&shape, &drawer)?;
            if !edges.is_null() {
                group.set_group_primitives_aspect(&drawer.free_boundary_aspect().aspect().cast());
                group.add_primitive_array(&edges.cast(), true);
            }
        }
        prs.current_group()?
            .set_flipping_options(false, &Axis2::default());
        let mut f = self.dimension_base().fields.write();
        f.selection_geom.text_pos = center;
        f.selection_geom.text_dir = text_dir;
        f.selection_geom.text_width = width + margin * 2.0;
        f.selection_geom.text_height = height;
        Ok(())
    }

    /// `DrawExtension`: the label at the end of an extension line and the line itself
    /// (lengthened under a 3D label not centered vertically); recorded for the selection.
    /// Example: `dimension.draw_extension(&prs, size, &start, &dir, &label, mode, position)?`.
    #[allow(clippy::too_many_arguments)]
    fn draw_extension(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        size: f64,
        start: &Point3,
        direction: &Direction3,
        label: &PrsDimLabel,
        mode: i32,
        label_position: i32,
    ) -> Result<(), Failure> {
        let line = Line3::new(*start, *direction);
        let has_label = label.text.length() > 0;
        if has_label
            && (mode == PrsDimComputeMode::All as i32 || mode == PrsDimComputeMode::Text as i32)
        {
            let position = elclib::value(size, &line);
            prs.new_group()?;
            self.draw_text(prs, &position, direction, label, label_position)?;
        }
        if mode != PrsDimComputeMode::All as i32 && mode != PrsDimComputeMode::Line as i32 {
            return Ok(());
        }
        let aspect = self.attributes().dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let is_text_3d = aspect.is_text_3d();
        let short_line = !is_text_3d || label_position & lp::V_CENTER != 0;
        let end = if !has_label || short_line {
            elclib::value(size, &line)
        } else {
            elclib::value(size + label.width, &line)
        };
        let segments = Handle::new(Graphic3dArrayOfSegments::new(
            2,
            0,
            GRAPHIC3D_ARRAY_FLAGS_NONE,
        )?);
        segments.add_vertex_pnt(start)?;
        segments.add_vertex_pnt(&end)?;
        {
            let mut f = self.dimension_base().fields.write();
            let curve = f.selection_geom.new_curve();
            curve.push(*start);
            curve.push(end);
        }
        let group = prs.new_group()?;
        let stencil = !is_text_3d && mode == PrsDimComputeMode::All as i32;
        if stencil {
            group.set_stencil_test_options(true);
        }
        group.set_primitives_aspect(&aspect.line_aspect().aspect().cast());
        group.add_primitive_array(&segments.cast(), true);
        if stencil {
            group.set_stencil_test_options(false);
        }
        Ok(())
    }
}
impl<T: PrsDimDimensionSource + ?Sized> PrsDimDimensionDraw for T {}

/// `PointsForArrow`: the two side points of an arrow at a peak.
/// Example: `points_for_arrow(&peak, &direction, &plane, 5.0, 0.3)?`.
pub fn points_for_arrow(
    peak: &Point3,
    direction: &Direction3,
    plane: &Direction3,
    length: f64,
    angle: f64,
) -> Result<(Point3, Point3), Failure> {
    let arrow = Line3::new(*peak, direction.reversed());
    let end = elclib::value(length, &arrow);
    let edge = Line3::new(end, direction.crossed(*plane)?);
    let edge_length = angle.tan() * length;
    Ok((
        elclib::value(edge_length, &edge),
        elclib::value(-edge_length, &edge),
    ))
}
