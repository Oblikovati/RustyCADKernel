//! Twin of tools/oracle/tkv3d_probe_base_frustum.cxx.
#[path = "../../tests/support/frustum_defaults_fake.rs"]
pub(super) mod frustum_defaults_fake;
use super::common::{Reader, attempt, captured, end, flag, int, pnt};
use frustum_defaults_fake::FrustumDefaultsIntersector;
use rustycad_tkernel::standard::{Failure, Handle, dump::DumpStream};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::select_mgr::*;

fn frustum_camera(volume: &FrustumDefaultsIntersector, r: &mut Reader<'_>) -> Result<(), Failure> {
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(if r.flag() {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
    camera.set_zero_to_one_depth(r.flag());
    camera.set_z_range(0.1, 100.)?;
    volume.set_camera(&camera);
    let builder = volume.base.builder.read().clone();
    flag(volume.camera().object_address() == camera.object_address());
    flag(builder.camera().object_address() == camera.object_address());
    volume.set_viewport(0.1, 0.2, 0.9, 0.8);
    pnt(builder.project_pnt_on_view_plane(123., 231., 0.2));
    volume.set_camera(&camera);
    pnt(builder.project_pnt_on_view_plane(123., 231., 0.2));
    replace_builder(volume, &camera);
    Ok(())
}
fn replace_builder(volume: &FrustumDefaultsIntersector, camera: &Handle<Graphic3dCamera>) {
    let replacement = Handle::new(SelectMgrFrustumBuilder::new());
    replacement.set_window_size(321, 123);
    volume.set_builder(&replacement);
    flag(volume.camera().is_null());
    flag(volume.base.builder.read().object_address() == replacement.object_address());
    volume.set_camera(camera);
    volume.set_builder(&Handle::default());
    flag(volume.camera().object_address() == camera.object_address());
    flag(volume.base.builder.read().is_null());
    volume.set_camera(&Handle::default());
    flag(volume.camera().is_null());
}
fn frustum_state(volume: &FrustumDefaultsIntersector, r: &mut Reader<'_>) {
    let other = FrustumDefaultsIntersector::default();
    let (w, h) = volume.window_size().unwrap();
    int(w);
    int(h);
    int(*volume.base.pixel_tolerance.read());
    flag(volume.base.builder.read().object_address() != other.base.builder.read().object_address());
    volume.set_pixel_tolerance(r.integer());
    volume.set_window_size(r.integer(), r.integer());
    let (w, h) = volume.window_size().unwrap();
    int(w);
    int(h);
    int(*volume.base.pixel_tolerance.read());
}
/// Compare propagation, replacement and JSON at each depth; `base_frustum_case(&values)`.
pub fn base_frustum_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let volume = FrustumDefaultsIntersector::default();
    frustum_state(&volume, &mut r);
    attempt(|| frustum_camera(&volume, &mut r));
    let depth = r.integer();
    captured(|output| {
        let mut stream = DumpStream::default();
        let result = volume.dump_json(&mut stream, depth);
        output.push_str(stream.text());
        result.map_err(|error| Failure::new(&error.to_string()))
    });
    end();
}
/// Compare projection, closed-boundary convention, virtual calls and output flags; `boundary_sphere_case(&values)`.
pub fn boundary_sphere_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let volume = FrustumDefaultsIntersector {
        reject_sphere_rays: r.flag(),
        ..Default::default()
    };
    let (center, radius, normal) = (r.pnt(), r.real(), r.dir());
    let mut inside = r.flag();
    let (_lower, count) = (r.integer(), r.integer());
    let points = (0..count).map(|_| r.pnt()).collect::<Vec<_>>();
    attempt(|| {
        flag(volume.is_boundary_intersect_sphere(
            &center,
            radius,
            &normal,
            &points,
            &mut inside,
        )?);
        Ok(())
    });
    flag(inside);
    end();
}
