//! Native set fixture with input sharing, child order and bounded-array failures exposed.
use super::{
    axis::axis_clip,
    common::{Reader, attempt, captured, end, flag, int, pnt, pnt2d, real},
    intersector_queries::intersector_query,
    triangular::{triangle_camera, triangle_transform},
};
use rustycad_tkernel::{
    ncollection::{Array1, DynamicArray, Vec4},
    standard::{Failure, Handle, RuntimeTyped, dump::DumpStream},
};
use rustycad_tkmath::gp::Point2;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::select_mgr::*;
fn set_points(r: &mut Reader<'_>) -> Result<Array1<'static, Point2>, Failure> {
    let (lower, count) = (r.integer(), r.integer());
    let mut points = Array1::with_bounds(lower, lower + count - 1)
        .map_err(|e| e.into_failure("set fixture points"))?;
    for index in lower..=points.upper() {
        points
            .set_value(index, r.pnt2d())
            .map_err(|e| e.into_failure("set fixture point"))?;
    }
    Ok(points)
}
fn set_dump_pointers(mut text: String) -> String {
    for field in ["Builder", "Camera"] {
        let prefix = format!("\"{field}\": \"");
        let mut offset = 0;
        while let Some(start) = text[offset..].find(&prefix) {
            let start = offset + start + prefix.len();
            let end = start + text[start..].find('"').expect("dump pointer quote");
            let value = if end == start { "null" } else { "present" };
            text.replace_range(start..end, value);
            offset = start + value.len();
        }
    }
    text
}
fn set_state(volume: &SelectMgrTriangularFrustumSet, r: &mut Reader<'_>) -> Result<(), Failure> {
    let (rebuild, depth) = (r.flag(), r.integer());
    if rebuild {
        volume.build()?;
    }
    flag(volume.is_scalable());
    int(volume.get_selection_type() as i32);
    pnt2d(volume.get_mouse_position());
    attempt(|| {
        pnt(volume.detected_point(3.)?);
        Ok(())
    });
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(11., 22., 33., 44.));
    volume.get_planes(&mut planes);
    int(planes.len() as i32);
    for plane in planes.iter() {
        for value in [plane.x, plane.y, plane.z, plane.w] {
            real(value);
        }
    }
    captured(|output| {
        let mut stream = DumpStream::default();
        volume
            .dump_json(&mut stream, depth)
            .map_err(|e| Failure::new(&e.to_string()))?;
        output.push_str(&set_dump_pointers(stream.text().to_owned()));
        Ok(())
    });
    Ok(())
}
struct SetFixture {
    original: Handle<SelectMgrTriangularFrustumSet>,
    points: Array1<'static, Point2>,
    camera: Handle<Graphic3dCamera>,
    other: Handle<Graphic3dCamera>,
}
fn set_fixture(r: &mut Reader<'_>) -> Result<SetFixture, Failure> {
    let (perspective, zero, viewport, allow, initialized, build) =
        (r.flag(), r.flag(), r.flag(), r.flag(), r.flag(), r.flag());
    let points = set_points(r)?;
    let camera = triangle_camera(perspective, zero, 20.)?;
    let other = triangle_camera(!perspective, zero, 7.)?;
    let original = Handle::new(SelectMgrTriangularFrustumSet::new());
    original.set_camera(&camera);
    original.set_window_size(800, 600);
    if viewport {
        original.set_viewport(0.1, 0.2, 0.9, 0.8);
    }
    original.set_pixel_tolerance(17);
    original.set_allow_overlap_detection(allow);
    if initialized {
        original.init(&points)?;
    }
    if build {
        original.build()?;
    }
    Ok(SetFixture {
        original,
        points,
        camera,
        other,
    })
}
fn set_transition(
    fixture: &mut SetFixture,
    r: &mut Reader<'_>,
) -> Result<Handle<SelectMgrTriangularFrustumSet>, Failure> {
    let (transition, scale, builder_kind) = (r.integer(), r.integer(), r.integer());
    let transform = triangle_transform(r)?;
    let mut builder = Handle::default();
    if builder_kind != 0 {
        builder = Handle::new(SelectMgrFrustumBuilder::new());
        builder.set_camera(if builder_kind == 1 {
            &fixture.other
        } else {
            &fixture.camera
        });
        builder.set_window_size(320, 240);
    }
    let mut volume = fixture.original.clone();
    if transition == 1 {
        volume = volume
            .scale_and_transform(scale, &transform, &builder)?
            .cast();
    }
    if transition >= 2 {
        volume = volume.copy_with_builder(&builder)?.cast();
    }
    if transition >= 4 {
        for point in fixture.points.iter_mut() {
            point.set_x(point.x() + 80.);
        }
        fixture.original.init(&fixture.points)?;
    }
    if transition == 3 || transition == 4 {
        volume.build()?;
    }
    Ok(volume)
}
fn set_case_inner(r: &mut Reader<'_>) -> Result<(), Failure> {
    let mut fixture = set_fixture(r)?;
    let volume = set_transition(&mut fixture, r)?;
    flag(volume.camera().object_address() == fixture.camera.object_address());
    flag(volume.camera().object_address() == fixture.other.object_address());
    captured(|output| {
        output.push_str(volume.dynamic_type().expect("set descriptor").name());
        Ok(())
    });
    captured(|output| {
        output.push_str(SelectMgrTriangularFrustumSet::static_type().name());
        Ok(())
    });
    flag(volume.is_kind_name("SelectMgr_TriangularFrustumSet"));
    flag(volume.is_instance_name("SelectMgr_BaseFrustum"));
    if r.integer() == 0 {
        return set_state(&volume, r);
    }
    let clip = axis_clip(r)?;
    intersector_query(volume.get().expect("set handle"), r, &clip)
}
/// Compare native polyline volumes and their child ordering; `frustum_set_case(&values)`.
pub fn frustum_set_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| set_case_inner(&mut r));
    end();
}
