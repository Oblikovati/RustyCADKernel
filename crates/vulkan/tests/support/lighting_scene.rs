//! Paired OCCT lighting fixtures use deliberately nontrivial coefficients and normals.
use super::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    quantity::{Color, ColorSpace},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;
#[path = "lighting_scene_alpha.rs"]
mod opacity_fixture;
pub struct LightingScene {
    pub base: ViewerScene,
    pub aspect: Handle<Graphic3dAspects>,
    pub vertices: Handle<Graphic3dArrayOfPrimitives>,
    pub light: Handle<Graphic3dCLight>,
}
impl LightingScene {
    /// Build the paired native fixture; `LightingScene::new(&driver, "lit-phong")`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let line = mode.starts_with("lit-line-");
        let alpha = mode.starts_with("lit-alpha-");
        let mode = lighting_fixture_mode(mode);
        let mode = mode.as_str();
        let base = ViewerScene::new(driver);
        let (aspect, vertices) = configure_lighting_group(&base, mode, line);
        if alpha {
            opacity_fixture::configure(&aspect, mode);
        }
        let light = configure_lights(&base, mode);
        configure_lighting_view(&base, mode);
        Self {
            base,
            aspect,
            vertices,
            light,
        }
    }
    /// Apply one live state change after the first redraw; `scene.mutate("lit-normal-updated")`.
    pub fn mutate(&self, mode: &str) {
        if mode.starts_with("lit-alpha-") {
            opacity_fixture::mutate(&self.aspect, mode);
        }
        let mode = lighting_fixture_mode(mode);
        let mode = mode.as_str();
        if mode == "lit-normal-updated" {
            self.vertices.set_vertex_normal(2, 1., 0., 0.).unwrap();
            self.vertices.attributes().invalidate().unwrap();
        }
        if mode == "lit-material-updated" {
            self.aspect.set_front_material(lighting_material(true));
        }
        if mode == "lit-light-updated" {
            self.light.set_direction_coords(-1., -1., -1.).unwrap();
        }
        if mode == "lit-light-off" {
            self.light.set_enabled(false);
        }
    }
}
fn rgb(values: [f32; 3]) -> Color {
    Color::new(
        values[0].into(),
        values[1].into(),
        values[2].into(),
        ColorSpace::Rgb,
    )
    .unwrap()
}
fn lighting_material(back: bool) -> Graphic3dMaterialAspect {
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
    material.set_material_type(Graphic3dTypeOfMaterial::Aspect);
    material.set_ambient_color(rgb([0.125, 0.25, 0.375]));
    material.set_diffuse_color(rgb(if back {
        [0.125, 0.5, 0.25]
    } else {
        [0.5, 0.25, 0.125]
    }));
    material.set_specular_color(rgb([0.25, 0.125, 0.0625]));
    material.set_emissive_color(rgb([0.015625, 0.03125, 0.0625]));
    material.set_shininess(0.25).unwrap();
    material
}
fn lighting_aspect(mode: &str) -> Handle<Graphic3dAspects> {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_polygon_offsets(0, 0., 0.);
    aspect.set_shading_model(match mode {
        "lit-facet" => Graphic3dTypeOfShadingModel::PhongFacet,
        "lit-gouraud" => Graphic3dTypeOfShadingModel::Gouraud,
        "lit-default" => Graphic3dTypeOfShadingModel::Default,
        _ => Graphic3dTypeOfShadingModel::Phong,
    });
    aspect.set_interior_color(rgb([0.75, 0.5, 0.25]));
    aspect.set_front_material(lighting_material(false));
    aspect.set_back_material(lighting_material(true));
    aspect.set_distinguish(mode == "lit-distinct-back");
    aspect.set_back_interior_color(rgb([0.25, 0.75, 0.5]));
    aspect.set_use_vertex_color_for_back_faces(mode != "lit-front-color-only");
    aspect
}
fn configure_lighting_group(
    base: &ViewerScene,
    mode: &str,
    line: bool,
) -> (Handle<Graphic3dAspects>, Handle<Graphic3dArrayOfPrimitives>) {
    base.group.clear(false);
    let aspect = lighting_aspect(mode);
    if line {
        configure_lit_line_aspect(&aspect, mode);
    }
    base.group.set_group_primitives_aspect(&aspect.cast());
    let vertices = lighting_vertices(mode, line);
    base.group.add_primitive_array(&vertices, true);
    (aspect, vertices)
}
fn configure_lit_line_aspect(aspect: &Graphic3dAspects, mode: &str) {
    aspect.set_line_width(3.).unwrap();
    if mode == "lit-patterned" {
        aspect.set_line_type(rustycad_tkservice::aspect::AspectTypeOfLine::Dash);
    }
}
fn lighting_vertices(mode: &str, line: bool) -> Handle<Graphic3dArrayOfPrimitives> {
    let kind = if line {
        Graphic3dTypeOfPrimitiveArray::Segments
    } else {
        Graphic3dTypeOfPrimitiveArray::Triangles
    };
    let vertices = Handle::new(
        Graphic3dArrayOfPrimitives::new(
            kind,
            3,
            0,
            if line { 6 } else { 3 },
            lighting_vertex_flags(mode),
        )
        .unwrap(),
    );
    fill_lighting_vertex_slots(&vertices, mode);
    fill_lighting_colors(&vertices, mode);
    fill_lighting_indices(&vertices, mode, line);
    vertices
}
fn fill_lighting_colors(vertices: &Graphic3dArrayOfPrimitives, mode: &str) {
    if !matches!(mode, "lit-colored" | "lit-front-color-only") {
        return;
    }
    for index in 1..=3 {
        vertices
            .set_vertex_color(index, rgb([0.5, 0.75, 0.25]))
            .unwrap();
    }
}
fn fill_lighting_indices(vertices: &Graphic3dArrayOfPrimitives, mode: &str, line: bool) {
    if line {
        for index in [1, 2, 2, 3, 3, 1] {
            vertices.add_edge(index).unwrap();
        }
        return;
    }
    let back = matches!(
        mode,
        "lit-back" | "lit-distinct-back" | "lit-front-color-only"
    );
    vertices
        .add_edges3(1, if back { 3 } else { 2 }, if back { 2 } else { 3 })
        .unwrap();
}
fn lighting_vertex_flags(mode: &str) -> Graphic3dArrayFlags {
    let mut flags = if mode == "lit-no-normals" {
        0
    } else {
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL
    };
    if mode == "lit-normal-updated" {
        flags |= GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_MUTABLE;
    }
    if matches!(mode, "lit-colored" | "lit-front-color-only") {
        flags |= GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR;
    }
    flags
}
fn fill_lighting_vertex_slots(vertices: &Graphic3dArrayOfPrimitives, mode: &str) {
    let points = [[-8., -8., 0.], [8., -8., 0.], [0., 8., 0.]];
    let normals = [[0., 0., 1.], [0.6, 0., 0.8], [0., 0.6, 0.8]];
    // Native mutable buffers start at full capacity; fill existing slots rather than append.
    for (index, (point, normal)) in points.into_iter().zip(normals).enumerate() {
        let rank = (index + 1) as i32;
        vertices
            .set_vertice(rank, point[0], point[1], point[2])
            .unwrap();
        if mode != "lit-no-normals" {
            vertices
                .set_vertex_normal(rank, normal[0], normal[1], normal[2])
                .unwrap();
        }
    }
}
pub(crate) fn configure_lights(base: &ViewerScene, mode: &str) -> Handle<Graphic3dCLight> {
    let light = lighting_source(mode);
    configure_light_parameters(&light, mode);
    let ambient = Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient));
    ambient.set_color(rgb([0.125, 0.25, 0.5]));
    ambient.set_intensity(2.).unwrap();
    let set = Handle::new(Graphic3dLightSet::default());
    set.add(&ambient).unwrap();
    if mode != "lit-ambient" {
        set.add(&light).unwrap();
    }
    base.cview().set_lights(&set);
    light
}
fn lighting_source(mode: &str) -> Handle<Graphic3dCLight> {
    let kind = match mode {
        "lit-point" | "lit-range" => Graphic3dTypeOfLightSource::Positional,
        "lit-spot" => Graphic3dTypeOfLightSource::Spot,
        _ => Graphic3dTypeOfLightSource::Directional,
    };
    let light = Handle::new(Graphic3dCLight::new(kind));
    light.set_color(rgb([0.75, 0.5, 0.25]));
    light
        .set_intensity(if mode == "lit-intensity" { 7. } else { 1. })
        .unwrap();
    if kind != Graphic3dTypeOfLightSource::Positional {
        light.set_direction_coords(0., 0., -1.).unwrap();
    }
    if kind != Graphic3dTypeOfLightSource::Directional {
        light.set_position_coords(0., 0., 12.).unwrap();
        light.set_attenuation(0.5, 0.125).unwrap();
    }
    light
}
fn configure_light_parameters(light: &Graphic3dCLight, mode: &str) {
    if mode == "lit-range" {
        light.set_range(14.).unwrap();
    }
    if mode == "lit-spot" {
        light.set_angle(0.75).unwrap();
        light.set_concentration(0.125).unwrap();
    }
    if mode.starts_with("lit-headlight") {
        light.set_headlight(true).unwrap();
    }
}

fn configure_lighting_view(base: &ViewerScene, mode: &str) {
    if mode == "lit-headlight-rotated" {
        base.view
            .camera()
            .set_direction(&rustycad_tkmath::gp::Direction3::new(0.4, 0., -1.).unwrap());
    }
    if mode == "lit-clipped" {
        let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        planes.append(Handle::new(
            Graphic3dClipPlane::from_equation(rustycad_tkernel::ncollection::Vec4::new(
                1., 0., 0., 0.,
            ))
            .unwrap(),
        ));
        base.cview().set_clip_planes(&planes);
    }
}

fn lighting_fixture_mode(mode: &str) -> String {
    mode.strip_prefix("lit-line-")
        .or_else(|| mode.strip_prefix("lit-alpha-"))
        .map_or_else(|| mode.to_owned(), |suffix| format!("lit-{suffix}"))
}
