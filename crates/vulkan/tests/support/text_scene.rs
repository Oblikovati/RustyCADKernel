#![allow(dead_code)]
//! Real FreeType glyph scenes through V3d and retained Graphic3d text commands.
use super::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub const SHADOW_MODES: [&str; 9] = [
    "text-shadow-basic",
    "text-shadow-rotated",
    "text-shadow-zoomable",
    "text-shadow-plane",
    "text-shadow-plane-local",
    "text-shadow-depth",
    "text-shadow-annotation",
    "text-shadow-opaque",
    "text-shadow-rgba",
];
pub const MODES: [&str; 23] = [
    "text-basic",
    "text-unicode",
    "text-centered",
    "text-top",
    "text-multiline",
    "text-tabs",
    "text-rotated",
    "text-zoomable",
    "text-plane",
    "text-plane-local",
    "text-color",
    "text-height",
    "text-default-height",
    "text-empty",
    "text-depth",
    "text-annotation",
    "text-resolution",
    "text-highlight",
    "text-opaque",
    "text-mask",
    "text-auto",
    "text-blend",
    "text-cutoff",
];
pub struct TextScene {
    pub scene: ViewerScene,
    pub text: Handle<Graphic3dText>,
    pub style: Handle<Graphic3dAspectText3d>,
}
impl TextScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>) -> Self {
        let scene = ViewerScene::new(driver);
        scene.group.clear(true);
        let style = Handle::new(Graphic3dAspectText3d::new(
            Color::named(NamedColor::WHITE),
            Some("DejaVu Sans"),
            1.,
            0.,
            AspectTypeOfStyleText::Normal,
            AspectTypeOfDisplayText::Normal,
        ));
        let text = Handle::new(Graphic3dText::new(16.));
        text.set_text_str("AV");
        text.set_position(&Point3::new(-5., 0., 0.));
        Self { scene, text, style }
    }
    pub fn apply(&self, mode: &str) {
        if self.apply_shadow_mode(mode) {
            return;
        }
        match mode {
            "text-opaque" => self.style.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.99),
            "text-mask" => self.style.set_alpha_mode(Graphic3dAlphaMode::Mask, 0.99),
            "text-auto" => self
                .style
                .set_alpha_mode(Graphic3dAlphaMode::BlendAuto, 0.99),
            "text-blend" => self.style.set_alpha_mode(Graphic3dAlphaMode::Blend, 0.99),
            "text-cutoff" => self
                .style
                .set_alpha_mode(Graphic3dAlphaMode::MaskBlend, 0.99),
            "text-unicode" => self.text.set_text_str("AΩБ"),
            "text-multiline" => self.text.set_text_str("A\nV"),
            "text-tabs" => self.text.set_text_str("A\tV"),
            "text-rotated" => self.style.set_text_angle(30.),
            "text-zoomable" => self.style.set_text_zoomable(true),
            "text-color" => self.style.set_interior_color(Color::named(NamedColor::RED)),
            "text-height" => self.text.set_height(24.),
            "text-default-height" => self.text.set_height(1.),
            "text-empty" => self.text.set_text_str(""),
            "text-resolution" => self
                .scene
                .view
                .change_rendering_params(&mut |p| p.resolution = 144),
            _ => self.geometric_mode(mode),
        }
        self.scene.group.set_primitives_aspect(&self.style.cast());
        self.scene.group.add_text(&self.text, true);
    }
    fn apply_shadow_mode(&self, mode: &str) -> bool {
        let Some(base) = mode.strip_prefix("text-shadow-") else {
            return false;
        };
        self.style
            .set_text_display_type(AspectTypeOfDisplayText::Shadow);
        self.style.set_edge_color(Color::named(NamedColor::RED));
        if base == "rgba" {
            self.style
                .set_edge_color_rgba(rustycad_tkernel::quantity::ColorRgba::from_rgb(
                    Color::named(NamedColor::RED),
                    0.5,
                ));
        }
        self.apply(&format!(
            "text-{}",
            if base == "rgba" { "basic" } else { base }
        ));
        true
    }
    fn geometric_mode(&self, mode: &str) {
        match mode {
            "text-basic" => {}
            "text-centered" => self.align(
                Graphic3dHorizontalTextAlignment::Center,
                Graphic3dVerticalTextAlignment::Center,
            ),
            "text-top" => self.align(
                Graphic3dHorizontalTextAlignment::Right,
                Graphic3dVerticalTextAlignment::Top,
            ),
            "text-plane" | "text-plane-local" => self.plane(mode == "text-plane"),
            "text-depth" | "text-annotation" => self.occlusion(mode == "text-annotation"),
            "text-highlight" => self.highlight(),
            _ => panic!("received {mode}; expected a text capture mode"),
        }
    }
    fn align(
        &self,
        horizontal: Graphic3dHorizontalTextAlignment,
        vertical: Graphic3dVerticalTextAlignment,
    ) {
        self.text.set_horizontal_alignment(horizontal);
        self.text.set_vertical_alignment(vertical);
    }
    fn plane(&self, own: bool) {
        self.text.set_orientation(
            &Axis2::new(
                Point3::new(-5., 0., 0.),
                Direction3::new(0., 1., 1.).unwrap(),
            )
            .unwrap(),
        );
        self.text.set_own_anchor_point(own);
    }
    fn occlusion(&self, annotation: bool) {
        self.scene
            .group
            .set_primitives_aspect(&super::scene::unlit_style([1., 0., 0., 1.]));
        self.scene
            .group
            .add_primitive_array(&self.scene.vertices, true);
        self.text.set_position(&Point3::new(-5., 0., -1.));
        if annotation {
            self.style.set_text_style(AspectTypeOfStyleText::Annotation);
        }
    }
    fn highlight(&self) {
        let style = Handle::new(Graphic3dPresentationAttributes::default());
        style.set_color(Color::named(NamedColor::GREEN));
        self.scene.structure.highlight(&style.cast(), true);
    }
}
