//! Native range/float guards and complete protected-map key traversal preserve observable behavior.
use super::{view_cube_appearance_fixture::*, view_cube_interaction_fixture::widget_rows, *};
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dDatumParts, v3d::V3dTypeOfOrientation,
};
use rustycad_tkernel::{ncollection::HashPolicy, tcollection::AsciiString};
#[test]
fn native_rounding_and_transparency_error_boundaries_nan_and_f32_rounding() {
    for row in widget_rows(APPEARANCE, "boundary") {
        replay_cube_boundary(&row);
    }
}
fn replay_cube_boundary(row: &[&str]) {
    let id = row[0].parse::<usize>().unwrap();
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    let values = cube_boundary_values();
    let result = if id < 5 {
        cube.set_round_radius(values[id])
    } else {
        cube.set_box_transparency(values[id])
    };
    assert_boundary_result(&result, row);
    let actual = if id < 5 {
        cube.round_radius()
    } else {
        cube.box_transparency()
    };
    assert_eq!(actual, row[2].parse::<f64>().unwrap());
    assert_eq!(
        cube_boundary_state(&scene, id),
        row[3].parse::<usize>().unwrap()
    );
}
fn assert_boundary_result(result: &Result<(), rustycad_tkernel::standard::Failure>, row: &[&str]) {
    assert_eq!(
        result
            .as_ref()
            .err()
            .map_or("none", |error| error.exception_type()),
        row[1]
    );
    if let Err(error) = result {
        assert!(
            error.to_string().contains("expected")
                || error.exception_type() == "Graphic3d_MaterialDefinitionError"
        );
    }
}
fn cube_boundary_state(scene: &CubeAppearanceScene, id: usize) -> usize {
    if id < 5 {
        return scene.scene.cube.presentations()[0].must_be_updated() as usize;
    }
    scene
        .trace
        .read()
        .iter()
        .filter(|event| {
            matches!(
                event,
                super::color_scale_recording::ScaleDrawRequest::Synchronize
            )
        })
        .count()
}
#[test]
fn native_all_valid_protected_map_keys_missing_queries_and_hash_traversal() {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    let mut settings = cube.view_cube_presentation_base().settings.write();
    settings.side_labels.clear(false);
    settings.axes_labels.clear(false);
    fill_protected_cube_label_keys(&mut settings);
    assert_enum_label_hashes();
    assert_label_map_orders(&settings);
    drop(settings);
    assert_cube_appearance(&scene, &widget_rows(APPEARANCE, "allkeys").next().unwrap());
}
fn assert_map_key_order(kind: &str, actual: Vec<i32>) {
    let row = widget_rows(APPEARANCE, "map-order")
        .find(|row| row[0] == kind)
        .unwrap();
    assert_eq!(
        actual,
        row[1..]
            .iter()
            .map(|value| value.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    );
}
fn assert_enum_label_hashes() {
    let policy = AisViewCubeLabelHash;
    for left in V3dTypeOfOrientation::ALL {
        assert_eq!(policy.hash(&left), left as i32 as usize);
        for right in V3dTypeOfOrientation::ALL {
            assert_eq!(policy.equal(&left, &right), left == right);
        }
    }
    for left in Prs3dDatumParts::DRAWN
        .into_iter()
        .chain([Prs3dDatumParts::None])
    {
        assert_eq!(policy.hash(&left), left as i32 as usize);
        for right in Prs3dDatumParts::DRAWN
            .into_iter()
            .chain([Prs3dDatumParts::None])
        {
            assert_eq!(policy.equal(&left, &right), left == right);
        }
    }
}

fn assert_label_map_orders(settings: &AisViewCubePresentationSettings) {
    assert_map_key_order(
        "sides",
        settings
            .side_labels
            .items()
            .map(|(key, _)| *key as i32)
            .collect(),
    );
    assert_map_key_order(
        "axes",
        settings
            .axes_labels
            .items()
            .map(|(key, _)| *key as i32)
            .collect(),
    );
}

fn fill_protected_cube_label_keys(settings: &mut AisViewCubePresentationSettings) {
    for key in V3dTypeOfOrientation::ALL {
        settings
            .side_labels
            .bind(key, AsciiString::from_integer(key as i32));
    }
    for key in Prs3dDatumParts::DRAWN
        .into_iter()
        .chain([Prs3dDatumParts::None])
    {
        settings
            .axes_labels
            .bind(key, AsciiString::from_integer(key as i32));
    }
}

fn cube_boundary_values() -> [f64; 9] {
    [
        -0.1,
        0.6,
        0.0,
        0.5,
        f64::NAN,
        -0.1,
        1.1,
        1.0000000001,
        f64::NAN,
    ]
}
