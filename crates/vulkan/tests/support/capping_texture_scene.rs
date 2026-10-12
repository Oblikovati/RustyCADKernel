//! Actual section planes select retained images and native cap alpha policies.
use super::{scene, scene::ViewerScene, texture_scene::texture_handle};
#[path = "capping_edge_boundaries.rs"]
mod edge_boundaries;
#[path = "capping_scene.rs"]
mod section_geometry;
#[path = "capping_section_mesh.rs"]
mod section_mesh;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    ncollection::{Vec2, Vec4},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;
pub struct CappingTextureScene {
    pub scene: ViewerScene,
    pub plane: Handle<Graphic3dClipPlane>,
    pub texture: Handle<Graphic3dTexture2D>,
}
impl CappingTextureScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let scene = ViewerScene::new(driver);
        scene.group.clear(false);
        scene
            .group
            .set_group_primitives_aspect(&super::scene::unlit_style([1., 0., 0., 1.]));
        section_geometry::cube(&*scene.group, 0.);
        let plane = section_geometry::cap_plane(cap_fixture_equation(mode));
        let texture = cap_texture(mode);
        configure_cap_aspect(&plane, &texture, mode);
        cap_object_properties(&scene, &plane, mode);
        let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        planes.append(plane.clone());
        scene.view.set_clip_planes(&planes);
        scene.structure.calculate_bound_box();
        let fixture = Self {
            scene,
            plane,
            texture,
        };
        edge_boundaries::install_cap_edge_boundary(&fixture, mode);
        fixture
    }
    pub fn mutate(&self, mode: &str) {
        refresh_cap_fixture_content(self, mode);
        if mode.contains("line-synchronized") {
            self.scene.group.synchronize_aspects();
        }
        if mode.contains("sampler-updated") {
            self.texture
                .get_params()
                .set_filter(Graphic3dTypeOfTextureFilter::Bilinear);
        }
        if mode.contains("plane-updated") {
            self.plane
                .set_equation(Vec4::new(0.6, 0., -0.8, -1.))
                .unwrap();
        }
        if mode.contains("alpha-updated") {
            self.plane
                .capping_aspect()
                .set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
        }
    }
}
fn refresh_cap_fixture_content(fixture: &CappingTextureScene, mode: &str) {
    if mode.contains("image-updated") || mode.contains("image-refreshed") {
        super::texture_scene::updates::mutate_texture_content(&fixture.texture, "image-updated");
    }
    if mode.contains("image-refreshed") {
        fixture
            .plane
            .set_capping_texture(fixture.texture.cast())
            .unwrap();
    }
}
fn cap_texture(mode: &str) -> Handle<Graphic3dTexture2D> {
    let texture = texture_handle(cap_image_mode(mode));
    texture
        .get_params()
        .set_filter(Graphic3dTypeOfTextureFilter::Nearest);
    texture.get_params().set_scale(Vec2::new(0.0625, 0.0625));
    if mode.contains("no-modulate") {
        texture.get_params().set_modulate(false);
    }
    configure_generated_cap_planes(&texture, mode);
    texture
}
fn configure_generated_cap_planes(texture: &Graphic3dTexture2D, mode: &str) {
    if mode.contains("generated") || mode.contains("shader-eye") {
        texture.get_params().set_gen_mode(
            if mode.contains("eye") {
                Graphic3dTypeOfTextureMode::Eye
            } else {
                Graphic3dTypeOfTextureMode::Object
            },
            Vec4::new(1., 0., 0., 0.),
            Vec4::new(0., 0., -1., 0.),
        );
    }
}
fn configure_cap_aspect(
    plane: &Graphic3dClipPlane,
    texture: &Handle<Graphic3dTexture2D>,
    mode: &str,
) {
    let aspect = plane.capping_aspect();
    aspect.set_texture_map(texture.cast());
    aspect.set_texture_map_on(!mode.contains("unmapped"));
    aspect.set_interior_color(rustycad_tkernel::quantity::Color::named(
        rustycad_tkernel::quantity::NamedColor::WHITE,
    ));
    let alpha = match cap_image_mode(mode) {
        "mask" | "mask-exact" => Graphic3dAlphaMode::Mask,
        "mask-blend" => Graphic3dAlphaMode::MaskBlend,
        "blend" => Graphic3dAlphaMode::Blend,
        "opaque-alpha" => Graphic3dAlphaMode::Opaque,
        _ => Graphic3dAlphaMode::BlendAuto,
    };
    aspect.set_alpha_mode(alpha, 0.5);
    section_mesh::configure_section_mesh_edges(&*aspect, mode);
}

fn cap_image_mode(mode: &str) -> &str {
    for name in [
        "format-alpha-f",
        "format-alpha",
        "format-gray",
        "format-rgb",
        "bc1-rgb",
        "bc1-rgba",
        "bc2",
        "bc3",
        "mask-exact",
        "mask-blend",
        "mask",
        "blend",
        "opaque-alpha",
    ] {
        if mode.contains(name) {
            return name;
        }
    }
    "nearest"
}
fn cap_object_properties(scene: &ViewerScene, plane: &Graphic3dClipPlane, mode: &str) {
    let group = scene.group.aspects();
    if mode.contains("object-texture") || mode.contains("source-mask") {
        let texture = cap_texture(if mode.contains("source-mask") {
            "mask"
        } else {
            "nearest"
        });
        texture.get_params().set_scale(Vec2::new(1., 1.));
        group.set_texture_map(texture.cast());
        group.set_texture_map_on(true);
        plane.set_use_object_texture(mode.contains("object-texture"));
        if mode.contains("source-mask") {
            group.set_alpha_mode(Graphic3dAlphaMode::Mask, 0.5);
        }
    }
    if cap_fixture_is_lit(mode) || mode.contains("object-material") {
        configure_cap_materials(scene, plane, mode);
    }
}
use super::texture_scene::lighting;
fn configure_cap_materials(scene: &ViewerScene, plane: &Graphic3dClipPlane, mode: &str) {
    if cap_fixture_is_lit(mode) {
        lighting::texture_lighting(scene, &*plane.capping_aspect());
        plane
            .capping_aspect()
            .set_shading_model(if mode.contains("gouraud") {
                Graphic3dTypeOfShadingModel::Gouraud
            } else if mode.contains("facet") {
                Graphic3dTypeOfShadingModel::PhongFacet
            } else {
                Graphic3dTypeOfShadingModel::Phong
            });
    }
    configure_inherited_cap_material(scene, plane, mode);
}

fn configure_inherited_cap_material(scene: &ViewerScene, plane: &Graphic3dClipPlane, mode: &str) {
    if mode.contains("object-material") {
        let group = Handle::new(Graphic3dAspects::default());
        lighting::texture_lighting(scene, &*group);
        let mut material = group.front_material();
        material.set_transparency(0.4).unwrap();
        scene.group.aspects().set_front_material(material);
        scene
            .group
            .aspects()
            .set_back_material(group.back_material());
        scene
            .group
            .aspects()
            .set_interior_color(group.interior_color());
        plane.set_use_object_material(true);
    }
}
fn cap_fixture_equation(mode: &str) -> [f64; 4] {
    if mode.contains("oblique") {
        return [0.6, 0., -0.8, -2.];
    }
    [0., 0., -1., 0.]
}

fn cap_fixture_is_lit(mode: &str) -> bool {
    mode.contains("phong") || mode.contains("gouraud") || mode.contains("facet")
}

#[cfg(test)]
mod tests {
    use super::cap_fixture_equation;
    #[test]
    fn compound_oblique_policies_keep_the_actual_native_plane() {
        for mode in [
            "oblique",
            "oblique-mask",
            "object-generated-oblique",
            "fixed-oblique",
            "oblique-image-updated",
            "oblique-mask-blend-alpha-updated",
        ] {
            assert_eq!(cap_fixture_equation(mode), [0.6, 0., -0.8, -2.], "{mode}");
        }
        assert_eq!(cap_fixture_equation("mask"), [0., 0., -1., 0.]);
    }
}
