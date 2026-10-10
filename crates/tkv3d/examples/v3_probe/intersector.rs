//! Twin of tools/oracle/tkv3d_probe_intersector.cxx.
#[path = "../../tests/support/base_defaults_fake.rs"]
mod base_defaults_fake;
use super::common::{Reader, attempt, captured, end, flag, int, pnt, real};
use base_defaults_fake::BaseDefaultsIntersector;
use rustycad_tkernel::ncollection::{DynamicArray, Vec4};
use rustycad_tkernel::standard::{Failure, Handle, dump::DumpStream};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::select_mgr::{SelectMgrBaseIntersector, SelectMgrBaseIntersectorMembers};

/// Compare successful and untouched sphere outputs; `ray_sphere_case(&values)`.
pub fn ray_sphere_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let volume = BaseDefaultsIntersector::default();
    let hit = volume.ray_sphere_intersection(&r.pnt(), r.real(), &r.pnt(), &r.dir());
    flag(hit.is_some());
    let (enter, leave) = hit.unwrap_or((123., 456.));
    real(enter);
    real(leave);
    end();
}
/// Compare capped/hollow cylinder and cone outputs; `ray_cylinder_case(&values)`.
pub fn ray_cylinder_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let volume = BaseDefaultsIntersector::default();
    attempt(|| {
        let hit = volume.ray_cylinder_intersection(
            r.real(),
            r.real(),
            r.real(),
            &r.pnt(),
            &r.dir(),
            r.flag(),
        )?;
        flag(hit.is_some());
        let (enter, leave) = hit.unwrap_or((123., None));
        real(enter);
        real(leave.unwrap_or(456.));
        Ok(())
    });
    end();
}
/// Compare filled disks and circle outlines; `ray_circle_case(&values)`.
pub fn ray_circle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let volume = BaseDefaultsIntersector::default();
    let hit = volume.ray_circle_intersection(r.real(), &r.pnt(), &r.dir(), r.flag());
    flag(hit.is_some());
    real(hit.unwrap_or(123.));
    end();
}
fn base_geometry(volume: &BaseDefaultsIntersector) {
    pnt(volume.get_near_pnt().unwrap());
    pnt(volume.get_far_pnt().unwrap());
    let direction = volume.get_view_ray_direction().unwrap();
    real(direction.x());
    real(direction.y());
    real(direction.z());
    let mouse = volume.get_mouse_position();
    real(mouse.x());
    real(mouse.y());
    real(
        volume
            .dist_to_geometry_center(&Point3::new(1., 2., 3.))
            .unwrap(),
    );
    pnt(volume.detected_point(10.).unwrap());
    base_planes(volume);
}
/// Compare base no-ops, sentinels, camera identity and JSON; `base_intersector_case(&values)`.
pub fn base_intersector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let volume = BaseDefaultsIntersector::default();
    volume.set_pixel_tolerance(4);
    volume.set_window_size(640, 480);
    volume.set_viewport(0., 0., 1., 1.);
    let (width, height) = volume.window_size().unwrap_or((123, 456));
    int(width);
    int(height);
    int(volume.get_selection_type() as i32);
    flag(volume.camera().is_null());
    base_geometry(&volume);
    base_camera(&volume, r.flag());
    base_dump(&volume);
    end();
}

fn base_camera(volume: &BaseDefaultsIntersector, perspective: bool) {
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(if perspective {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
    volume.set_camera(&camera);
    flag(volume.camera().object_address() == camera.object_address());
    flag(volume.is_orthographic());
    volume.set_camera(&Handle::default());
    flag(volume.camera().is_null());
}

fn base_dump(volume: &BaseDefaultsIntersector) {
    captured(|output| {
        let mut stream = DumpStream::default();
        let result = volume.dump_json(&mut stream, -1);
        output.push_str(stream.text());
        result.map_err(|error| Failure::new(&error.to_string()))
    });
}

fn base_planes(volume: &BaseDefaultsIntersector) {
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(1., 2., 3., 4.));
    volume.get_planes(&mut planes);
    int(planes.len() as i32);
}
