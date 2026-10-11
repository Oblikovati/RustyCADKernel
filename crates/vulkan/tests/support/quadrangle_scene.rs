#![allow(dead_code)]
//! Paired native controls expose quad diagonals, strip winding and unordered alpha composition.
use crate::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    quantity::{Color, ColorRgba, ColorSpace},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;
pub const QUAD_MODES: [&str; 11] = [
    "quad-gradient",
    "quad-indexed",
    "quad-strips",
    "quad-bounds",
    "quad-transparent",
    "quad-material-only",
    "quad-layers",
    "quad-clear-depth",
    "quad-occlusion",
    "quad-overlap",
    "quad-alpha-opaque",
];
pub struct QuadrangleScene {
    pub scene: ViewerScene,
    pub occluder: Handle<Graphic3dStructure>,
}
impl QuadrangleScene {
    /// Build an analytic assembly control; `QuadrangleScene::new(&driver,"quad-gradient",0.)`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str, offset: f64) -> Self {
        let viewer_scene = ViewerScene::new(driver);
        let occluder =
            Graphic3dStructure::new(&viewer_scene.viewer.structure_manager(), &Handle::default())
                .unwrap();
        if mode == "quad-layers" || mode == "quad-clear-depth" {
            let mut settings = Graphic3dZLayerSettings::default();
            settings.set_clear_depth(mode == "quad-clear-depth");
            let layer = viewer_scene.viewer.add_z_layer(&settings).unwrap().unwrap();
            occluder.set_z_layer(layer);
        }
        let scene = Self {
            scene: viewer_scene,
            occluder,
        };
        scene.rebuild(mode, offset);
        scene
    }
    /// Replace live geometry without replacing the view; `scene.rebuild("quad-overlap",0.)`.
    pub fn rebuild(&self, mode: &str, offset: f64) {
        let group = &self.scene.group;
        group.clear(false);
        group.set_primitives_aspect(&primary_quad_aspect(mode));
        group.add_primitive_array(&quad_vertices(mode, offset, 0.), true);
        self.add_overlap(mode, offset);
        self.add_layer_occluder(mode, offset);
    }
    fn add_overlap(&self, mode: &str, offset: f64) {
        if mode != "quad-occlusion" && mode != "quad-overlap" {
            return;
        }
        self.scene.group.set_primitives_aspect(&quad_aspect(
            [1., 0., 0.],
            if mode == "quad-overlap" { 0.5 } else { 0. },
            false,
        ));
        self.scene
            .group
            .add_primitive_array(&quad_vertices("quad-transparent", offset, 1.), true);
    }
    fn add_layer_occluder(&self, mode: &str, offset: f64) {
        self.occluder.clear(false);
        if mode != "quad-layers" && mode != "quad-clear-depth" {
            return;
        }
        let group = self.occluder.new_group().unwrap();
        group.set_primitives_aspect(&quad_aspect([1., 0., 0.], 0., false));
        group.add_primitive_array(&quad_vertices("quad-transparent", offset, -1.), true);
        self.occluder.display();
    }
}
fn primary_quad_aspect(mode: &str) -> Handle<dyn Graphic3dAspectsSource> {
    let transparent = matches!(
        mode,
        "quad-transparent"
            | "quad-material-only"
            | "quad-occlusion"
            | "quad-overlap"
            | "quad-alpha-opaque"
            | "quad-layers"
            | "quad-clear-depth"
    );
    let aspect = quad_aspect(
        [0., 1., 0.],
        if transparent { 0.5 } else { 0. },
        mode == "quad-alpha-opaque",
    );
    if mode == "quad-material-only" {
        aspect.set_interior_color_rgba(ColorRgba::from_rgb(aspect.interior_color(), 1.));
    }
    aspect
}
fn quad_aspect(
    rgb: [f64; 3],
    transparency: f32,
    opaque: bool,
) -> Handle<dyn Graphic3dAspectsSource> {
    let style = Handle::new(Graphic3dAspects::default());
    style.set_polygon_offsets(0, 0., 0.);
    style.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    style.set_interior_color(Color::new(rgb[0], rgb[1], rgb[2], ColorSpace::Rgb).unwrap());
    style.set_interior_color_rgba(ColorRgba::from_rgb(
        style.interior_color(),
        1. - transparency,
    ));
    let mut material = style.front_material();
    material.set_transparency(transparency).unwrap();
    style.set_front_material(material);
    if opaque {
        style.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
    }
    style.cast()
}
const QUAD_STRIP_POINTS: [[f32; 2]; 6] = [
    [-6., -4.],
    [-6., 4.],
    [0., -4.],
    [0., 4.],
    [6., -4.],
    [6., 4.],
];
const QUAD_BOUND_POINTS: [[f32; 2]; 8] = [
    [-7., -4.],
    [-7., 4.],
    [-1., -4.],
    [-1., 4.],
    [1., -4.],
    [1., 4.],
    [7., -4.],
    [7., 4.],
];
const QUAD_RECTANGLE_POINTS: [[f32; 2]; 4] = [[-6., -4.], [6., -4.], [6., 4.], [-6., 4.]];
fn quad_points(mode: &str, z: f32) -> Vec<[f32; 3]> {
    let points: &[[f32; 2]] = match mode {
        "quad-strips" => &QUAD_STRIP_POINTS,
        "quad-bounds" => &QUAD_BOUND_POINTS,
        _ => &QUAD_RECTANGLE_POINTS,
    };
    points.iter().map(|[x, y]| [*x, *y, z]).collect()
}
fn quad_vertices(mode: &str, offset: f64, z: f32) -> Handle<Graphic3dArrayOfPrimitives> {
    let points = quad_points(mode, z);
    let colored = mode == "quad-gradient" || mode == "quad-strips";
    let array = allocate_quad_array(mode, points.len());
    append_quad_vertices(&array, &points, offset, colored);
    if mode == "quad-indexed" {
        for index in [4, 3, 2, 1] {
            array.add_edge(index).unwrap();
        }
    }
    if mode == "quad-bounds" {
        array.add_bound_rgb(4, 1., 0., 0.).unwrap();
        array.add_bound_rgb(4, 0., 1., 0.).unwrap();
    }
    array
}
fn allocate_quad_array(mode: &str, count: usize) -> Handle<Graphic3dArrayOfPrimitives> {
    let kind = if mode == "quad-strips" || mode == "quad-bounds" {
        Graphic3dTypeOfPrimitiveArray::Quadranglestrips
    } else {
        Graphic3dTypeOfPrimitiveArray::Quadrangles
    };
    let flags = quad_color_flags(mode);
    Handle::new(
        Graphic3dArrayOfPrimitives::new(
            kind,
            count as i32,
            if mode == "quad-bounds" { 2 } else { 0 },
            if mode == "quad-indexed" { 4 } else { 0 },
            flags,
        )
        .unwrap(),
    )
}
fn quad_color_flags(mode: &str) -> i32 {
    match mode {
        "quad-gradient" | "quad-strips" => GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR,
        "quad-bounds" => GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR,
        _ => 0,
    }
}
fn append_quad_vertices(
    array: &Graphic3dArrayOfPrimitives,
    points: &[[f32; 3]],
    offset: f64,
    colored: bool,
) {
    let colors = [[1., 0., 0.], [0., 1., 0.], [1., 1., 1.], [0., 0., 1.]];
    for (index, [x, y, z]) in points.iter().enumerate() {
        let rank = array
            .add_vertex((*x as f64 + offset) as f32, *y, *z)
            .unwrap();
        if colored {
            let [r, g, b] = colors[index % 4];
            array.set_vertex_color_rgb(rank, r, g, b).unwrap();
        }
    }
}
