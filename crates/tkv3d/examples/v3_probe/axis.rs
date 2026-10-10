//! Native AxisIntersector counterpart, including initialization failures and pick mutation.
use super::{
    common::{Reader, attempt, captured, end, flag, int, pnt, pnt2d, real, xyz},
    intersector_queries::{intersector_pick_state, intersector_query},
    triangular::triangle_transform,
};
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Failure, Handle, RuntimeTyped, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{Axis1, GeneralTransform3, Point3, Vector3},
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::{select_basics::SelectBasicsPickResult, select_mgr::*};
pub(super) fn axis_clip(r: &mut Reader<'_>) -> Result<SelectMgrViewClipRange, Failure> {
    let mut clip = SelectMgrViewClipRange::default();
    let (empty, first, last) = (r.flag(), r.real(), r.real());
    if empty {
        clip.change_unclip_range().set_void();
    } else {
        *clip.change_unclip_range() = Range1::new(first, last)?;
    }
    for _ in 0..r.integer() {
        clip.add_clip_sub_range(Range1::new(r.real(), r.real())?);
    }
    Ok(clip)
}
fn axis_identity(volume: &dyn SelectMgrBaseIntersector) {
    volume.set_camera(&Handle::new(Graphic3dCamera::default()));
    volume.set_pixel_tolerance(17);
    volume.set_window_size(10, 20);
    volume.set_viewport(0.1, 0.2, 0.3, 0.4);
    flag(volume.camera().is_null());
    flag(volume.is_scalable());
    int(volume.get_selection_type() as i32);
    let (width, height) = volume.window_size().unwrap_or((123, 456));
    int(width);
    int(height);
    captured(|output| {
        output.push_str(volume.dynamic_type().name());
        Ok(())
    });
    captured(|output| {
        output.push_str(SelectMgrAxisIntersector::static_type().name());
        Ok(())
    });
    flag(volume.is_kind_name("SelectMgr_AxisIntersector"));
    flag(volume.is_instance_name("SelectMgr_BaseIntersector"));
}
fn axis_state(volume: &dyn SelectMgrBaseIntersector, r: &mut Reader<'_>) -> Result<(), Failure> {
    let (depth, distance, point) = (r.integer(), r.real(), r.pnt());
    volume.build()?;
    axis_identity(volume);
    attempt(|| {
        pnt(volume.get_near_pnt()?);
        Ok(())
    });
    attempt(|| {
        pnt(volume.get_far_pnt()?);
        Ok(())
    });
    attempt(|| {
        xyz(volume.get_view_ray_direction()?.xyz());
        Ok(())
    });
    pnt2d(volume.get_mouse_position());
    attempt(|| {
        real(volume.dist_to_geometry_center(&point)?);
        Ok(())
    });
    attempt(|| {
        pnt(volume.detected_point(distance)?);
        Ok(())
    });
    captured(|output| {
        let mut stream = DumpStream::default();
        volume
            .dump_json(&mut stream, depth)
            .map_err(|e| Failure::new(&e.to_string()))?;
        output.push_str(stream.text());
        Ok(())
    });
    Ok(())
}
fn axis_protected(volume: &SelectMgrAxisIntersector, r: &mut Reader<'_>) {
    let kind = r.integer();
    let (mut entry, mut leave) = (123., 456.);
    let mut pick = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    pick.set_surface_normal_vec(&Vector3::new(1., 2., 3.));
    match kind {
        0 => {
            let (a, b) = (r.xyz(), r.xyz());
            let hit = volume.has_intersection_box(
                &Vec3::new(a.x(), a.y(), a.z()),
                &Vec3::new(b.x(), b.y(), b.z()),
            );
            flag(hit.is_some());
            if let Some(times) = hit {
                (entry, leave) = times;
            }
        }
        1 => {
            let point = r.pnt();
            attempt(|| {
                let hit = volume.has_intersection_point(&point)?;
                flag(hit.is_some());
                if let Some(depth) = hit {
                    entry = depth;
                }
                Ok(())
            });
        }
        2 => flag(volume.ray_segment_distance(&r.pnt(), &r.pnt(), &mut pick)),
        3 => flag(volume.ray_plane_intersection(&Vector3::from(r.xyz()), &r.pnt(), &mut pick)),
        _ => panic!("axis protected kind={kind}; expected 0..=3"),
    }
    real(entry);
    real(leave);
    intersector_pick_state(&pick);
}
fn axis_case_inner(r: &mut Reader<'_>) -> Result<(), Failure> {
    let (initialized, point, direction) = (r.flag(), r.pnt(), r.dir());
    let original = Handle::new(SelectMgrAxisIntersector::new());
    if initialized {
        original.init(Axis1::new(point, direction));
    }
    let mut volume: Handle<dyn SelectMgrBaseIntersector> = original.cast();
    let (transition, matrix_mode, scale) = (r.integer(), r.integer(), r.integer());
    let matrix = triangle_transform(r)?;
    let transform = if matrix_mode == 0 {
        GeneralTransform3::default()
    } else {
        matrix
    };
    if transition == 1 {
        volume = volume.scale_and_transform(scale, &transform, &Handle::default())?;
    }
    if transition == 2 {
        volume = volume.copy_with_builder(&Handle::default())?;
    }
    match r.integer() {
        0 => axis_state(volume.get().expect("axis base handle"), r)?,
        1 => {
            let clip = axis_clip(r)?;
            intersector_query(volume.get().expect("axis base handle"), r, &clip)?;
        }
        2 => axis_protected(&original, r),
        kind => panic!("axis family={kind}; expected 0..=2"),
    }
    Ok(())
}
/// Compare native axis state and all intersections; `axis_case(&values)`.
pub fn axis_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| axis_case_inner(&mut r));
    end();
}
