//! Graphic3d_CullingTool, the twin of tools/oracle/tkservice_probe_culling.cxx.
use super::camera::read_camera;
use super::common::{Reader, end, flag, int, real};
use rustycad_tkernel::ncollection::{Mat4, Vec3, Vec4};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{Graphic3dCullingContext, Graphic3dCullingTool};

fn vec3(v: &Vec3<f64>) {
    [v.x, v.y, v.z].into_iter().for_each(real);
}

fn boxes(r: &mut Reader<'_>, tool: &Graphic3dCullingTool, ctx: &Graphic3dCullingContext) {
    for _ in 0..r.integer() {
        let (a, b) = (r.xyz(), r.xyz());
        let (min, max) = (
            Vec3::new(a.x(), a.y(), a.z()),
            Vec3::new(b.x(), b.y(), b.z()),
        );
        flag(tool.is_out_frustum(&min, &max, None));
        let mut in1 = true;
        flag(tool.is_out_frustum(&min, &max, Some(&mut in1)));
        flag(in1);
        let mut in2 = true;
        flag(tool.is_too_distant(ctx, &min, &max, Some(&mut in2)));
        flag(in2);
        flag(tool.is_too_small(ctx, &min, &max));
        let mut in3 = true;
        flag(tool.is_culled(ctx, &min, &max, Some(&mut in3)));
        flag(in3);
        flag(tool.is_culled(ctx, &min, &max, None));
    }
}

/// `sv_culling`: a culling tool over a camera, its context and the box tests.
pub fn culling_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let camera = Handle::new(read_camera(&mut r));
    let t = r.xyz();
    let (w, h) = (r.integer(), r.integer());
    let [ratio, distance, size] = [0; 3].map(|_| r.real());
    let mut model = Mat4::<f64>::identity();
    (0..3).for_each(|row| model.set_value(row, 3, [t.x(), t.y(), t.z()][row]));
    let mut tool = Graphic3dCullingTool::default();
    tool.set_view_volume(&camera, &model);
    tool.set_viewport_size(w, h, ratio);
    tool.cache_clip_pts_projections();
    let mut ctx = Graphic3dCullingContext::default();
    tool.set_culling_distance(&mut ctx, distance);
    tool.set_culling_size(&mut ctx, size);
    real(ctx.dist_cull);
    real(ctx.size_cull2);
    int(tool.viewport_width());
    int(tool.viewport_height());
    vec3(tool.camera_eye());
    vec3(tool.camera_direction());
    real(tool.signed_plane_point_distance(
        &Vec4::new(1.0, 2.0, 2.0, -3.0),
        &Vec4::new(t.x(), t.y(), t.z(), 1.0),
    ));
    boxes(&mut r, &tool, &ctx);
    end();
}
