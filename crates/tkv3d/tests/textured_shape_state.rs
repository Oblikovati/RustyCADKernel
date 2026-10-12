//! Pinned native constructor/source/mapping rows replay without OCCT or filesystem I/O.
#[path = "support/textured_shape_state_fixture.rs"]
mod state_fixture;
use rustycad_tkernel::{
    standard::{Handle, RuntimeTyped},
    tcollection::AsciiString,
};
use rustycad_tkservice::image::ImagePixMap;
use rustycad_tkv3d::ais::*;
use state_fixture::TexturedStateFixture;
use std::sync::atomic::Ordering;
const NATIVE: &str = include_str!("fixtures/ais_textured_shape/native-state.txt");

fn native_state_row(label: &str) -> Vec<&'static str> {
    let prefix = format!("MARTENSITE_STATE {label} ");
    NATIVE
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .expect("committed native state label")
        .split_whitespace()
        .collect()
}
fn compare_textured_fields(object: &TexturedStateFixture, label: &str) {
    let row = native_state_row(label);
    compare_textured_flags(object, label, &row[3..9]);
    compare_textured_coordinates(object, label, &row[9..15]);
    assert_eq!(
        object.base.fields.read().predefined_texture as i32,
        row[16].parse::<i32>().unwrap()
    );
}
fn compare_textured_flags(object: &TexturedStateFixture, label: &str, expected: &[&str]) {
    let flags = [
        object.texture_map_state(),
        object.texture_modulate(),
        object.texture_origin(),
        object.texture_repeat(),
        object.texture_scale(),
        object.show_triangles(),
    ];
    for (actual, native) in flags.into_iter().zip(expected) {
        assert_eq!(actual, *native == "1", "{label}");
    }
}
fn compare_textured_coordinates(object: &TexturedStateFixture, label: &str, expected: &[&str]) {
    let scalars = [
        object.texture_u_origin(),
        object.texture_v_origin(),
        object.u_repeat(),
        object.v_repeat(),
        object.texture_scale_u(),
        object.texture_scale_v(),
    ];
    for (actual, native) in scalars.into_iter().zip(expected) {
        assert_eq!(actual, native.parse::<f64>().unwrap(), "{label}");
    }
}
const FILE_CASES: [&str; 8] = [
    "21",
    "22",
    "-1",
    " 7 ",
    "1e2",
    "2147483648",
    "texture path.png",
    "",
];
#[test]
fn native_constructor_and_mapping_flags_preserve_disabled_coordinates() {
    let object = TexturedStateFixture::new();
    compare_textured_fields(&object, "default");
    object
        .set_texture_file_name(&AsciiString::default())
        .unwrap();
    configure_textured_mapping(&object, false);
    compare_textured_fields(&object, "disabled");
    configure_textured_mapping(&object, true);
    compare_textured_fields(&object, "enabled");
    assert!(object.base.shape.fields.read().shape.is_null());
    assert!(object.base.shape.fields.read().bounds.is_void());
    assert!(object.base.shape.fields.read().recompute_bounds);
    assert_eq!(object.base.shape.fields.read().initial_angle, 0.);
}
fn configure_textured_mapping(object: &TexturedStateFixture, enabled: bool) {
    if enabled {
        object.set_texture_map_on();
        object.enable_texture_modulate();
        object.set_texture_origin(true, -1., 2.);
        object.set_texture_repeat(true, 3., 4.);
        object.set_texture_scale(true, 5., 6.);
    } else {
        object.set_texture_map_off();
        object.disable_texture_modulate();
        object.set_texture_origin(false, 2.5, -3.5);
        object.set_texture_repeat(false, -4., 0.);
        object.set_texture_scale(false, 0., -7.);
    }
    object.set_show_triangles(!enabled);
}
#[test]
fn native_filename_prefix_parsing_dispatch_and_source_release_are_preserved() {
    let object = TexturedStateFixture::new();
    let inherited = object.cast::<dyn AisTexturedShapeSource>();
    for (index, name) in FILE_CASES.into_iter().enumerate() {
        inherited.set_texture_pix_map(&Handle::new(ImagePixMap::default()));
        inherited
            .set_texture_file_name(&AsciiString::from(name))
            .unwrap();
        compare_textured_fields(&object, &format!("file{index}"));
        assert!(inherited.texture_pix_map().read().is_null());
    }
    assert_eq!(object.file_calls.load(Ordering::Relaxed), 8);
    assert_eq!(object.pixmap_calls.load(Ordering::Relaxed), 8);
    assert!(inherited.is_kind(<dyn AisShapeSource>::static_type()));
    compare_native_filename_messages(&object);
}
#[test]
fn native_const_pixmap_alias_survives_replacement_and_filename_nullification() {
    let object = TexturedStateFixture::new();
    let first = Handle::new(ImagePixMap::default());
    let second = Handle::new(ImagePixMap::default());
    first.set_top_down(true);
    second.set_top_down(false);
    object.set_texture_pix_map(&first);
    let alias = object.texture_pix_map();
    assert_eq!(alias.read().object_address(), first.object_address());
    object.set_texture_pix_map(&second);
    assert_eq!(alias.read().object_address(), second.object_address());
    assert!(first.is_top_down());
    assert!(!second.is_top_down());
    object
        .set_texture_file_name(&AsciiString::from("3"))
        .unwrap();
    assert!(alias.read().is_null());
}

fn compare_native_filename_messages(object: &TexturedStateFixture) {
    let expected: Vec<&str> = NATIVE
        .lines()
        .filter_map(|line| {
            line.strip_prefix("\x1b[31;1m")
                .and_then(|text| text.strip_suffix("\x1b[0m"))
        })
        .collect();
    let messages = object.diagnostics.messages.read();
    assert_eq!(messages.len(), expected.len());
    for ((gravity, actual), expected) in messages.iter().zip(expected) {
        assert_eq!(*gravity, rustycad_tkernel::message::Gravity::Fail);
        assert_eq!(actual.as_bytes(), expected.as_bytes());
    }
}
