//! Concrete rectangular fixture, matching native state and output argument sequencing.
use super::{
    axis::axis_clip,
    common::{Reader, attempt, captured, end, flag, int, pnt, pnt2d, real, xyz},
    frustum_core::pointer_identities,
    intersector_queries::{intersector_pick_state, intersector_query},
    triangular::{triangle_camera, triangle_transform},
};
use rustycad_tkernel::{
    ncollection::{DynamicArray, Vec4},
    standard::{Failure, Handle, RuntimeTyped, dump::DumpStream},
};
use rustycad_tkmath::gp::{GeneralTransform3, Point3, Vector3};
use rustycad_tkv3d::{select_basics::SelectBasicsPickResult, select_mgr::*};
fn rectangle_state(
    volume: &SelectMgrRectangularFrustum,
    r: &mut Reader<'_>,
) -> Result<(), Failure> {
    let (rebuild, depth) = (r.flag(), r.integer());
    if rebuild {
        volume.build()?;
    }
    flag(volume.is_scalable());
    int(volume.get_selection_type() as i32);
    pnt(volume.get_near_pnt()?);
    pnt(volume.get_far_pnt()?);
    xyz(volume.get_view_ray_direction()?.xyz());
    pnt2d(volume.get_mouse_position());
    attempt(|| {
        real(volume.dist_to_geometry_center(&Point3::new(1., 2., 3.))?);
        Ok(())
    });
    attempt(|| {
        pnt(volume.detected_point(7.)?);
        Ok(())
    });
    for vertex in volume.get_vertices() {
        pnt(vertex);
    }
    rectangle_planes(volume);
    captured(|output| {
        let mut stream = DumpStream::default();
        volume
            .dump_json(&mut stream, depth)
            .map_err(|e| Failure::new(&e.to_string()))?;
        output.push_str(&if depth == 0 {
            stream.text().to_owned()
        } else {
            pointer_identities(stream.text().to_owned())
        });
        Ok(())
    });
    Ok(())
}
fn rectangle_planes(volume: &SelectMgrRectangularFrustum) {
    if volume.camera().is_null() {
        return;
    }
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(11., 22., 33., 44.));
    volume.get_planes(&mut planes);
    int(planes.len() as i32);
    for plane in planes.iter() {
        for value in [plane.x, plane.y, plane.z, plane.w] {
            real(value);
        }
    }
}
fn rectangle_protected(
    volume: &SelectMgrRectangularFrustum,
    r: &mut Reader<'_>,
) -> Result<(), Failure> {
    let kind = r.integer();
    let mut pick = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    pick.set_surface_normal_vec(&Vector3::new(1., 2., 3.));
    match kind {
        0 => volume.segment_segment_distance(&r.pnt(), &r.pnt(), &mut pick),
        1 => flag(volume.segment_plane_intersection(&Vector3::from(r.xyz()), &r.pnt(), &mut pick)),
        2 => flag(volume.is_segments_intersect(r.pnt(), r.pnt(), r.pnt(), r.pnt())),
        3 => {
            let (radius, center, transform, points) = (
                r.real(),
                r.pnt(),
                super::frustum_core::transformation(r)?,
                super::frustum_core::points(r),
            );
            attempt(|| {
                flag(volume.is_intersect_circle(radius, center, transform, &points)?);
                Ok(())
            });
        }
        _ => panic!("rectangle protected kind={kind}; expected 0..=3"),
    }
    intersector_pick_state(&pick);
    Ok(())
}
fn rectangle_bounds(r: &mut Reader<'_>) {
    let mut bounds = SelectMgrSelectionRectangle::new();
    pnt2d(bounds.min_pnt());
    pnt2d(bounds.max_pnt());
    let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
    bounds.set_mouse_pos(a);
    pnt2d(bounds.mouse_pos());
    pnt2d(bounds.max_pnt());
    bounds.set_min_pnt(b);
    bounds.set_max_pnt(c);
    pnt2d(bounds.mouse_pos());
    pnt2d(bounds.min_pnt());
    pnt2d(bounds.max_pnt());
}
struct RectangleFixture {
    original: Handle<SelectMgrRectangularFrustum>,
    camera: Handle<rustycad_tkservice::graphic3d::Graphic3dCamera>,
    other: Handle<rustycad_tkservice::graphic3d::Graphic3dCamera>,
}
fn rectangle_fixture(r: &mut Reader<'_>) -> Result<RectangleFixture, Failure> {
    let (perspective, zero, viewport, selection, a, b, tolerance, build) = (
        r.flag(),
        r.flag(),
        r.flag(),
        r.integer(),
        r.pnt2d(),
        r.pnt2d(),
        r.integer(),
        r.flag(),
    );
    let camera = triangle_camera(perspective, zero, 20.)?;
    let other = triangle_camera(!perspective, zero, 7.)?;
    let original = Handle::new(SelectMgrRectangularFrustum::new());
    original.set_camera(&camera);
    original.set_window_size(800, 600);
    if viewport {
        original.set_viewport(0.1, 0.2, 0.9, 0.8);
    }
    original.set_pixel_tolerance(tolerance);
    if selection == 1 {
        original.init(a);
    }
    if selection == 2 {
        original.init_box(a, b);
    }
    if build {
        original.build()?;
    }
    Ok(RectangleFixture {
        original,
        camera,
        other,
    })
}
fn rectangle_transition(
    fixture: &RectangleFixture,
    r: &mut Reader<'_>,
) -> Result<Handle<SelectMgrRectangularFrustum>, Failure> {
    let (transition, scale, mode, builder_kind) =
        (r.integer(), r.integer(), r.integer(), r.integer());
    let matrix = triangle_transform(r)?;
    let transform = if mode == 0 {
        GeneralTransform3::default()
    } else {
        matrix
    };
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
    if transition == 2 || transition == 3 {
        volume = volume.copy_with_builder(&builder)?.cast();
    }
    if transition == 3 {
        volume.build()?;
    }
    Ok(volume)
}
fn rectangle_case_inner(r: &mut Reader<'_>) -> Result<(), Failure> {
    let fixture = rectangle_fixture(r)?;
    let volume = rectangle_transition(&fixture, r)?;
    flag(volume.camera().object_address() == fixture.camera.object_address());
    flag(volume.camera().object_address() == fixture.other.object_address());
    captured(|output| {
        output.push_str(
            volume
                .dynamic_type()
                .expect("rectangular runtime descriptor")
                .name(),
        );
        Ok(())
    });
    captured(|output| {
        output.push_str(SelectMgrRectangularFrustum::static_type().name());
        Ok(())
    });
    flag(volume.is_kind_name("SelectMgr_RectangularFrustum"));
    flag(volume.is_instance_name("SelectMgr_BaseFrustum"));
    match r.integer() {
        0 => rectangle_state(&volume, r)?,
        1 => {
            let clip = axis_clip(r)?;
            intersector_query(volume.get().unwrap(), r, &clip)?;
        }
        2 => rectangle_protected(&fixture.original, r)?,
        3 => rectangle_bounds(r),
        kind => panic!("rectangle family={kind}; expected 0..=3"),
    }
    Ok(())
}
/// Compare concrete rectangular volumes and protected helpers; `rectangular_case(&values)`.
pub fn rectangular_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| rectangle_case_inner(&mut r));
    end();
}
