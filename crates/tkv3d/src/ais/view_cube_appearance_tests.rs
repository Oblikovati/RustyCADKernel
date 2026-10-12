//! Native displayed-widget mutations distinguish stale presentation, live style and cached selection.
use super::{
    view_cube_appearance_fixture::*, view_cube_appearance_mutations::*,
    view_cube_interaction_fixture::widget_rows, *,
};
use crate::{prs3d::*, select_mgr::*, v3d::*};
use rustycad_tkernel::tcollection::AsciiString;
#[test]
fn native_all_appearance_setters_getters_equal_tiny_and_custom_state_cases() {
    for row in widget_rows(APPEARANCE, "mutation") {
        let encoded = row[0].parse::<i32>().unwrap();
        let (id, pass) = (encoded / 3, encoded % 3);
        let scene = CubeAppearanceScene::new();
        let cube = &scene.scene.cube;
        let delta = match pass {
            0 => 0.0,
            1 => 1e-8,
            _ => 0.2,
        };
        if id >= 12 && pass == 2 {
            prepare_custom_appearance(cube);
        }
        if id >= 12 && pass == 1 {
            apply_appearance_mutation(cube, id, delta);
        }
        scene.clear_events();
        apply_appearance_mutation(cube, id, delta);
        assert_cube_appearance(&scene, &row);
    }
}
#[test]
fn native_arbitrary_and_missing_label_map_keys_and_y_up_remapping() {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    prepare_arbitrary_cube_labels(cube);
    for row in widget_rows(APPEARANCE, "maps") {
        match row[0] {
            "1" => cube.set_y_up(true, true),
            "2" => cube.set_y_up(false, true),
            _ => {}
        }
        assert_cube_appearance(&scene, &row);
    }
}
#[test]
fn native_geometry_updates_preserve_cached_selection_until_full_redisplay() {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    let original = cube.selection(0);
    let first = original.entities()[0].base_sensitive();
    cube.set_draw_edges(false);
    cube.set_draw_vertices(false);
    for row in widget_rows(APPEARANCE, "lifecycle") {
        advance_cube_appearance_lifecycle(&scene, row[0]);
        assert_cube_appearance(&scene, &row);
        if row[0] != "0" {
            assert_cached_appearance_selection(cube, &original, &first, row[0] == "2");
        }
    }
}
fn assert_cached_appearance_selection(
    cube: &AisViewCube,
    original: &rustycad_tkernel::standard::Handle<SelectMgrSelection>,
    first: &rustycad_tkernel::standard::Handle<dyn crate::select3d::Select3DSensitiveEntity>,
    updated: bool,
) {
    let expected = widget_rows(APPEARANCE, "cache")
        .find(|row| row[0] == if updated { "1" } else { "0" })
        .unwrap();
    assert_eq!(cube.selection(0) == *original, expected[1] == "1");
    assert_eq!(
        cube.selection(0).entities()[0].base_sensitive() == *first,
        expected[2] == "1"
    );
}

fn prepare_arbitrary_cube_labels(cube: &AisViewCube) {
    {
        let mut settings = cube.view_cube_presentation_base().settings.write();
        settings.side_labels.unbind(&V3dTypeOfOrientation::Yneg);
        settings.side_labels.bind(
            V3dTypeOfOrientation::XposYpos,
            AsciiString::from("edge-key"),
        );
        settings
            .axes_labels
            .bind(Prs3dDatumParts::Origin, AsciiString::from("origin-key"));
        settings.axes_labels.unbind(&Prs3dDatumParts::XAxis);
    }
}

fn advance_cube_appearance_lifecycle(scene: &CubeAppearanceScene, stage: &str) {
    match stage {
        "1" => scene
            .scene
            .context
            .recompute_prs_only(&scene.scene.cube.cast(), false, false)
            .unwrap(),
        "2" => scene
            .scene
            .context
            .redisplay(&scene.scene.cube.cast(), false, false)
            .unwrap(),
        _ => {}
    }
}
