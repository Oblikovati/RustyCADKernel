//! Every native face presentation overload uses genuine located topology and mesh fallback.
use super::{
    StdPrsIsolines, isoline_face_fixture::isoline_fixture_surface, isoline_triangle_fixture::*,
    isoline_triangle_tests::isoline_presentation_snapshot,
};
use crate::{
    ais::xr_tests::{
        xr_recording_group::XrRecordingEvents, xr_recording_structure::xr_recording_presentation,
    },
    prs3d::Prs3dDrawer,
    select3d::compare_oracle_snapshot,
};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::sync::Arc;
const NATIVE_FACE_PATHS: &str = include_str!("../../tests/fixtures/isolines/native-face-paths.txt");
#[test]
fn native_face_surface_mesh_overloads_and_missing_mesh_fallback() {
    for (scenario, row) in NATIVE_FACE_PATHS.lines().enumerate() {
        compare_oracle_snapshot(&face_path_snapshot(scenario as i32), row);
    }
}
fn face_path_topology(scenario: i32) -> Face {
    let source = face_path_surface(scenario);
    let mut face = BRepBuilderAPIMakeFace::from_surface_bounds(&source, (0., 1., 0., 1.), 1e-7)
        .unwrap()
        .face()
        .unwrap();
    if scenario % 12 >= 6 || scenario >= 24 {
        BRepBuilder::default()
            .update_face_triangulation(&face, &isoline_triangle_grid(&source, 0), true)
            .unwrap();
    }
    locate_face_path_topology(&mut face, &source, scenario);
    face
}
fn face_path_snapshot(scenario: i32) -> Vec<f64> {
    let face = face_path_topology(scenario);
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    drawer.u_iso_aspect().set_number(2);
    drawer.v_iso_aspect().set_number(3);
    drawer.set_iso_on_triangulation(true);
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    let result = match scenario % 3 {
        0 => StdPrsIsolines::add(&prs, &face, &drawer, 0.2),
        1 => StdPrsIsolines::add_on_surface(&prs, &face, &drawer, 0.2),
        _ => StdPrsIsolines::add_on_triangulation(&prs, &face, &drawer),
    };
    isoline_presentation_snapshot(&prs, &events, scenario, result.is_err())
}

fn face_path_surface(scenario: i32) -> Handle<dyn rustycad_tkg3d::geom::GeomSurface> {
    let kind = if scenario < 24 && scenario / 12 != 0 {
        4
    } else {
        0
    };
    let surface = isoline_fixture_surface(kind);
    if scenario < 24 {
        return surface;
    }
    Handle::new(rustycad_tkg3d::geom::GeomOffsetSurface::new(&surface, 0.4, false).unwrap()).cast()
}

fn locate_face_path_topology(
    face: &mut Face,
    source: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    scenario: i32,
) {
    if scenario >= 27 {
        offset_face_path_surface_location(face, source);
    }
    if scenario % 6 >= 3 {
        face.set_location(&isoline_triangle_location(true).snapshot(), false)
            .unwrap();
    }
}

fn offset_face_path_surface_location(
    face: &Face,
    source: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
) {
    let mut shifted = rustycad_tkmath::gp::Transform3::default();
    shifted.set_translation(rustycad_tkmath::gp::Vector3::new(3., -2., 0.));
    let location = rustycad_tkmath::toploc::Location::new(shifted);
    BRepBuilder::default()
        .update_face(face, source, &location, 1e-7)
        .unwrap();
}
