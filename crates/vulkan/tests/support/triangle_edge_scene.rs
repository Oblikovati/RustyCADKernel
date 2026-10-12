//! Genuine retained fill aspects exercise same-pass mesh shaders and mapped line groups.
use super::texture_scene::SurfaceTextureScene;
#[path = "triangle_edge_geometry.rs"]
mod geometry;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
pub struct TriangleEdgeScene {
    pub fixture: SurfaceTextureScene,
}
impl TriangleEdgeScene {
    /// Feed live Graphic3d edge fields into the renderer; `TriangleEdgeScene::new(&driver,"solid")`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let fixture = SurfaceTextureScene::with_mode(driver, "nearest");
        configure_edge_aspect(&fixture, mode);
        install_edge_geometry(&fixture, mode);
        configure_edge_view(&fixture, mode);
        Self { fixture }
    }
}
fn configure_edge_aspect(fixture: &SurfaceTextureScene, mode: &str) {
    let aspect = &fixture.aspects;
    configure_edge_image(fixture, mode);
    aspect.set_draw_edges(true);
    aspect.set_edge_color_rgba(ColorRgba::new(0., 1., 1., edge_fixture_alpha(mode)).unwrap());
    aspect
        .set_edge_width(if mode.contains("wide") { 3. } else { 1. })
        .unwrap();
    aspect.set_skip_first_edge(mode.contains("skip-first"));
    configure_edge_material(aspect, mode);
    configure_edge_model(aspect, mode);
    configure_edge_visibility(aspect, mode);
}
fn configure_edge_image(fixture: &SurfaceTextureScene, mode: &str) {
    if mode.contains("mask") || mode.contains("blend") {
        fixture
            .texture
            .set_image(super::texture_scene::texture_image("mask"));
        fixture.texture.update_revision();
    }
    fixture
        .aspects
        .set_texture_map_on(!mode.contains("unmapped"));
    if mode.contains("no-modulate") {
        fixture.texture.get_params().set_modulate(false);
    }
}
fn edge_fixture_alpha(mode: &str) -> f32 {
    if mode.contains("negative-alpha") {
        return -0.25;
    }
    if mode.contains("alpha") {
        return 0.25;
    }
    if mode.contains("background") {
        return 0.05;
    }
    1.
}
fn configure_edge_material(aspect: &Graphic3dAspects, mode: &str) {
    if mode.contains("node-alpha") {
        aspect.set_interior_color_rgba(ColorRgba::new(0.75, 0.5, 0.25, 0.25).unwrap());
    }
    if mode.contains("mat-alpha") {
        let mut material = aspect.front_material();
        material.set_transparency(0.5).unwrap();
        aspect.set_front_material(material);
    }
    let alpha = if mode.contains("mask-blend") {
        Graphic3dAlphaMode::MaskBlend
    } else if mode.contains("mask") {
        Graphic3dAlphaMode::Mask
    } else if mode.contains("blend") {
        Graphic3dAlphaMode::Blend
    } else {
        return;
    };
    aspect.set_alpha_mode(alpha, 0.5);
}
fn configure_edge_model(aspect: &Graphic3dAspects, mode: &str) {
    for (name, model) in [
        ("phong", Graphic3dTypeOfShadingModel::Phong),
        ("gouraud", Graphic3dTypeOfShadingModel::Gouraud),
        ("facet", Graphic3dTypeOfShadingModel::PhongFacet),
    ] {
        if mode.contains(name) {
            aspect.set_shading_model(model);
        }
    }
    if mode.contains("dashed") {
        aspect.set_line_type(AspectTypeOfLine::Dash);
    }
}
fn configure_edge_visibility(aspect: &Graphic3dAspects, mode: &str) {
    if mode.contains("back-cull") {
        aspect.set_face_culling(Graphic3dTypeOfBackfacingModel::BackCulled);
    }
    if mode.contains("front-cull") {
        aspect.set_face_culling(Graphic3dTypeOfBackfacingModel::FrontCulled);
    }
}
fn install_edge_geometry(fixture: &SurfaceTextureScene, mode: &str) {
    let group = &fixture.scene.group;
    group.clear(true);
    group.set_group_primitives_aspect(&fixture.aspects.cast());
    if mode.contains("projective") {
        super::texture_scene::projective::projective_surface_group(&**group);
        return;
    }
    let array = if mode.contains("line") {
        geometry::edge_fixture_lines(mode)
    } else {
        geometry::edge_triangle_array(mode)
    };
    group.add_primitive_array(&array, true);
}
fn configure_edge_view(fixture: &SurfaceTextureScene, mode: &str) {
    let cview = fixture.scene.cview();
    let mut params = cview.rendering_params();
    if mode.contains("resolution-144") {
        params.resolution = 144;
    }
    if mode.contains("feather-two") {
        params.line_feather = 2.;
    }
    if mode.contains("feather-zero") {
        params.line_feather = 0.;
    }
    cview.change_rendering_params(&mut |current| *current = params.clone());
    for view_mode in ["perspective", "clipped"] {
        if mode.contains(view_mode) {
            super::texture_scene::texture_view::configure_texture_view(&fixture.scene, view_mode);
        }
    }
}
