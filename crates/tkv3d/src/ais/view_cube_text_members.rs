//! Text style changes synchronize live aspects; height changes rebuild text geometry.
use super::{AisViewCubeColorRef, AisViewCubeFontRef, AisViewCubePresentationSource};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::{
    precision,
    quantity::Color,
    standard::Handle,
    tcollection::{AsciiString, HAsciiString},
};
use rustycad_tkservice::graphic3d::Graphic3dAspectsTextMembers;
/// Native text appearance; `cube.set_font(&AsciiString::from("sans-serif"))`.
pub trait AisViewCubeTextMembers: AisViewCubePresentationSource {
    /// Bind the actual text-aspect color field; `cube.text_color_ref().value()`.
    fn text_color_ref(&self) -> AisViewCubeColorRef<'_> {
        AisViewCubeColorRef::text(
            &self.attributes().text_aspect().aspect(),
            self.view_cube_presentation_base(),
        )
    }
    /// Set color and synchronize even when equal; `cube.set_text_color(red)`.
    fn set_text_color(&self, color: Color) {
        self.attributes().text_aspect().set_color(color);
        self.synchronize_aspects();
    }
    /// Bind the chosen HAsciiString payload or native static empty string; `cube.font_ref().value()`.
    fn font_ref(&self) -> AisViewCubeFontRef<'_> {
        AisViewCubeFontRef::new(
            &self.attributes().text_aspect().aspect().text_font(),
            self.view_cube_presentation_base(),
        )
    }
    /// Replace TextFont, preserving exact AsciiString bytes, and synchronize; `cube.set_font(&name)`.
    fn set_font(&self, font: &AsciiString) {
        let retained = if font.is_empty() {
            Handle::default()
        } else {
            Handle::new(HAsciiString::from(font.clone()))
        };
        self.attributes()
            .text_aspect()
            .aspect()
            .set_text_font(retained);
        self.synchronize_aspects();
    }
    /// Read font height; `cube.font_height()`.
    fn font_height(&self) -> f64 {
        self.attributes().text_aspect().height()
    }
    /// Mark presentation stale only beyond Confusion; `cube.set_font_height(16.0)`.
    fn set_font_height(&self, value: f64) {
        let text = self.attributes().text_aspect();
        if (text.height() - value).abs() > precision::confusion() {
            text.set_height(value);
            self.set_to_update_all();
        }
    }
}
impl<T: AisViewCubePresentationSource + ?Sized> AisViewCubeTextMembers for T {}
