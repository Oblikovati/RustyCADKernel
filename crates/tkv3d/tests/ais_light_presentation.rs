//! Native light constructor and actual primitive-buffer replay; no OCCT runtime dependency.
use rustycad_tkernel::{
    quantity::{Color, ColorSpace},
    standard::Handle,
    tcollection::AsciiString,
};
use rustycad_tkmath::gp::Direction3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::V3dViewer};
#[path = "support/sensitive_display_driver.rs"]
mod recording_driver;
#[path = "support/light_presentation_snapshot.rs"]
mod snapshot;
fn native_light(kind: usize, flags: i32) -> Handle<Graphic3dCLight> {
    let light = fixture_source::fixture_light(kind);
    light.set_color(Color::new(0.2, 0.4, 0.8, ColorSpace::Rgb).unwrap());
    light.set_enabled(flags & 2 != 0);
    if kind != 0 {
        light.set_headlight(flags & 16 != 0).unwrap();
    }
    if (kind == 2 || kind == 3) && flags & 8 != 0 {
        light.set_range(80.0).unwrap();
    }
    if kind == 1 || kind == 3 {
        light.set_direction(&Direction3::Z.reversed()).unwrap();
    }
    light.set_name(&AsciiString::new("light é"));
    light
}
fn configure(object: &AisLightSource, flags: i32, arrows: i32) {
    object.set_zoomable(flags & 1 != 0);
    object.set_display_range(flags & 4 != 0);
    object.set_display_name(flags & 32 != 0);
    object.set_nb_arrows(arrows);
    object.set_nb_splits_quadric(4);
    object.set_nb_splits_arrow(3);
}
#[test]
fn constructor_and_rendered_arrays_match_native_snapshots() {
    for (index, line) in include_str!("fixtures/light-presentation-native.txt")
        .lines()
        .enumerate()
    {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        let actual = replay(&expected);
        assert_eq!(actual.len(), expected.len() - 4, "case {index}");
        for (column, (actual, expected)) in actual.iter().zip(&expected[4..]).enumerate() {
            assert!(
                (actual - expected).abs() < 2e-12,
                "case {index} column {column}: actual {actual}, native {expected}"
            );
        }
    }
}
fn replay(expected: &[f64]) -> Vec<f64> {
    let (kind, mode, flags, arrows) = (
        expected[0] as usize,
        expected[1] as i32,
        expected[2] as i32,
        expected[3] as i32,
    );
    let (driver, trace) = recording_driver::SensitiveDisplayDriver::new_handle(-1);
    let viewer = V3dViewer::new(&driver).unwrap();
    let object = AisLightSource::new(&native_light(kind, flags)).unwrap();
    let mut actual = snapshot::constructor(&object);
    configure(&object, flags, arrows);
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    reset_presentation_trace(&trace);
    object
        .compute(&Handle::default(), &prs.cast(), mode)
        .unwrap();
    append_presentation_capture(&prs.cast(), &trace.read(), &mut actual);
    actual
}

#[path = "support/light_fixture_source.rs"]
mod fixture_source;

fn append_presentation_capture(
    prs: &Handle<dyn Graphic3dStructureSource>,
    trace: &recording_driver::SensitiveDisplayTrace,
    actual: &mut Vec<f64>,
) {
    let events = trace.events.clone();
    actual.push(events.len() as f64);
    actual.extend(events);
    snapshot::append_groups(prs, actual);
    snapshot::append_texts(&trace.texts, actual);
}

fn reset_presentation_trace(
    trace: &rustycad_tkernel::standard::SharedState<recording_driver::SensitiveDisplayTrace>,
) {
    trace.write().events.clear();
    trace.write().groups = 0;
}
