//! Font replacement keeps old payload references fixed; mutation of that retained payload remains live.
use super::{
    view_cube_appearance_fixture::CubeAppearanceScene, view_cube_interaction_fixture::widget_rows,
    *,
};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::{
    standard::Handle,
    tcollection::{AsciiString, HAsciiString},
};
use rustycad_tkservice::graphic3d::Graphic3dAspectsTextMembers;
const ALIASES: &str = include_str!("../../tests/fixtures/view-cube-appearance-aliases.txt");
#[test]
fn native_font_payload_aliases_keep_original_storage_and_static_empty_identity() {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    let owner = cube.attributes().text_aspect().aspect().text_font();
    let left = cube.font_ref();
    let right = cube.font_ref();
    assert_font_alias_row(&left, &right, &owner, &left.value(), 0);
    cube.set_font(&AsciiString::from("new-font"));
    assert_font_alias_row(&left, &cube.font_ref(), &owner, &cube.font_ref().value(), 1);
    owner.set_value_char(1, b'Q').unwrap();
    assert_font_alias_row(&left, &right, &owner, &cube.font_ref().value(), 2);
    replay_empty_cube_font(cube, &left, &owner);
}
fn replay_empty_cube_font(
    cube: &AisViewCube,
    left: &AisViewCubeFontRef<'_>,
    owner: &Handle<HAsciiString>,
) {
    cube.set_font(&AsciiString::default());
    let empty = cube.font_ref();
    assert_font_alias_row(left, &empty, owner, &empty.value(), 3);
    cube.set_font(&AsciiString::from("restored"));
    assert_font_alias_row(&empty, &cube.font_ref(), owner, &cube.font_ref().value(), 4);
    assert_eq!(left.object_address(), owner.object_address());
    assert!(empty.object_address().is_null());
}
fn assert_font_alias_row(
    left: &AisViewCubeFontRef<'_>,
    compared: &AisViewCubeFontRef<'_>,
    owner: &Handle<HAsciiString>,
    current: &AsciiString,
    step: i32,
) {
    let row = widget_rows(ALIASES, "font")
        .find(|row| row[0].parse::<i32>().unwrap() == step)
        .unwrap();
    assert_eq!(left.is_same_reference(compared), row[1] == "1");
    assert_eq!(owner.ref_count(), row[2].parse::<i32>().unwrap());
    assert_font_payload(&left.value(), &row[3..5]);
    assert_font_payload(current, &row[5..7]);
}
fn assert_font_payload(value: &AsciiString, row: &[&str]) {
    assert_eq!(value.length(), row[0].parse::<i32>().unwrap());
    let hex = value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(if hex.is_empty() { "-" } else { &hex }, row[1]);
}
