//! Native reset ownership snapshots exclude dangling aliases by retaining every original owner.
use super::{
    view_cube_appearance_fixture::CubeAppearanceScene,
    view_cube_appearance_mutations::prepare_custom_appearance,
    view_cube_interaction_fixture::widget_rows, *,
};
use crate::{prs_mgr::*, prs3d::*};
use rustycad_tkernel::standard::Handle;
use std::sync::{Arc, atomic::Ordering};
const ALIASES: &str = include_str!("../../tests/fixtures/view-cube-appearance-aliases.txt");
struct CubeResetOwners {
    drawer: Handle<Prs3dDrawer>,
    side: Handle<Prs3dShadingAspect>,
    edge: Handle<Prs3dShadingAspect>,
    corner: Handle<Prs3dShadingAspect>,
    dynamic: Handle<Prs3dDrawer>,
    highlight: Handle<Prs3dShadingAspect>,
    ordinary: Handle<Prs3dDrawer>,
    animation: Handle<dyn AisAnimationCameraSource>,
    persistence: Handle<dyn rustycad_tkservice::graphic3d::Graphic3dTransformPersSource>,
    start: Handle<rustycad_tkservice::graphic3d::Graphic3dCamera>,
    end: Handle<rustycad_tkservice::graphic3d::Graphic3dCamera>,
}
impl CubeResetOwners {
    fn retain(cube: &AisViewCube) -> Self {
        let dynamic = cube.dynamic_hilight_attributes();
        let highlight = dynamic.shading_aspect();
        let fields = cube.interaction_base().fields.read();
        Self {
            drawer: cube.attributes(),
            side: cube.box_side_style_ref().clone_handle(),
            edge: cube.box_edge_style_ref().clone_handle(),
            corner: cube.box_corner_style_ref().clone_handle(),
            dynamic,
            highlight,
            ordinary: cube.hilight_attributes(),
            animation: fields.animation.clone(),
            persistence: cube.transform_persistence(),
            start: fields.start.clone(),
            end: fields.end.clone(),
        }
    }
    fn snapshot(&self, cube: &AisViewCube) -> Vec<i32> {
        let fields = cube.interaction_base().fields.read();
        vec![
            cube.attributes() == self.drawer,
            cube.box_side_style_ref().object_address() == self.side.object_address(),
            cube.box_edge_style_ref().object_address() == self.edge.object_address(),
            cube.box_corner_style_ref().object_address() == self.corner.object_address(),
            cube.dynamic_hilight_attributes() == self.dynamic,
            self.dynamic.shading_aspect() == self.highlight,
            cube.hilight_attributes() == self.ordinary,
            cube.hilight_attributes().is_null(),
            fields.animation == self.animation,
            cube.transform_persistence() == self.persistence,
            fields.start == self.start,
            fields.end == self.end,
        ]
        .into_iter()
        .map(i32::from)
        .collect()
    }
}
#[test]
fn native_resets_preserve_drawer_aspect_navigation_camera_and_callback_owners() {
    for (reset, row) in widget_rows(ALIASES, "reset-owners").enumerate() {
        replay_reset_owners(reset, &row);
    }
}
fn replay_reset_owners(reset: usize, row: &[&str]) {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    prepare_custom_appearance(cube);
    cube.set_hilight_attributes(&Handle::new(Prs3dDrawer::new()));
    let owners = CubeResetOwners::retain(cube);
    let events = retained_cube_events(cube);
    match reset {
        0 => cube.unset_attributes(),
        1 => cube.unset_hilight_attributes(),
        _ => cube.reset_styles().unwrap(),
    };
    assert_reset_owner_snapshot(&scene, &owners, row);
    assert_reset_callbacks(&scene, &events);
}
fn assert_reset_owner_snapshot(
    scene: &CubeAppearanceScene,
    owners: &CubeResetOwners,
    row: &[&str],
) {
    let cube = &scene.scene.cube;
    let mut actual = owners.snapshot(cube);
    actual.extend([
        scene.scene.events.frames.load(Ordering::SeqCst) as i32,
        scene.scene.events.finished.load(Ordering::SeqCst) as i32,
    ]);
    assert_eq!(
        actual,
        row[1..]
            .iter()
            .map(|value| value.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    );
}
fn assert_reset_callbacks(
    scene: &CubeAppearanceScene,
    events: &Arc<dyn AisViewCubeAnimationEvents>,
) {
    let cube = &scene.scene.cube;
    assert!(Arc::ptr_eq(events, &retained_cube_events(cube)));
    cube.on_after_animation();
    cube.on_animation_finished();
    assert_eq!(scene.scene.events.frames.load(Ordering::SeqCst), 1);
    assert_eq!(scene.scene.events.finished.load(Ordering::SeqCst), 1);
}

fn retained_cube_events(cube: &AisViewCube) -> Arc<dyn AisViewCubeAnimationEvents> {
    cube.interaction_base()
        .fields
        .read()
        .events
        .clone()
        .unwrap()
}
