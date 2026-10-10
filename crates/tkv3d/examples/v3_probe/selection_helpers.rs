//! Twin of tools/oracle/tkv3d_probe_selection_helpers.cxx.
use super::common::{Reader, attempt, captured, end, flag, int, pnt, real};
use rustycad_tkernel::{
    ncollection::{Vec3, Vec4},
    standard::{Failure, Handle},
};
use rustycad_tkmath::{bnd::Range1, gp::Axis1};
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dCameraProjection, Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane,
};
use rustycad_tkv3d::select_mgr::{SelectMgrFrustumBuilder, SelectMgrViewClipRange};

fn setup_builder(builder: &SelectMgrFrustumBuilder, r: &mut Reader<'_>) -> Result<(), Failure> {
    let (present, perspective, zero) = (r.flag(), r.flag(), r.flag());
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(if perspective {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
    camera.set_zero_to_one_depth(zero);
    camera.set_z_range(0.1, 100.)?;
    if present {
        builder.set_camera(&camera);
    }
    builder.set_window_size(r.integer(), r.integer());
    let (viewport, x, y, w, h) = (r.flag(), r.real(), r.real(), r.real(), r.real());
    if viewport {
        builder.set_viewport(x, y, w, h);
    }
    flag(builder.camera().object_address() == camera.object_address());
    Ok(())
}
/// Compare projection, viewport invalidation and plane distance; `frustum_builder_case(&values)`.
pub fn frustum_builder_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let builder = SelectMgrFrustumBuilder::new();
    let (w, h) = builder.window_size();
    int(w);
    int(h);
    flag(builder.camera().is_null());
    pnt(builder.project_pnt_on_view_plane(1., 2., 0.));
    attempt(|| setup_builder(&builder, &mut r));
    let (x, y, z) = (r.real(), r.real(), r.real());
    let eq = r.xyz();
    let point = r.xyz();
    let (w, h) = builder.window_size();
    int(w);
    int(h);
    let point = Vec3::new(point.x(), point.y(), point.z());
    real(builder.signed_plane_pnt_dist(&Vec3::new(eq.x(), eq.y(), eq.z()), &point));
    real(builder.signed_plane_pnt_dist(&Vec3::new(0., 0., 0.), &point));
    pnt(builder.project_pnt_on_view_plane(x, y, z));
    builder.invalidate_viewport();
    pnt(builder.project_pnt_on_view_plane(x, y, z));
    end();
}
fn clip_planes(r: &mut Reader<'_>) -> Result<Graphic3dSequenceOfHClipPlane, Failure> {
    let sequence = Graphic3dSequenceOfHClipPlane::default();
    let mut previous = Handle::default();
    for _ in 0..r.integer() {
        let equation = Vec4::new(r.real(), r.real(), r.real(), r.real());
        let (on, linked) = (r.flag(), r.flag());
        let plane = Handle::new(Graphic3dClipPlane::from_equation(equation)?);
        plane.set_on(on)?;
        if linked {
            Graphic3dClipPlane::set_chain_next_plane(&previous, plane.clone())?;
        } else {
            sequence.append(plane.clone());
        }
        previous = plane;
    }
    Ok(sequence)
}
fn clip_queries(clip: &mut SelectMgrViewClipRange, r: &mut Reader<'_>) {
    for _ in 0..r.integer() {
        let (first, last) = (r.real(), r.real());
        let mut depth = 123.;
        let query = if first <= last {
            Range1::new(first, last).unwrap()
        } else {
            Range1::default()
        };
        flag(clip.is_clipped(first));
        flag(clip.is_clipped(last));
        flag(clip.get_nearest_depth_into(&query, &mut depth));
        real(depth);
    }
    captured(|output| {
        let mut stream = rustycad_tkernel::standard::dump::DumpStream::default();
        let result = clip.dump_json(&mut stream, -1);
        output.push_str(stream.text());
        result.map_err(|error| Failure::new(&error.to_string()))
    });
    clip.set_void();
    flag(clip.is_clipped(0.));
}
/// Compare range/plane chains and failed lookup output; `clip_range_case(&values)`.
pub fn clip_range_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut clip = SelectMgrViewClipRange::default();
    let (empty, first, last) = (r.flag(), r.real(), r.real());
    *clip.change_unclip_range() = if empty {
        Range1::default()
    } else {
        Range1::new(first, last).unwrap()
    };
    for _ in 0..r.integer() {
        clip.add_clip_sub_range(Range1::new(r.real(), r.real()).unwrap());
    }
    attempt(|| {
        let planes = clip_planes(&mut r)?;
        let ray = Axis1::new(r.pnt(), r.dir());
        clip.add_clipping_planes(&planes, &ray)?;
        clip_queries(&mut clip, &mut r);
        Ok(())
    });
    end();
}
