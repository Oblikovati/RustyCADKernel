//! Full actual submission replay: shaded axes/sphere, side frames, outlines and group closure.
use super::{color_scale_recording::*, view_cube_fixture_replay::*, view_cube_test_probe::*, *};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkservice::graphic3d::*;
use std::sync::{Arc, atomic::Ordering};
#[test]
fn native_cube_complete_presentation_submission_and_oriented_text_frames() {
    for id in 0..10 {
        let cube = configured_cube(id % 8);
        let trace = Arc::new(SharedState::new(Vec::new()));
        let prs = recording_oriented_presentation(&trace);
        ais_view_cube_compute(&*cube, &prs, if id < 8 { 0 } else { 1 }).unwrap();
        let expected = expected_cube_scene(id);
        assert_eq!(trace.read().len(), expected.len(), "scene {id}");
        for (actual, expected) in trace.read().iter().zip(&expected) {
            assert_cube_request(actual, expected);
        }
        assert_cube_groups(&prs, id);
        assert_eq!(
            cube.side_calls.load(Ordering::Relaxed),
            if id < 8 { 12 } else { 0 }
        );
    }
}
fn expected_cube_scene(id: i32) -> Vec<ScaleDrawRequest> {
    PRESENTATIONS
        .lines()
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts[1].parse::<i32>().unwrap() != id {
                return None;
            }
            match parts[0] {
                "array" => Some(parse_cube_array(&parts[1..])),
                "text" => Some(parse_cube_text(&parts[1..])),
                _ => None,
            }
        })
        .collect()
}
fn assert_cube_groups(prs: &Handle<dyn Graphic3dStructureSource>, id: i32) {
    let row = fixture_rows(PRESENTATIONS, "groups")
        .find(|row| row[0].parse::<i32>().unwrap() == id)
        .unwrap();
    assert_eq!(prs.groups().len(), row[1].parse::<usize>().unwrap());
    assert_eq!(prs.is_infinite(), row[2] == "1");
    for (group, closed) in prs.groups().iter().zip(&row[3..]) {
        assert_eq!(group.is_closed(), *closed == "1");
    }
}
#[test]
fn cube_partial_contract_rejects_selection_and_preserves_geometry_adapter() {
    let cube = CubePresentationProbe::new();
    let selection = Handle::new(SelectMgrSelection::new(0));
    assert!(cube.compute_selection(&selection, 0).is_err());
    let trace = Arc::new(SharedState::new(Vec::new()));
    let prs = recording_oriented_presentation(&trace);
    cube.compute(&Handle::default(), &prs, 0).unwrap();
    assert!(!trace.read().is_empty());
}
#[test]
fn cube_context_drawer_preconditions_return_typed_failures() {
    let cube = CubePresentationProbe::new();
    cube.attributes().link_to(&Handle::default());
    let trace = Arc::new(SharedState::new(Vec::new()));
    let prs = recording_oriented_presentation(&trace);
    assert!(ais_view_cube_compute(&*cube, &prs, 0).is_err());
    assert!(prs.is_infinite());
    cube.display.settings.write().draw_axes = false;
    cube.attributes().set_face_boundary_draw(true);
    assert!(ais_view_cube_compute(&*cube, &prs, 0).is_err());
}
#[test]
fn cube_default_state_appearance_and_persistence_match_native_display_defaults() {
    let cube = CubePresentationProbe::new();
    let settings = cube.display.settings.read();
    assert_eq!(
        (
            settings.geometry.size,
            settings.geometry.facet_extension,
            settings.geometry.axes_padding
        ),
        (70.0, 10.5, 7.0)
    );
    assert!(settings.draw_axes && settings.draw_edges && settings.draw_corners);
    assert!(!settings.is_y_up);
    assert!(!cube.attributes().has_own_datum_aspect());
    assert!(!cube.attributes().datum_aspect().is_null());
    assert!(cube.is_infinite());
    assert!(cube.is_mutable());
    assert_eq!(cube.attributes().z_layer(), GRAPHIC3D_ZLAYER_ID_TOPMOST);
}
#[test]
fn cube_text_appearance_matches_native_display_defaults() {
    let cube = CubePresentationProbe::new();
    let text = cube.attributes().text_aspect();
    assert_eq!(text.height(), 70.0 * 0.16);
    assert!(text.aspect().is_text_zoomable());
    assert_eq!(
        text.aspect().face_culling(),
        Graphic3dTypeOfBackfacingModel::BackCulled
    );
}
