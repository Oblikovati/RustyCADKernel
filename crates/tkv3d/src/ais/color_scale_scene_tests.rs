//! Real drawing calls replay native colored triangles, frame vertices, text requests and group counts.
use super::{AisColorScale, color_scale_recording::*};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Handle, SharedState},
    tcollection::ExtendedString,
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
use std::sync::Arc;
const SCENES: [&str; 2] = [
    include_str!("../../tests/fixtures/color-scale-native-scenes-0.txt"),
    include_str!("../../tests/fixtures/color-scale-native-scenes-1.txt"),
];
pub(super) fn configured_scale(id: i32) -> Handle<AisColorScale> {
    let scale = AisColorScale::new().unwrap();
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.set_text_aspect(Handle::new(Prs3dTextAspect::new()));
    scale.attributes().link_to(&drawer);
    configure_scale_layout(&scale, id);
    configure_custom_scale(&scale, id);
    if id % 7 == 0 {
        scale.set_color(Color::named(NamedColor::CYAN)).unwrap();
    }
    if id % 6 == 0 {
        scale.set_hue_range(60.0, 300.0);
    }
    scale
}
fn configure_custom_scale(scale: &AisColorScale, id: i32) {
    if id >= 16 {
        scale.set_color_type(AspectTypeOfColorScaleData::User);
        for (index, color) in [NamedColor::RED, NamedColor::GREEN, NamedColor::BLUE]
            .into_iter()
            .enumerate()
        {
            scale.set_interval_color(Color::named(color), index as i32 + 1);
        }
    }
    if id >= 24 {
        scale.set_label_type(AspectTypeOfColorScaleData::User);
        for (index, label) in ["low", "", "high", "free"].into_iter().enumerate() {
            scale.set_label(&ExtendedString::from_utf8(label), index as i32 + 1);
        }
    }
}
#[test]
fn native_color_scale_actual_submissions_and_layout() {
    for id in 0..32 {
        let scale = configured_scale(id);
        let trace = Arc::new(SharedState::new(Vec::new()));
        let prs = recording_presentation(&trace);
        scale.compute(&Handle::default(), &prs, 0).unwrap();
        let (requests, groups) = expected_scene(id);
        assert_eq!(*trace.read(), requests, "native scene {id}");
        assert_eq!(prs.groups().len(), groups, "native scene {id}");
        assert_scale_text_aspect(&scale);
    }
}
fn assert_scale_text_aspect(scale: &AisColorScale) {
    let aspect = scale.attributes().text_aspect();
    assert_eq!(aspect.height(), f64::from(scale.get_text_height()));
    assert_eq!(
        aspect.horizontal_justification(),
        Graphic3dHorizontalTextAlignment::Left
    );
    assert_eq!(
        aspect.vertical_justification(),
        Graphic3dVerticalTextAlignment::Bottom
    );
    assert!(aspect.aspect().is_text_zoomable());
}
fn expected_scene(id: i32) -> (Vec<ScaleDrawRequest>, usize) {
    let lines = SCENES
        .iter()
        .flat_map(|fixture| fixture.lines())
        .filter(|line| {
            line.split_whitespace()
                .nth(1)
                .unwrap()
                .parse::<i32>()
                .unwrap()
                == id
        });
    let mut events = Vec::new();
    let mut groups = 0;
    for line in lines {
        parse_scene_line(line, &mut events, &mut groups);
    }
    (events, groups)
}
fn parse_scene_line(line: &str, events: &mut Vec<ScaleDrawRequest>, groups: &mut usize) {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    match parts[0] {
        "array" => events.push(ScaleDrawRequest::Array {
            kind: parts[2].parse().unwrap(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }),
        "vertex" => append_expected_vertex(events, &parts),
        "indices" => append_expected_indices(events, &parts),
        "text" => events.push(expected_text(&parts)),
        "groups" => *groups = parts[2].parse().unwrap(),
        unexpected => {
            panic!("native scene command={unexpected}; expected array/vertex/indices/text/groups")
        }
    }
}
fn append_expected_vertex(events: &mut [ScaleDrawRequest], parts: &[&str]) {
    let Some(ScaleDrawRequest::Array { vertices, .. }) = events.last_mut() else {
        panic!("vertex must follow array")
    };
    vertices.push(
        parts[4..]
            .iter()
            .map(|value| value.parse().unwrap())
            .collect(),
    );
}
fn append_expected_indices(events: &mut [ScaleDrawRequest], parts: &[&str]) {
    let Some(ScaleDrawRequest::Array { indices, .. }) = events.last_mut() else {
        panic!("indices must follow array")
    };
    *indices = parts[2..]
        .iter()
        .map(|value| value.parse().unwrap())
        .collect();
}
fn expected_text(parts: &[&str]) -> ScaleDrawRequest {
    ScaleDrawRequest::Text {
        point: [parts[2].parse().unwrap(), parts[3].parse().unwrap()],
        height: parts[4].parse().unwrap(),
        alignment: parts[5].parse().unwrap(),
        anchor: parts[6] == "1",
        text: parts[7].trim_matches('"').to_owned(),
    }
}
#[test]
fn scale_unsupported_modes_and_selection_preserve_existing_content() {
    let scale = AisColorScale::new().unwrap();
    let trace = Arc::new(SharedState::new(Vec::new()));
    let prs = recording_presentation(&trace);
    scale.compute(&Handle::default(), &prs, 3).unwrap();
    assert!(trace.read().is_empty());
    assert!(!scale.attributes().has_own_text_aspect());
    let selection = Handle::new(SelectMgrSelection::new(0));
    scale.compute_selection(&selection, 0).unwrap();
    assert!(selection.entities().is_empty());
    assert!(scale.accept_display_mode(0));
    assert!(!scale.accept_display_mode(1));
}

fn configure_scale_layout(scale: &AisColorScale, id: i32) {
    scale.set_size(90, if id % 5 == 0 { 50 } else { 241 });
    scale.set_position(-3, 7);
    scale.set_number_of_intervals(id % 3 + 1);
    scale.set_label_position(
        [
            AspectTypeOfColorScalePosition::None,
            AspectTypeOfColorScalePosition::Left,
            AspectTypeOfColorScalePosition::Right,
            AspectTypeOfColorScalePosition::Center,
        ][id as usize % 4],
    );
    scale.set_smooth_transition((id / 4) % 2 != 0);
    scale.set_reversed((id / 8) % 2 != 0);
    scale.set_label_at_border((id / 16) % 2 == 0);
    if id % 3 == 1 {
        scale.set_title(&ExtendedString::from_utf8("Pressure"));
    }
}
#[test]
fn scale_selection_keeps_existing_sensitive_entities() {
    use crate::select3d::Select3DSensitivePoint;
    use rustycad_tkmath::gp::Point3;
    let scale = AisColorScale::new().unwrap();
    let selection = Handle::new(SelectMgrSelection::new(0));
    let point = Handle::new(Select3DSensitivePoint::new(
        &Handle::default(),
        Point3::new(1.0, 2.0, 3.0),
    ));
    selection.add(&point.cast()).unwrap();
    scale.compute_selection(&selection, 0).unwrap();
    assert_eq!(selection.entities().len(), 1);
}
#[test]
fn scale_lone_border_label_returns_typed_failure() {
    let scale = configured_scale(1);
    scale.set_label_type(AspectTypeOfColorScaleData::User);
    scale.set_labels(&[ExtendedString::from_utf8("one")]);
    let trace = Arc::new(SharedState::new(Vec::new()));
    let prs = recording_presentation(&trace);
    assert!(scale.compute(&Handle::default(), &prs, 0).is_err());
}
#[test]
fn scale_unicode_title_and_nonpositive_bar_height_submit_only_text() {
    let scale = configured_scale(1);
    scale.set_title(&ExtendedString::from_utf8("温度 °C"));
    scale.set_label_type(AspectTypeOfColorScaleData::User);
    scale.set_labels(&[]);
    scale.set_height(0);
    let trace = Arc::new(SharedState::new(Vec::new()));
    let prs = recording_presentation(&trace);
    scale.compute(&Handle::default(), &prs, 0).unwrap();
    assert_eq!(trace.read().len(), 1);
    let ScaleDrawRequest::Text { text, .. } = &trace.read()[0] else {
        panic!("expected title")
    };
    assert_eq!(text, "温度 °C");
}
