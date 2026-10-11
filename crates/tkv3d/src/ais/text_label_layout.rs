//! Label metrics deliberately ignore the optional presentation formatter (OCCT 3d097a0).
use super::{AisInteractiveObjectMembers, AisTextLabel};
use rustycad_tkernel::{
    ncollection::String8,
    standard::{Failure, exceptions::null_object},
};
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3, Transform3, Vector3};
use rustycad_tkservice::{font::*, graphic3d::*};
impl AisTextLabel {
    fn measurement_font(&self) -> Result<rustycad_tkernel::standard::Handle<FontFTFont>, Failure> {
        let rendering = self.label_rendering_params()?;
        let params = FontFTFontParams {
            point_size: self.label_aspect().height() as u32,
            resolution: rendering.resolution,
            font_hinting: rendering.font_hinting,
            ..Default::default()
        };
        Ok(FontFTFont::find_and_create(
            &self.font_name(),
            self.font_aspect(),
            &params,
            FontStrictLevel::Any,
        ))
    }
    fn label_rendering_params(&self) -> Result<Graphic3dRenderingParams, Failure> {
        let context = self.get_context();
        let Some(context) = context.get() else {
            return Err(null_object(
                "AIS_TextLabel context is null; expected a live interactive context for font measurement",
            ));
        };
        let viewer = context.current_viewer().clone();
        let Some(viewer) = viewer.get() else {
            return Err(null_object(
                "AIS_TextLabel viewer is null; expected a live viewer for font measurement",
            ));
        };
        Ok(viewer.default_rendering_params())
    }
    /// Native center and dimensions, or no valid font; `label.calculate_label_params(origin)?`.
    pub fn calculate_label_params(
        &self,
        position: Point3,
    ) -> Result<Option<(Point3, f64, f64)>, Failure> {
        let font = self.measurement_font()?;
        let Some(font) = font.get() else {
            return Ok(None);
        };
        let aspect = self.label_aspect();
        let mut units = self.text().as_utf16().to_vec();
        units.push(0);
        let text = String8::from_unicode_units(&units, -1);
        let bounds = font.bounding_box(
            &text,
            aspect.horizontal_justification(),
            aspect.vertical_justification(),
        )?;
        let width = f64::from(bounds.width().abs());
        let height = f64::from(bounds.height().abs());
        Ok(Some((
            self.label_center(position, width, height),
            width,
            height,
        )))
    }
    fn label_center(&self, position: Point3, width: f64, height: f64) -> Point3 {
        let aspect = self.label_aspect();
        let x = match aspect.horizontal_justification() {
            Graphic3dHorizontalTextAlignment::Left => width * 0.5,
            Graphic3dHorizontalTextAlignment::Right => -width * 0.5,
            _ => 0.0,
        };
        let y = match aspect.vertical_justification() {
            Graphic3dVerticalTextAlignment::Bottom => height * 0.5,
            Graphic3dVerticalTextAlignment::Top => -height * 0.5,
            _ => 0.0,
        };
        let axes = self.orientation_3d();
        Point3::from(position.xyz() + axes.x_direction().xyz() * x + axes.y_direction().xyz() * y)
    }
    /// Rotate axes and center around world Z, then build the label plane; `label.calculate_label_trsf(origin,&mut center)?`.
    pub fn calculate_label_trsf(
        &self,
        position: Point3,
        center: &mut Point3,
    ) -> Result<Transform3, Failure> {
        let angle =
            f64::from(self.label_aspect().aspect().text_angle()) * std::f64::consts::PI / 180.0;
        let rotation = Axis1::new(position, Direction3::Z);
        let axes = self.orientation_3d().rotated(rotation, angle)?;
        *center = center.rotated(rotation, angle)?;
        let mut plane = Transform3::default();
        plane.set_transformation_between(Axis3::from_axis2(axes), Axis3::default());
        plane.set_translation_part(Vector3::from(center.xyz()));
        Ok(plane)
    }
}
