//! Stateful manager fixture: reuse, clipping and immediate copy reconstruction.
use super::{
    common::{Reader, attempt, captured, end, flag, int, pnt, pnt2d, real, xyz},
    intersector_queries::intersector_pick_state,
    triangular::triangle_camera,
};
use rustycad_tkernel::{
    ncollection::{Array1, DynamicArray, Vec3, Vec4},
    standard::{Failure, Handle, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{Axis1, Direction3, GeneralTransform3, Point2, Point3, Transform3, Vector3, Xy},
};
use rustycad_tkservice::graphic3d::{Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane};
use rustycad_tkv3d::{
    select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager},
    select_mgr::*,
};
fn manager_pixels() -> Array1<'static, Point2> {
    let mut points = Array1::with_bounds(1, 4).unwrap();
    for (i, (x, y)) in [(200., 100.), (600., 100.), (600., 500.), (200., 500.)]
        .into_iter()
        .enumerate()
    {
        points
            .set_value(i as i32 + 1, Point2::from(Xy::new(x, y)))
            .unwrap();
    }
    points
}
fn manager_init(manager: &mut SelectMgrSelectingVolumeManager, kind: i32) -> Result<(), Failure> {
    match kind {
        0 => manager.init_selecting_volume(&Handle::default()),
        1 => manager.init_point_selecting_volume(Point2::from(Xy::new(400., 300.))),
        2 => manager.init_box_selecting_volume(
            Point2::from(Xy::new(200., 100.)),
            Point2::from(Xy::new(600., 500.)),
        ),
        3 => manager.init_polyline_selecting_volume(&manager_pixels())?,
        4 => manager.init_axis_selecting_volume(Axis1::new(
            Point3::new(0., 0., 10.),
            Direction3::new(0., 0., -1.)?,
        )),
        5 => manager.init_selecting_volume(&Handle::new(SelectMgrAxisIntersector::new()).cast()),
        _ => (),
    }
    Ok(())
}
fn manager_plane(z: f64) -> Result<Handle<Graphic3dSequenceOfHClipPlane>, Failure> {
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    planes.append(Handle::new(Graphic3dClipPlane::from_equation(Vec4::new(
        0., 0., 1., -z,
    ))?));
    Ok(planes)
}
fn manager_clips(
    manager: &mut SelectMgrSelectingVolumeManager,
    mode: i32,
    world_kind: i32,
) -> Result<(), Failure> {
    let mut range = SelectMgrViewClipRange::default();
    range.add_clip_sub_range(Range1::new(2., 4.)?);
    manager.set_view_clip_ranges(&range);
    if mode == 0 {
        return Ok(());
    }
    let mut world = SelectMgrSelectingVolumeManager::new();
    manager_init(&mut world, world_kind)?;
    let view = if mode >= 2 {
        manager_plane(5.)?
    } else {
        Handle::default()
    };
    let object = if mode >= 3 {
        manager_plane(6.)?
    } else {
        Handle::default()
    };
    attempt(|| {
        manager.set_view_clipping(
            &view,
            &object,
            if world_kind < 0 { None } else { Some(&world) },
        )
    });
    if mode == 4 {
        manager_init(manager, 2)?;
        if manager.camera().is_null() {
            manager.set_camera(&triangle_camera(false, false, 20.)?)?;
            manager.set_window_size(800, 600)?;
        }
        manager.set_view_clipping(&Handle::default(), &Handle::default(), None)?;
    }
    Ok(())
}
fn manager_copy(
    manager: &SelectMgrSelectingVolumeManager,
    mode: i32,
) -> Result<SelectMgrSelectingVolumeManager, Failure> {
    if mode == 0 {
        return Ok(manager.clone());
    }
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    builder.set_camera(&triangle_camera(false, false, 7.)?);
    builder.set_window_size(320, 240);
    if mode == 1 {
        return manager.copy_with_builder(&builder);
    }
    if mode == 2 {
        return manager.copy_with_builder(&Handle::default());
    }
    manager.scale_and_transform(20, &GeneralTransform3::default(), &builder)
}
fn manager_dump(manager: &SelectMgrSelectingVolumeManager, depth: i32) {
    captured(|output| {
        let mut stream = DumpStream::default();
        manager
            .dump_json(&mut stream, depth)
            .map_err(|e| Failure::new(&e.to_string()))?;
        let mut text = stream.text().to_owned();
        for field in [
            "ActiveSelectingVolume",
            "ViewClipPlanes",
            "ObjectClipPlanes",
        ] {
            let prefix = format!("\"{field}\": \"");
            let start = text.find(&prefix).expect("manager pointer field") + prefix.len();
            let end = start + text[start..].find('"').unwrap();
            text.replace_range(start..end, if end == start { "null" } else { "present" });
        }
        output.push_str(&text);
        Ok(())
    });
}
fn manager_state(manager: &SelectMgrSelectingVolumeManager, depth: i32) {
    int(manager.get_active_selection_type() as i32);
    flag(manager.is_overlap_allowed());
    flag(manager.is_scalable_active_volume());
    let (w, h) = manager.window_size().unwrap_or((123, 456));
    int(w);
    int(h);
    pnt2d(manager.get_mouse_position());
    attempt(|| {
        pnt(manager.get_near_picked_pnt()?);
        Ok(())
    });
    attempt(|| {
        pnt(manager.get_far_picked_pnt()?);
        Ok(())
    });
    attempt(|| {
        xyz(manager.get_view_ray_direction()?.xyz());
        Ok(())
    });
    attempt(|| {
        pnt(manager.detected_point(3.)?);
        Ok(())
    });
    attempt(|| {
        real(manager.dist_to_geometry_center(&Point3::new(1., 2., 3.))?);
        Ok(())
    });
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(11., 22., 33., 44.));
    manager.get_planes(&mut planes);
    int(planes.len() as i32);
    for plane in planes.iter() {
        for value in [plane.x, plane.y, plane.z, plane.w] {
            real(value);
        }
    }
    for depth in [-10., 0., 2., 3., 4., 5., 6., 10., 100.] {
        flag(manager.view_clip_ranges().is_clipped(depth));
    }
    manager_dump(manager, depth);
}
fn manager_linear_queries(
    manager: &SelectMgrSelectingVolumeManager,
    point: Point3,
    radius: f64,
    pick: &mut SelectBasicsPickResult,
) {
    let lo = Vec3::new(point.x() - radius, point.y() - radius, point.z() - radius);
    let hi = Vec3::new(point.x() + radius, point.y() + radius, point.z() + radius);
    let mut inside = true;
    attempt(|| {
        flag(manager.overlaps_box_inside(&lo, &hi, Some(&mut inside))?);
        Ok(())
    });
    flag(inside);
    attempt(|| {
        flag(manager.overlaps_box(&lo, &hi, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
    attempt(|| {
        flag(manager.overlaps_point_only(&point)?);
        Ok(())
    });
    attempt(|| {
        flag(manager.overlaps_point(&point, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
    let end = Point3::new(point.x() + radius, point.y(), point.z());
    attempt(|| {
        flag(manager.overlaps_segment(&point, &end, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
    let third = Point3::new(point.x(), point.y() + radius, point.z());
    attempt(|| {
        flag(manager.overlaps_triangle(&point, &end, &third, 0, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
    let mut points = Array1::with_bounds(-2, 0).unwrap();
    for (i, p) in [point, end, third].into_iter().enumerate() {
        points.set_value(i as i32 - 2, p).unwrap();
    }
    attempt(|| {
        flag(manager.overlaps_polygon(&points, 1, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
}
fn manager_quadric_queries(
    manager: &SelectMgrSelectingVolumeManager,
    point: Point3,
    radius: f64,
    pick: &mut SelectBasicsPickResult,
) {
    let mut inside = true;
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::from(point.xyz()));
    attempt(|| {
        flag(manager.overlaps_sphere_inside(&point, radius, Some(&mut inside))?);
        Ok(())
    });
    flag(inside);
    attempt(|| {
        flag(manager.overlaps_sphere(&point, radius, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
    attempt(|| {
        flag(manager.overlaps_cylinder_inside(
            radius,
            0.,
            3.,
            &transform,
            false,
            Some(&mut inside),
        )?);
        Ok(())
    });
    flag(inside);
    attempt(|| {
        flag(manager.overlaps_cylinder(radius, 0., 3., &transform, false, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
    attempt(|| {
        flag(manager.overlaps_circle_inside(radius, &transform, true, Some(&mut inside))?);
        Ok(())
    });
    flag(inside);
    attempt(|| {
        flag(manager.overlaps_circle(radius, &transform, true, pick)?);
        Ok(())
    });
    intersector_pick_state(pick);
}
fn manager_case_inner(r: &mut Reader<'_>) -> Result<(), Failure> {
    let (kind, built, next, allow, late, copy, clip, world, depth) = (
        r.integer(),
        r.flag(),
        r.integer(),
        r.flag(),
        r.flag(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let (point, radius) = (r.pnt(), r.real());
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.allow_overlap_detection(allow);
    manager_init(&mut manager, kind)?;
    let camera = triangle_camera(false, false, 20.)?;
    attempt(|| manager.set_camera(&camera));
    attempt(|| manager.set_window_size(800, 600));
    attempt(|| manager.set_pixel_tolerance(20));
    attempt(|| manager.set_viewport(0., 0., 1., 1.));
    if built {
        attempt(|| manager.build_selecting_volume());
    }
    let old = manager.active_volume().clone();
    manager_init(&mut manager, next)?;
    flag(old == *manager.active_volume());
    flag(manager.camera() == camera);
    if !manager.active_volume().is_null() && manager.camera().is_null() {
        manager.set_camera(&camera)?;
        manager.set_window_size(800, 600)?;
        if built {
            attempt(|| manager.build_selecting_volume());
        }
    }
    manager.allow_overlap_detection(late);
    manager_clips(&mut manager, clip, world)?;
    let manager = manager_copy(&manager, copy)?;
    manager_state(&manager, depth);
    let mut pick = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    pick.set_surface_normal_vec(&Vector3::new(1., 2., 3.));
    manager_linear_queries(&manager, point, radius, &mut pick);
    manager_quadric_queries(&manager, point, radius, &mut pick);
    Ok(())
}
/// Compare manager transitions with OCCT; `selecting_volume_case(&values)`.
pub fn selecting_volume_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| manager_case_inner(&mut r));
    end();
}
