//! Reusable whole-WF native callback replay for TexturedShape's genuine presentation dependency.
#[allow(dead_code)]
#[path = "sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::Plane3;
use rustycad_tkservice::{aspect::AspectTypeOfDeflection, graphic3d::*};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsWfShape, v3d::V3dViewer};

pub fn replay_textured_wireframe_case(scenario: i32) {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2., 3., -1., 4.))
        .unwrap()
        .face()
        .unwrap();
    StdPrsWfShape::add(&prs.cast(), &face, &textured_wireframe_drawer(scenario)).unwrap();
    let (groups, expected) = native_wireframe_events(scenario);
    assert_eq!(prs.groups().len(), groups);
    assert_eq!(trace.read().events, expected, "native WF case {scenario}");
}
fn native_wireframe_events(scenario: i32) -> (usize, Vec<f64>) {
    let native = include_str!("../fixtures/ais_textured_shape/native-wireframe.txt");
    let prefix = format!("MARTENSITE_WF {scenario} ");
    let mut fields = native
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
        .split_whitespace();
    let groups = fields.next().unwrap().parse().unwrap();
    let count: usize = fields.next().unwrap().parse().unwrap();
    let events: Vec<f64> = fields.map(|value| value.parse().unwrap()).collect();
    assert_eq!(events.len(), count);
    (groups, events)
}
fn textured_wireframe_drawer(scenario: i32) -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.set_auto_triangulation(false);
    drawer.set_type_of_deflection(AspectTypeOfDeflection::Absolute);
    drawer.set_maximal_chordial_deviation(0.2);
    drawer.set_iso_on_plane(scenario & 8 != 0);
    drawer.u_iso_aspect().set_number(2);
    drawer.v_iso_aspect().set_number(3);
    configure_wireframe_route_colors(&drawer, scenario);
    drawer
}
fn configure_wireframe_route_colors(drawer: &Prs3dDrawer, scenario: i32) {
    drawer
        .wire_aspect()
        .set_color(Color::named(NamedColor::GREEN));
    drawer
        .free_boundary_aspect()
        .set_color(Color::named(NamedColor::WHITE));
    drawer
        .u_iso_aspect()
        .set_color(Color::named(NamedColor::RED));
    let v = if scenario & 1 != 0 {
        NamedColor::RED
    } else {
        NamedColor::BLUE
    };
    drawer.v_iso_aspect().set_color(Color::named(v));
    configure_wireframe_common_colors(drawer, scenario);
}

fn configure_wireframe_common_colors(drawer: &Prs3dDrawer, scenario: i32) {
    if scenario & 2 != 0 {
        drawer
            .u_iso_aspect()
            .set_color(Color::named(NamedColor::GREEN));
    }
    if scenario & 4 != 0 {
        drawer
            .v_iso_aspect()
            .set_color(Color::named(NamedColor::GREEN));
    }
}
