//! Native colored triangle, frame polyline and text submissions through existing Graphic3d interfaces.
use super::{
    AisColorScale,
    color_scale_bar::ColorScaleQuad,
    color_scale_layout::{ColorScaleFrame, ColorScaleTextPlacement},
};
use crate::{prs_mgr::*, prs3d::Prs3dTextAspect};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    ncollection::String8,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
impl AisColorScale {
    fn update_scale_text_aspect(&self) {
        self.initialize_scale_text_aspect();
        let aspect = self.attributes().text_aspect();
        aspect.set_color(self.scale_foreground());
        aspect.set_height(f64::from(self.get_text_height()));
        aspect.set_horizontal_justification(Graphic3dHorizontalTextAlignment::Left);
        aspect.set_vertical_justification(Graphic3dVerticalTextAlignment::Bottom);
        aspect.aspect().set_text_zoomable(true);
    }
    fn scale_foreground(&self) -> Color {
        if self.has_color() {
            self.attributes().color()
        } else {
            Color::named(NamedColor::WHITE)
        }
    }
    fn compute_scale(&self, prs: &Handle<dyn Graphic3dStructureSource>) -> Result<(), Failure> {
        self.update_scale_text_aspect();
        let frame = self.color_scale_frame()?;
        let labels = self.scale_text_group(prs, &frame)?;
        self.submit_scale_title(&labels, &frame)?;
        self.submit_scale_bar(prs, &frame)?;
        for label in self.label_placements(&frame)? {
            self.submit_scale_text(&labels, &label)?;
        }
        Ok(())
    }
    fn submit_scale_text(
        &self,
        group: &Handle<dyn Graphic3dGroup>,
        placement: &ColorScaleTextPlacement,
    ) -> Result<(), Failure> {
        let text = Handle::new(Graphic3dText::new(
            self.attributes().text_aspect().height() as f32
        ));
        let mut units = placement.text.as_utf16().to_vec();
        units.push(0);
        text.set_text(&String8::from_unicode_units(&units, -1));
        text.set_orientation(&Axis2::new(
            Point3::new(f64::from(placement.x), f64::from(placement.y), 0.0),
            Direction3::Z,
        )?);
        text.set_own_anchor_point(false);
        text.set_vertical_alignment(placement.alignment);
        group.add_text(&text, true);
        Ok(())
    }
    fn submit_scale_bar(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        frame: &ColorScaleFrame,
    ) -> Result<(), Failure> {
        let quads = self.color_bar_quads(frame.bottom, frame.height)?;
        if quads.is_empty() {
            return Ok(());
        }
        let triangles = Handle::new(Graphic3dArrayOfTriangles::new(
            quads.len() as i32 * 4,
            quads.len() as i32 * 6,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR,
        )?);
        for quad in &quads {
            append_scale_quad(&triangles, frame.bar_x, frame.width, quad)?;
        }
        let group = prs.new_group()?;
        group.set_group_primitives_aspect(&self.attributes().shading_aspect().aspect().cast());
        group.add_primitive_array(&triangles.cast(), true);
        self.submit_scale_frame(prs, frame)
    }
    fn submit_scale_frame(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        frame: &ColorScaleFrame,
    ) -> Result<(), Failure> {
        let x = frame.bar_x - 1;
        let y = frame.bottom - 1;
        let right = x + frame.width + 2;
        let top = y + frame.height + 2;
        let lines = Handle::new(Graphic3dArrayOfPolylines::new(5, 0, 0, 0)?);
        for (x, y) in [(x, y), (right, y), (right, top), (x, top), (x, y)] {
            lines.add_vertex(x as f32, y as f32, 0.0)?;
        }
        let aspect = Handle::new(Graphic3dAspectLine3d::new(
            self.scale_foreground(),
            AspectTypeOfLine::Solid,
            1.0,
        )?);
        let group = prs.new_group()?;
        group.set_group_primitives_aspect(&aspect.cast());
        group.add_primitive_array(&lines.cast(), true);
        Ok(())
    }
}
fn append_scale_quad(
    triangles: &Graphic3dArrayOfTriangles,
    x: i32,
    width: i32,
    quad: &ColorScaleQuad,
) -> Result<(), Failure> {
    let start = triangles.vertex_number() + 1;
    for (x, y, color) in [
        (x, quad.y + quad.height, quad.top),
        (x, quad.y, quad.bottom),
        (x + width, quad.y, quad.bottom),
        (x + width, quad.y + quad.height, quad.top),
    ] {
        triangles.add_vertex_pnt_color(&Point3::new(f64::from(x), f64::from(y), 0.0), color)?;
    }
    triangles.add_quad_triangle_edges(start, start + 1, start + 2, start + 3)?;
    Ok(())
}
impl PrsMgrPresentableObject for AisColorScale {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != 0 {
            return Ok(());
        }
        self.compute_scale(&prs.observe())
    }
    crate::ais_presentable_overrides!();
}

impl AisColorScale {
    fn initialize_scale_text_aspect(&self) {
        let drawer = self.attributes();
        if drawer.has_own_text_aspect() {
            return;
        }
        let own = Handle::new(Prs3dTextAspect::new());
        let linked = drawer.link();
        if let Some(inherited) = linked
            .get()
            .map(|link| link.text_aspect())
            .filter(|aspect| !aspect.is_null())
        {
            *own.aspect().aspects_base().fields.write() =
                inherited.aspect().aspects_base().fields.read().clone();
        }
        // OCCT dereferences a null drawer link here; retain the default text aspect safely.
        drawer.set_text_aspect(own);
    }
    fn scale_text_group(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        frame: &ColorScaleFrame,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        if self.fields.read().inputs.title.is_empty() && frame.labels.is_empty() {
            return Ok(Handle::default());
        }
        let group = prs.new_group()?;
        group.set_group_primitives_aspect(&self.attributes().text_aspect().aspect().cast());
        Ok(group)
    }
    fn submit_scale_title(
        &self,
        group: &Handle<dyn Graphic3dGroup>,
        frame: &ColorScaleFrame,
    ) -> Result<(), Failure> {
        let title = self.fields.read().inputs.title.clone();
        if title.is_empty() {
            return Ok(());
        }
        let placement = ColorScaleTextPlacement {
            text: title,
            x: frame.layout.x + frame.layout.spacing,
            y: frame.top + frame.layout.text_height / 2 + 2 * frame.layout.spacing,
            alignment: Graphic3dVerticalTextAlignment::Bottom,
        };
        self.submit_scale_text(group, &placement)
    }
}
