//! Exact protected-frustum counterpart; raw cached arrays are supplied by the corpus.
use super::base_frustum::frustum_defaults_fake::FrustumDefaultsIntersector;
use super::common::{Reader, attempt, captured, end, flag, xyz};
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Failure, Handle, dump::DumpStream},
};
use rustycad_tkmath::gp::{Axis1, Point3, Transform3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::select_mgr::*;

fn frustum_core<const N: usize>(r: &mut Reader<'_>) -> SelectMgrFrustumCore<N> {
    let mut core = SelectMgrFrustumCore::new();
    for plane in core.planes_mut() {
        *plane = r.xyz();
    }
    for vertex in core.vertices_mut() {
        *vertex = r.pnt();
    }
    for value in core.max_verts_projections_mut() {
        *value = r.real();
    }
    for value in core.min_verts_projections_mut() {
        *value = r.real();
    }
    for value in core.max_ortho_verts_projections_mut() {
        *value = r.real();
    }
    for value in core.min_ortho_verts_projections_mut() {
        *value = r.real();
    }
    for edge in core.edge_dirs_mut() {
        *edge = r.xyz();
    }
    core
}
pub(super) fn points(r: &mut Reader<'_>) -> Vec<Point3> {
    let (_lower, count) = (r.integer(), r.integer());
    (0..count).map(|_| r.pnt()).collect()
}
pub(super) fn transformation(r: &mut Reader<'_>) -> Result<Transform3, Failure> {
    let (axis, angle, shift, scale) = (r.dir(), r.real(), r.xyz(), r.real());
    let mut transform = Transform3::default();
    transform.set_rotation(Axis1::new(Point3::default(), axis), angle)?;
    transform.set_translation_part(Vector3::from(shift));
    transform.set_scale_factor(scale)?;
    Ok(transform)
}
fn box_query<const N: usize>(core: &SelectMgrFrustumCore<N>, r: &mut Reader<'_>, ortho: bool) {
    let (lo, hi, mode) = (r.xyz(), r.xyz(), r.integer());
    let (min, max) = (
        Vec3::new(lo.x(), lo.y(), lo.z()),
        Vec3::new(hi.x(), hi.y(), hi.z()),
    );
    let mut inside = mode == 1;
    attempt(|| {
        flag(core.has_box_overlap(
            &min,
            &max,
            ortho,
            if mode < 0 { None } else { Some(&mut inside) },
        )?);
        Ok(())
    });
    flag(inside);
}
fn sat_query<const N: usize>(core: &SelectMgrFrustumCore<N>, r: &mut Reader<'_>, ortho: bool) {
    let kind = r.integer();
    let mut normal = r.xyz();
    match kind {
        0 => flag(core.has_point_overlap(&r.pnt(), ortho)),
        1 => flag(core.has_segment_overlap(&r.pnt(), &r.pnt(), ortho)),
        2 => box_query(core, r, ortho),
        3 => {
            let vertices = points(r);
            attempt(|| {
                flag(core.has_polygon_overlap(&vertices, ortho, &mut normal)?);
                Ok(())
            });
            xyz(normal);
        }
        4 => {
            flag(core.has_triangle_overlap(&r.pnt(), &r.pnt(), &r.pnt(), ortho, &mut normal));
            xyz(normal);
        }
        _ => panic!("frustum SAT kind {kind}: expected 0..=4"),
    }
}
fn quadric_query<const N: usize>(
    core: &SelectMgrFrustumCore<N>,
    volume: &FrustumDefaultsIntersector,
    r: &mut Reader<'_>,
    ortho: bool,
) -> Result<(), Failure> {
    let (kind, mode) = (r.integer(), r.integer());
    let mut inside = mode == 1;
    let output = if mode < 0 { None } else { Some(&mut inside) };
    match kind {
        0 => {
            let (point, radius) = (r.pnt(), r.real());
            attempt(|| {
                flag(core.has_sphere_overlap(volume, point, radius, ortho, output)?);
                Ok(())
            });
        }
        1 => {
            let (a, b, h, hollow) = (r.real(), r.real(), r.real(), r.flag());
            let t = transformation(r)?;
            attempt(|| {
                flag(core.has_cylinder_overlap(a, b, h, t, hollow, ortho, output)?);
                Ok(())
            });
        }
        2 => {
            let (radius, filled) = (r.real(), r.flag());
            let t = transformation(r)?;
            attempt(|| {
                flag(core.has_circle_overlap(radius, t, filled, ortho, output)?);
                Ok(())
            });
        }
        3 => {
            let point = r.pnt();
            let vertices = points(r);
            attempt(|| {
                flag(core.is_dot_inside(point, &vertices)?);
                Ok(())
            });
        }
        4 => {
            let (a, b, h) = (r.real(), r.real(), r.real());
            let t = transformation(r)?;
            let vertices = points(r);
            attempt(|| {
                flag(core.is_inside_cylinder_end_face(a, b, h, t, &vertices)?);
                Ok(())
            });
        }
        _ => panic!("frustum quadric kind {kind}: expected 0..=4"),
    }
    flag(inside);
    Ok(())
}
pub(super) fn pointer_identities(text: String) -> String {
    named_pointer_identities(text, &["Builder", "Camera"])
}
pub(super) fn named_pointer_identities(mut text: String, fields: &[&str]) -> String {
    for field in fields {
        let prefix = format!("\"{field}\": \"");
        let start = text.find(&prefix).expect("dump pointer field") + prefix.len();
        let end = start + text[start..].find('"').expect("dump pointer closing quote");
        text.replace_range(start..end, if end == start { "null" } else { "present" });
    }
    text
}
fn dump_query<const N: usize>(
    core: &SelectMgrFrustumCore<N>,
    volume: &FrustumDefaultsIntersector,
    r: &mut Reader<'_>,
    ortho: bool,
) {
    let depth = r.integer();
    let builder = volume.base.builder.read().clone();
    flag(!builder.is_null());
    flag(volume.camera().object_address() == builder.camera().object_address());
    captured(|output| {
        let mut stream = DumpStream::default();
        let result = core.dump_json(&volume.base, ortho, &mut stream, depth);
        output.push_str(&pointer_identities(stream.text().to_owned()));
        result.map_err(|error| Failure::new(&error.to_string()))
    });
}
fn dispatch<const N: usize>(r: &mut Reader<'_>, ortho: bool, family: i32) -> Result<(), Failure> {
    let core = frustum_core::<N>(r);
    let volume = FrustumDefaultsIntersector::default();
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(if ortho {
        Graphic3dCameraProjection::Orthographic
    } else {
        Graphic3dCameraProjection::Perspective
    });
    volume.set_camera(&camera);
    match family {
        0 => sat_query(&core, r, ortho),
        1 => quadric_query(&core, &volume, r, ortho)?,
        2 => dump_query(&core, &volume, r, ortho),
        _ => panic!("frustum family {family}: expected 0..=2"),
    }
    Ok(())
}
/// Compare a native frustum specialization; `frustum_core_case(&values)`.
pub fn frustum_core_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (n, ortho, family) = (r.integer(), r.flag(), r.integer());
    attempt(|| {
        if n == 3 {
            dispatch::<3>(&mut r, ortho, family)
        } else {
            dispatch::<4>(&mut r, ortho, family)
        }
    });
    end();
}
