//! Native programmable versus generated-compatible coordinates use actual Texture2Dplane handles.
use super::texture_scene::{geometry, projective};
use super::{scene::ViewerScene, texture_scene::texture_image};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use rustycad_tkservice::{aspect::AspectInteriorStyle, graphic3d::*};
pub struct TextureGenerationScene {
    pub scene: ViewerScene,
    pub texture: Handle<Graphic3dTexture2Dplane>,
}
impl TextureGenerationScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let scene = ViewerScene::new(driver);
        scene.group.clear(true);
        let texture = generation_texture(mode);
        let aspect = generation_aspect(&texture);
        scene.group.set_group_primitives_aspect(&aspect.cast());
        generation_geometry(&*scene.group, mode);
        configure_generation_view(&scene, mode);
        shared_generation_groups(&scene, mode);
        Self { scene, texture }
    }
    pub fn mutate(&self, mode: &str) {
        if mode == "eye-params-updated" {
            self.texture.get_params().set_gen_mode(
                Graphic3dTypeOfTextureMode::Eye,
                Vec4::new(0.03125, 0., 0., 0.25),
                Vec4::new(0., 0.03125, 0., 0.25),
            );
            self.scene.group.synchronize_aspects();
            return;
        }
        if !mode.contains("updated") {
            return;
        }
        self.texture.set_plane_s(0.03125, 0., 0., 0.25);
        self.texture.set_plane_t(0., 0.03125, 0., 0.25);
        self.scene.group.synchronize_aspects();
    }
}
fn generation_texture(mode: &str) -> Handle<Graphic3dTexture2Dplane> {
    let texture = Handle::new(Graphic3dTexture2Dplane::from_pix_map(texture_image(
        "nearest",
    )));
    let parameters = texture.get_params();
    parameters.set_filter(Graphic3dTypeOfTextureFilter::Nearest);
    let generation = if mode.starts_with("object") {
        Graphic3dTypeOfTextureMode::Object
    } else if mode.starts_with("eye") {
        Graphic3dTypeOfTextureMode::Eye
    } else {
        Graphic3dTypeOfTextureMode::Manual
    };
    parameters.set_gen_mode(
        generation,
        Vec4::new(0.0625, 0., 0.0125, 0.5),
        Vec4::new(0., 0.0625, 0., 0.5),
    );
    texture
}
fn generation_aspect(texture: &Handle<Graphic3dTexture2Dplane>) -> Handle<Graphic3dAspects> {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_interior_style(AspectInteriorStyle::Solid);
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspect.set_polygon_offsets(0, 0., 0.);
    aspect.set_interior_color(
        rustycad_tkernel::quantity::Color::new(
            1.,
            1.,
            1.,
            rustycad_tkernel::quantity::ColorSpace::Rgb,
        )
        .unwrap(),
    );
    aspect.set_texture_map(texture.cast());
    aspect.set_texture_map_on(true);
    aspect
}
fn generation_geometry(group: &dyn Graphic3dGroup, mode: &str) {
    if mode.contains("-q") {
        projective::projective_surface_group(group);
        return;
    }
    if !mode.contains("no-uv") {
        group.add_primitive_array(&geometry::surface_texture_quad("nearest"), true);
        return;
    }
    let array = generation_untextured_array();
    for point in [[-8., -8., 0.], [8., -8., 0.], [8., 8., 0.], [-8., 8., 0.]] {
        array.add_vertex_normal(point, [0., 0., 1.]).unwrap();
    }
    for index in [1, 2, 3, 1, 3, 4] {
        array.add_edge(index).unwrap();
    }
    group.add_primitive_array(&array, true);
}
fn generation_untextured_array() -> Handle<Graphic3dArrayOfPrimitives> {
    Handle::new(
        Graphic3dArrayOfPrimitives::new(
            Graphic3dTypeOfPrimitiveArray::Triangles,
            4,
            0,
            6,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
        )
        .unwrap(),
    )
}
fn configure_generation_view(scene: &ViewerScene, mode: &str) {
    if mode.contains("moved") {
        scene
            .group
            .set_transformation(&Transform3::translation_vector(Vector3::new(2., 1., 1.)));
    }
    if mode.contains("camera") {
        scene
            .view
            .camera()
            .set_eye(&Point3::new(2., 1., 20.))
            .unwrap();
        scene
            .view
            .camera()
            .set_center(&Point3::new(2., 1., 0.))
            .unwrap();
    }
}
fn shared_generation_groups(scene: &ViewerScene, mode: &str) {
    if !mode.contains("shared") {
        return;
    }
    let aspect = scene.group.aspects();
    scene
        .group
        .set_transformation(&Transform3::translation_vector(Vector3::new(-4., 0., 0.)));
    if mode.contains("reset") {
        let middle = scene.structure.new_group().unwrap();
        middle.set_group_primitives_aspect(&super::scene::unlit_style([1.; 4]));
        generation_geometry(&*middle, "manual-uv");
        middle.set_transformation(&Transform3::translation_vector(Vector3::new(100., 0., 0.)));
    }
    let second = scene.structure.new_group().unwrap();
    second.set_group_primitives_aspect(&generation_second_aspect(&aspect, mode));
    generation_geometry(&*second, mode);
    second.set_transformation(&Transform3::translation_vector(Vector3::new(4., 0., 0.)));
}
fn generation_second_aspect(
    first: &Handle<dyn Graphic3dAspectsSource>,
    mode: &str,
) -> Handle<dyn Graphic3dAspectsSource> {
    if !mode.contains("revision") {
        return first.clone();
    }
    let texture = generation_texture(mode);
    texture.texture_root_base().fields.write().tex_id = first.texture_map().get_id();
    texture.update_revision();
    generation_aspect(&texture).cast()
}
