//! Native six-category width propagation, release, linked restoration and failure state.
#[allow(dead_code)]
#[path = "support/textured_shape_state_fixture.rs"]
mod state_fixture;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkv3d::{
    ais::*,
    prs_mgr::*,
    prs3d::{Prs3dDrawer, Prs3dLineAspect},
};
use state_fixture::TexturedStateFixture;

fn width_lines(drawer: &Prs3dDrawer) -> [Handle<Prs3dLineAspect>; 6] {
    [
        drawer.line_aspect(),
        drawer.wire_aspect(),
        drawer.free_boundary_aspect(),
        drawer.un_free_boundary_aspect(),
        drawer.seen_line_aspect(),
        drawer.face_boundary_aspect(),
    ]
}
fn width_fixture(linked: bool) -> Handle<TexturedStateFixture> {
    let object = TexturedStateFixture::new();
    object.attributes().setup_own_defaults().unwrap();
    if !linked {
        return object;
    }
    let defaults = Handle::new(Prs3dDrawer::new());
    defaults.setup_own_defaults().unwrap();
    for (index, line) in width_lines(&defaults).into_iter().enumerate() {
        line.set_width((index + 1) as f64).unwrap();
    }
    object.attributes().link_to(&defaults);
    object
}
#[test]
fn textured_parent_width_updates_six_categories_then_releases_and_noops() {
    let object = width_fixture(true);
    ais_shape_set_width(&*object, 2.75).unwrap();
    compare_width(&object, "plain");
    ais_shape_unset_width(&*object).unwrap();
    compare_width(&object, "plainUnset");
    ais_shape_unset_width(&*object).unwrap();
    compare_width(&object, "plainNoop");
}
#[test]
fn textured_parent_colored_width_restores_distinct_linked_or_unlinked_defaults() {
    for (linked, before, after) in [
        (true, "color", "colorUnset"),
        (false, "unlinked", "unlinkedUnset"),
    ] {
        let object = width_fixture(linked);
        ais_shape_set_color(&*object, Color::named(NamedColor::RED)).unwrap();
        ais_shape_set_width(&*object, 2.75).unwrap();
        compare_width(&object, before);
        ais_shape_unset_width(&*object).unwrap();
        compare_width(&object, after);
    }
}
#[test]
fn textured_parent_rejected_width_publishes_float_before_aspect_failure() {
    let object = width_fixture(true);
    assert!(ais_shape_set_width(&*object, -2.0).is_err());
    compare_width(&object, "failure");
}
fn compare_width(object: &TexturedStateFixture, label: &str) {
    let drawer = object.attributes();
    let mut actual = vec![
        f64::from(i32::from(object.has_width())),
        object.width(),
        f64::from(i32::from(drawer.has_own_line_aspect())),
    ];
    actual.extend(
        width_lines(&drawer)
            .into_iter()
            .map(|line| f64::from(line.aspect().width())),
    );
    assert_eq!(actual, native_width(label), "native width {label}");
}
fn native_width(label: &str) -> Vec<f64> {
    let prefix = format!("MARTENSITE_WIDTH {label} ");
    include_str!("fixtures/ais_textured_shape/native-width.txt")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}
