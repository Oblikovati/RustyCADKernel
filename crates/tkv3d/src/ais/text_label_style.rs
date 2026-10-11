//! Native label setters retain aspect identity; only color and transparency synchronize.
use super::AisTextLabel;
use crate::{prs_mgr::*, prs3d::Prs3dTextAspect};
use rustycad_tkernel::{
    quantity::{Color, ColorRgba},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkservice::{aspect::AspectTypeOfDisplayText, font::FontFontAspect, graphic3d::*};
impl AisTextLabel {
    pub(super) fn label_aspect(&self) -> Handle<Prs3dTextAspect> {
        self.attributes().text_aspect()
    }
    pub(super) fn set_label_color(&self, color: Color) {
        self.base.presentable.fields.write().has_own_color = true;
        self.attributes().set_color(color);
        self.label_aspect().set_color(color);
        self.synchronize_aspects();
    }
    pub(super) fn set_label_transparency(&self, value: f64) {
        let aspect = self.label_aspect().aspect();
        let alpha = (1.0 - value) as f32;
        aspect.set_interior_color_rgba(ColorRgba::from_rgb(aspect.interior_color(), alpha));
        aspect.set_edge_color_rgba(ColorRgba::from_rgb(aspect.edge_color(), alpha));
        self.attributes().set_transparency(value as f32);
        self.synchronize_aspects();
    }
    /// Native font name, with owned text-aspect identity; `label.font_name()`.
    pub fn font_name(&self) -> AsciiString {
        self.label_aspect().aspect().font()
    }
    /// Native font style; `label.font_aspect()`.
    pub fn font_aspect(&self) -> FontFontAspect {
        self.label_aspect().aspect().text_font_aspect()
    }
    /// Set horizontal anchor alignment; `label.set_h_justification(Graphic3dHorizontalTextAlignment::Center)`.
    pub fn set_h_justification(&self, value: Graphic3dHorizontalTextAlignment) {
        self.label_aspect().set_horizontal_justification(value);
    }
    /// Set vertical anchor alignment; `label.set_v_justification(Graphic3dVerticalTextAlignment::Top)`.
    pub fn set_v_justification(&self, value: Graphic3dVerticalTextAlignment) {
        self.label_aspect().set_vertical_justification(value);
    }
    /// Convert label radians to native aspect degrees; `label.set_angle(1.0)`.
    pub fn set_angle(&self, value: f64) {
        self.label_aspect()
            .aspect()
            .set_text_angle_f64(value * 180.0 / std::f64::consts::PI);
    }
    /// Set screen/physical sizing policy; `label.set_zoomable(true)`.
    pub fn set_zoomable(&self, value: bool) {
        self.label_aspect().aspect().set_text_zoomable(value);
    }
    /// Set point size without validating native storage; `label.set_height(16.0)`.
    pub fn set_height(&self, value: f64) {
        self.label_aspect().set_height(value);
    }
    /// Set the font face style; `label.set_font_aspect(FontFontAspect::Bold)`.
    pub fn set_font_aspect(&self, value: FontFontAspect) {
        self.label_aspect().aspect().set_text_font_aspect(value);
    }
    /// Set the requested system font; `label.set_font("DejaVu Sans")`.
    pub fn set_font(&self, value: &str) {
        self.label_aspect().set_font(value);
    }
    /// Retain the legacy display mode for the graphic driver; `label.set_display_type(AspectTypeOfDisplayText::Normal)`.
    pub fn set_display_type(&self, value: AspectTypeOfDisplayText) {
        self.label_aspect().aspect().set_text_display_type(value);
    }
    /// Retain subtitle RGB and its current opacity; `label.set_color_subtitle(Color::default())`.
    pub fn set_color_subtitle(&self, value: Color) {
        self.label_aspect().aspect().set_edge_color(value);
    }
}
impl PrsMgrPresentableObject for AisTextLabel {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != 0 {
            return Ok(());
        }
        self.compute_label(prs)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.set_label_color(color);
        Ok(())
    }
    fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        self.set_label_transparency(value);
        Ok(())
    }
    fn unset_transparency(&self) -> Result<(), Failure> {
        self.set_transparency(0.0)
    }
    fn set_material(&self, _: &Graphic3dMaterialAspect) -> Result<(), Failure> {
        Ok(())
    }
    crate::ais_presentable_overrides!();
}
