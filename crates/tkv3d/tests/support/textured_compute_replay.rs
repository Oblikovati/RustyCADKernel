//! Concrete protected Compute replay uses the genuine inherited WF/shading presentation path.
#[allow(dead_code)]
#[path = "sensitive_display_driver.rs"]
mod display_driver;
#[path = "textured_face_fixture.rs"]
mod face_fixture;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{
    graphic3d::*,
    image::{ImageFormat, ImagePixMap},
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::V3dViewer};

pub fn replay_textured_compute_case(scenario: i32, mode: i32) {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let object = textured_compute_object(scenario);
    object
        .compute(&Handle::default(), &prs.cast(), mode)
        .unwrap();
    let (groups, expected) = native_textured_compute_events(scenario, mode);
    assert_eq!(
        prs.groups().len(),
        groups,
        "Compute groups {scenario}/{mode}"
    );
    assert_eq!(
        trace.read().events,
        expected,
        "native Compute {scenario}/{mode}"
    );
}
fn textured_compute_object(scenario: i32) -> Handle<AisTexturedShape> {
    let face = face_fixture::textured_face_fixture(1).unwrap();
    let object = AisTexturedShape::new(&face);
    configure_textured_compute(&object, scenario);
    object
}
fn configure_textured_compute(object: &AisTexturedShape, scenario: i32) {
    let drawer = object.attributes();
    drawer.setup_own_defaults().unwrap();
    drawer.set_auto_triangulation(false);
    drawer.u_iso_aspect().set_number(0);
    drawer.v_iso_aspect().set_number(0);
    object.set_texture_origin(scenario & 1 != 0, 0.25, -0.5);
    object.set_texture_repeat(scenario & 2 != 0, 2.0, 3.0);
    object.set_texture_scale(scenario & 4 != 0, 0.5, 2.0);
    let pixels = Handle::new(ImagePixMap::default());
    assert!(pixels.init_zero(ImageFormat::Rgba, 2, 2, 0, 0));
    pixels.set_top_down(true);
    object.set_texture_pix_map(&pixels);
}
fn native_textured_compute_events(scenario: i32, mode: i32) -> (usize, Vec<f64>) {
    let prefix = format!("MARTENSITE_COMPUTE {scenario} {mode} ");
    let native = include_str!("../fixtures/ais_textured_shape/native-compute.txt");
    let row = native
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap();
    let mut fields = row.split_whitespace();
    let groups = fields.next().unwrap().parse().unwrap();
    let count: usize = fields.next().unwrap().parse().unwrap();
    let events: Vec<f64> = fields.map(|value| value.parse().unwrap()).collect();
    assert_eq!(events.len(), count);
    (groups, events)
}
