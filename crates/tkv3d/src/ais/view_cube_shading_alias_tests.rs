//! Handle aliases bind actual local/drawer slots and keep old drawer references independent of replacement.
use super::{
    view_cube_appearance_fixture::CubeAppearanceScene, view_cube_interaction_fixture::widget_rows,
    *,
};
use crate::{prs_mgr::PrsMgrPresentableObject, prs_mgr::PrsMgrPresentableObjectMembers, prs3d::*};
use rustycad_tkernel::standard::Handle;
const ALIASES: &str = include_str!("../../tests/fixtures/view-cube-appearance-aliases.txt");
#[test]
fn native_side_edge_corner_handle_aliases_observe_the_bound_slot_without_ownership_inflation() {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    let drawer = cube.attributes();
    let original = cube.box_side_style_ref().clone_handle();
    let next = Handle::new(Prs3dShadingAspect::new().unwrap());
    let side = cube.box_side_style_ref();
    assert_shading_alias_row(&side, &cube.box_side_style_ref(), &original, &next, 0);
    drawer.set_shading_aspect(next.clone());
    assert_shading_alias_row(&side, &cube.box_side_style_ref(), &original, &next, 1);
    let replacement = Handle::new(Prs3dDrawer::new());
    replacement.set_shading_aspect(original.clone());
    cube.set_attributes(&replacement);
    assert_shading_alias_row(&side, &cube.box_side_style_ref(), &original, &next, 2);
    replay_local_shading_slots(cube, &next);
}
fn replay_local_shading_slots(cube: &AisViewCube, next: &Handle<Prs3dShadingAspect>) {
    let original = cube.box_edge_style_ref().clone_handle();
    let edge = cube.box_edge_style_ref();
    assert_shading_alias_row(&edge, &cube.box_edge_style_ref(), &original, next, 3);
    *cube.view_cube_presentation_base().edge_aspect.write() = next.clone();
    assert_shading_alias_row(&edge, &cube.box_edge_style_ref(), &original, next, 4);
    replay_corner_shading_slot(cube, next, &edge);
}
fn replay_corner_shading_slot(
    cube: &AisViewCube,
    next: &Handle<Prs3dShadingAspect>,
    edge: &AisViewCubeShadingRef<'_>,
) {
    let corner_original = cube.box_corner_style_ref().clone_handle();
    let corner = cube.box_corner_style_ref();
    assert_shading_alias_row(
        &corner,
        &cube.box_corner_style_ref(),
        &corner_original,
        next,
        5,
    );
    *cube.view_cube_presentation_base().corner_aspect.write() = next.clone();
    assert_shading_alias_row(
        &corner,
        &cube.box_corner_style_ref(),
        &corner_original,
        next,
        6,
    );
    assert!(!edge.is_same_reference(&corner));
}
fn assert_shading_alias_row(
    alias: &AisViewCubeShadingRef<'_>,
    current: &AisViewCubeShadingRef<'_>,
    original: &Handle<Prs3dShadingAspect>,
    next: &Handle<Prs3dShadingAspect>,
    step: i32,
) {
    let row = widget_rows(ALIASES, "shade")
        .find(|row| row[0].parse::<i32>().unwrap() == step)
        .unwrap();
    let actual = [
        alias.is_same_reference(current) as i32,
        (alias.object_address() == original.object_address()) as i32,
        (alias.object_address() == next.object_address()) as i32,
        (current.object_address() == alias.object_address()) as i32,
        original.ref_count(),
        next.ref_count(),
    ];
    assert_eq!(
        actual.to_vec(),
        row[1..]
            .iter()
            .map(|v| v.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    );
}
#[test]
fn shading_reference_callbacks_can_replace_null_and_restore_the_native_slot() {
    let cube = AisViewCube::new().unwrap();
    let alias = cube.box_edge_style_ref();
    let original = alias.clone_handle();
    alias.with_shading(|value| {
        assert!(std::ptr::eq(value.unwrap(), &*original));
        *cube.view_cube_presentation_base().edge_aspect.write() = Handle::default();
    });
    assert!(alias.is_null());
    assert!(alias.with_shading(|value| value.is_none()));
    *cube.view_cube_presentation_base().edge_aspect.write() = original;
    assert!(!alias.is_null());
}

#[test]
fn native_inherited_handle_alias_remains_in_parent_slot_when_child_overrides() {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    let original = cube.box_side_style_ref().clone_handle();
    let next = Handle::new(Prs3dShadingAspect::new().unwrap());
    let parent = Handle::new(Prs3dDrawer::new());
    parent.set_shading_aspect(original.clone());
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.link_to(&parent);
    cube.set_attributes(&drawer);
    let alias = cube.box_side_style_ref();
    assert_shading_alias_row(&alias, &cube.box_side_style_ref(), &original, &next, 7);
    drawer.set_shading_aspect(next.clone());
    assert_shading_alias_row(&alias, &cube.box_side_style_ref(), &original, &next, 8);
    parent.set_shading_aspect(next.clone());
    assert_shading_alias_row(&alias, &cube.box_side_style_ref(), &original, &next, 9);
    parent.set_shading_aspect(Handle::default());
    assert_shading_alias_row(&alias, &cube.box_side_style_ref(), &original, &next, 10);
    parent.set_shading_aspect(original.clone());
    assert_shading_alias_row(&alias, &cube.box_side_style_ref(), &original, &next, 11);
}
