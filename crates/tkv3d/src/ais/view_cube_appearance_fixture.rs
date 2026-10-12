//! Named context/recorder observes native retained property and cache lifecycle without GPU I/O.
use super::{color_scale_recording::*, view_cube_interaction_fixture::WidgetScene, *};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*, v3d::*};
use rustycad_tkernel::standard::SharedState;
use std::sync::Arc;
pub(super) const APPEARANCE: &str =
    include_str!("../../tests/fixtures/view-cube-appearance-state.txt");
pub(super) struct CubeAppearanceScene {
    pub(super) scene: WidgetScene,
    pub(super) trace: ScaleDrawTrace,
}
impl CubeAppearanceScene {
    pub(super) fn new() -> Self {
        let scene = WidgetScene::new(false);
        scene.context.display(&scene.cube.cast(), false).unwrap();
        let trace = Arc::new(SharedState::new(Vec::new()));
        install_scale_recording(&scene.cube.presentations()[0].cast(), &trace, true);
        Self { scene, trace }
    }
    pub(super) fn clear_events(&self) {
        self.scene.cube.presentations()[0].set_update_status(false);
        self.trace.write().clear();
    }
}
pub(super) fn assert_cube_appearance(scene: &CubeAppearanceScene, row: &[&str]) {
    let values = cube_appearance_numeric(scene);
    assert_eq!(values.len(), 38);
    for (index, (actual, expected)) in values.iter().zip(&row[1..39]).enumerate() {
        let expected = expected.parse::<f64>().unwrap();
        if index < 11 {
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "geometry channel {index}"
            );
        }
        assert!(
            (actual - expected).abs() < 1e-6,
            "appearance channel {index}: actual={actual}; expected={expected}; case={}",
            row[0]
        );
    }
    assert_cube_appearance_strings(&scene.scene.cube, &row[39..]);
    assert_cube_array_counts(scene, row);
}
fn assert_cube_array_counts(scene: &CubeAppearanceScene, row: &[&str]) {
    let counts = cube_appearance_array_counts(&scene.trace);
    assert_eq!(
        counts.to_vec(),
        row[row.len() - 3..]
            .iter()
            .map(|v| v.parse::<usize>().unwrap())
            .collect::<Vec<_>>()
    );
}
fn cube_appearance_numeric(scene: &CubeAppearanceScene) -> Vec<f64> {
    let cube = &scene.scene.cube;
    let mut values = cube_geometry_channels(cube);
    values.extend(
        [
            cube.to_draw_axes(),
            cube.to_draw_edges(),
            cube.to_draw_vertices(),
            cube.is_y_up(),
        ]
        .map(|value| value as i32 as f64),
    );
    values.extend(cube_appearance_cache_channels(scene));
    values.extend(cube_appearance_style_channels(cube));
    values
}
fn cube_geometry_channels(cube: &AisViewCube) -> Vec<f64> {
    vec![
        cube.size(),
        cube.box_facet_extension(),
        cube.axes_padding(),
        cube.box_edge_gap(),
        cube.box_edge_min_size(),
        cube.box_corner_min_size(),
        cube.round_radius(),
        cube.axes_radius(),
        cube.axes_cone_radius(),
        cube.axes_sphere_radius(),
        cube.font_height(),
    ]
}
fn assert_cube_appearance_strings(cube: &AisViewCube, row: &[&str]) {
    let mut strings = vec![cube.font_ref().value()];
    strings.extend(V3dTypeOfOrientation::ALL.map(|side| cube.box_side_label(side)));
    strings.extend(
        Prs3dDatumParts::DRAWN
            .into_iter()
            .chain([Prs3dDatumParts::None])
            .map(|part| cube.axis_label(part)),
    );
    for (value, pair) in strings.iter().zip(row.as_chunks::<2>().0) {
        assert_eq!(value.length(), pair[0].parse::<i32>().unwrap());
        let encoded = value
            .as_bytes()
            .iter()
            .take(value.length() as usize)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(if encoded.is_empty() { "-" } else { &encoded }, pair[1]);
    }
}
fn cube_appearance_array_counts(trace: &ScaleDrawTrace) -> [usize; 3] {
    let mut counts = [0, 0, 0];
    for request in trace.read().iter() {
        if let ScaleDrawRequest::Array {
            vertices, indices, ..
        } = request
        {
            counts[0] += 1;
            counts[1] += vertices.len();
            counts[2] += indices.len();
        }
    }
    counts
}

fn cube_appearance_style_channels(cube: &AisViewCube) -> Vec<f64> {
    let mut values = Vec::new();
    values.extend([
        cube.has_color() as i32 as f64,
        cube.has_material() as i32 as f64,
        cube.transparency(),
        cube.box_transparency(),
    ]);
    values.extend(cube_appearance_color_channels(cube));
    values
}
fn cube_appearance_color_channels(cube: &AisViewCube) -> Vec<f64> {
    let mut values = Vec::new();
    for color in [
        cube.box_color_ref().value(),
        cube.box_edge_style_ref()
            .clone_handle()
            .color(rustycad_tkservice::aspect::AspectTypeOfFacingModel::FrontSide),
        cube.box_corner_style_ref()
            .clone_handle()
            .color(rustycad_tkservice::aspect::AspectTypeOfFacingModel::FrontSide),
        cube.inner_color_ref().value(),
        cube.text_color_ref().value(),
    ] {
        values.extend([color.red(), color.green(), color.blue()]);
    }
    values
}

fn cube_appearance_cache_channels(scene: &CubeAppearanceScene) -> [f64; 4] {
    let cube = &scene.scene.cube;
    let synchronized = scene
        .trace
        .read()
        .iter()
        .filter(|event| matches!(event, ScaleDrawRequest::Synchronize))
        .count();
    [
        cube.presentations()[0].must_be_updated() as i32 as f64,
        synchronized as f64,
        cube.selection(0).entities().len() as f64,
        cube.selection(0).update_status() as i32 as f64,
    ]
}
