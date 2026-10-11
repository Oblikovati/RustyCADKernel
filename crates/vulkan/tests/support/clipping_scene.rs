//! Paired with martensite_vulkan_clipping.cxx: retained planes, not direct GPU submissions.
use super::scene::ViewerScene;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
pub const MODES: [&str; 27] = [
    "clip-global",
    "clip-intersection",
    "clip-union",
    "clip-two-chains",
    "clip-local",
    "clip-override",
    "clip-empty-override",
    "clip-head-off",
    "clip-child-off",
    "clip-equation-updated",
    "clip-cleared",
    "clip-group-transform",
    "clip-lines",
    "clip-points",
    "clip-outside",
    "clip-inside",
    "clip-pers-in",
    "clip-pers-out",
    "clip-pers-local",
    "clip-pers-override",
    "clip-pers-2d",
    "clip-group-pers",
    "clip-nested-pers",
    "clip-text-baseline",
    "clip-text-cut",
    "clip-text-outside",
    "clip-text-no-box",
];
fn clip_plane(x: f64, y: f64, offset: f64) -> Handle<Graphic3dClipPlane> {
    Handle::new(Graphic3dClipPlane::from_equation(Vec4::new(x, y, 0., offset)).unwrap())
}
fn single_plane(plane: Handle<Graphic3dClipPlane>) -> Handle<Graphic3dSequenceOfHClipPlane> {
    let sequence = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    sequence.append(plane);
    sequence
}
/// Set native-compatible clipping cases; `configure_clipping(&scene, "clip-union")`.
pub fn configure_clipping(scene: &ViewerScene, mode: &str) {
    assert!(
        MODES.contains(&mode),
        "received {mode}; expected a clipping capture mode"
    );
    let first = clip_plane(1., 0., 0.);
    let global = single_plane(first.clone());
    configure_chains(&global, &first, mode);
    scene.view.set_clip_planes(&global);
    configure_local(scene, mode);
    configure_geometry(scene, mode);
    persistence::configure(scene, mode);
    scene.structure.calculate_bound_box();
    update_live_planes(scene, &first, mode);
}
fn update_live_planes(scene: &ViewerScene, first: &Graphic3dClipPlane, mode: &str) {
    if mode == "clip-equation-updated" || mode == "clip-cleared" {
        scene.render_view().try_redraw().unwrap();
    }
    if mode == "clip-equation-updated" {
        first.set_equation(Vec4::new(1., 0., 0., -2.)).unwrap();
    }
    if mode == "clip-cleared" {
        scene.view.set_clip_planes(&Handle::default());
    }
}
fn configure_chains(
    sequence: &Graphic3dSequenceOfHClipPlane,
    first: &Handle<Graphic3dClipPlane>,
    mode: &str,
) {
    match mode {
        "clip-intersection" => {
            sequence.append(clip_plane(0., 1., 0.));
        }
        "clip-union" | "clip-two-chains" | "clip-child-off" => {
            let child = clip_plane(0., 1., 0.);
            child.set_on(mode != "clip-child-off").unwrap();
            Graphic3dClipPlane::set_chain_next_plane(first, child).unwrap();
        }
        "clip-head-off" => first.set_on(false).unwrap(),
        "clip-outside" => first.set_equation(Vec4::new(1., 0., 0., -20.)).unwrap(),
        "clip-inside" => first.set_equation(Vec4::new(1., 0., 0., 20.)).unwrap(),
        _ => {}
    }
    if mode == "clip-two-chains" {
        let second = clip_plane(-1., 0., 5.);
        Graphic3dClipPlane::set_chain_next_plane(&second, clip_plane(0., -1., 5.)).unwrap();
        sequence.append(second);
    }
}
fn configure_local(scene: &ViewerScene, mode: &str) {
    if !matches!(mode, "clip-local" | "clip-override" | "clip-empty-override") {
        return;
    }
    let local = single_plane(clip_plane(0., 1., 0.));
    local.set_override_global(mode != "clip-local");
    if mode == "clip-empty-override" {
        local.clear();
    }
    if mode == "clip-local" {
        scene.view.set_clip_planes(&Handle::default());
    }
    scene.structure.set_clip_planes(&local);
}
fn configure_geometry(scene: &ViewerScene, mode: &str) {
    if mode == "clip-group-transform" {
        scene
            .group
            .set_transformation(&Transform3::translation_vector(Vector3::new(2., 3., 0.)));
    }
    if mode != "clip-lines" && mode != "clip-points" {
        return;
    }
    let aspects = scene.group.aspects();
    aspects.set_marker_type(AspectTypeOfMarker::Plus);
    aspects.set_marker_scale(2.).unwrap();
    scene.group.clear(false);
    scene.group.set_group_primitives_aspect(&aspects);
    scene
        .group
        .add_primitive_array(&clipped_vertices(mode), true);
}
fn clipped_vertices(mode: &str) -> Handle<Graphic3dArrayOfPrimitives> {
    let kind = if mode == "clip-lines" {
        Graphic3dTypeOfPrimitiveArray::Segments
    } else {
        Graphic3dTypeOfPrimitiveArray::Points
    };
    let vertices = Handle::new(Graphic3dArrayOfPrimitives::new(kind, 6, 0, 0, 0).unwrap());
    for y in [-3., 3.] {
        for x in [-8., 0., 8.] {
            vertices.add_vertex(x, y, 0.).unwrap();
        }
    }
    vertices
}

#[path = "clipping_persistence.rs"]
mod persistence;
