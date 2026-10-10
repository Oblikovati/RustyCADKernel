//! Twin of the native concrete triangular volume fixture.
use super::common::{Reader, attempt, captured, end, flag, int, real};
use super::frustum_core::pointer_identities;
use rustycad_tkernel::{
    ncollection::{DynamicArray, Vec4},
    standard::{Failure, Handle, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{GeneralTransform3, Point3},
};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};
use rustycad_tkv3d::select_mgr::*;

pub(super) fn triangle_camera(
    perspective: bool,
    zero: bool,
    scale: f64,
) -> Result<Handle<Graphic3dCamera>, Failure> {
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(if perspective {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
    camera.set_zero_to_one_depth(zero);
    camera.set_eye_and_center(&Point3::new(0., 0., 10.), &Point3::default())?;
    camera.set_z_range(0.1, 100.)?;
    camera.set_scale(scale);
    Ok(camera)
}
pub(super) fn triangle_transform(r: &mut Reader<'_>) -> Result<GeneralTransform3, Failure> {
    let mut transform = GeneralTransform3::default();
    for row in 1..=3 {
        for column in 1..=4 {
            transform.set_value(row, column, r.real())?;
        }
    }
    Ok(transform)
}
fn triangular_query(
    volume: &SelectMgrTriangularFrustum,
    r: &mut Reader<'_>,
) -> Result<(), Failure> {
    let mut clip = SelectMgrViewClipRange::default();
    *clip.change_unclip_range() = Range1::new(200., 201.)?;
    super::intersector_queries::intersector_query(volume, r, &clip)
}
fn triangular_state(
    volume: &SelectMgrTriangularFrustum,
    r: &mut Reader<'_>,
    cleared: bool,
) -> Result<(), Failure> {
    let (rebuild, depth) = (r.flag(), r.integer());
    if rebuild {
        volume.build()?;
    }
    flag(volume.is_scalable());
    int(volume.get_selection_type() as i32);
    if !cleared {
        let (width, height) = volume.window_size().unwrap();
        int(width);
        int(height);
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
struct TriangleConfiguration {
    volume: Handle<SelectMgrTriangularFrustum>,
    builder: Handle<SelectMgrFrustumBuilder>,
    original_camera: Handle<Graphic3dCamera>,
}
fn triangle_configuration(r: &mut Reader<'_>) -> Result<TriangleConfiguration, Failure> {
    let (perspective, zero, viewport) = (r.flag(), r.flag(), r.flag());
    let original_camera = triangle_camera(perspective, zero, 20.)?;
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    builder.set_camera(&triangle_camera(!perspective, zero, 7.)?);
    builder.set_window_size(320, 240);
    let volume = Handle::new(SelectMgrTriangularFrustum::new());
    volume.set_camera(&original_camera);
    volume.set_window_size(800, 600);
    if viewport {
        volume.set_viewport(0.1, 0.2, 0.9, 0.8);
    }
    volume.init(r.pnt2d(), r.pnt2d(), r.pnt2d());
    volume.build()?;
    volume.set_pixel_tolerance(9);
    Ok(TriangleConfiguration {
        volume,
        builder,
        original_camera,
    })
}
fn triangle_variant(
    fixture: &mut TriangleConfiguration,
    r: &mut Reader<'_>,
) -> Result<i32, Failure> {
    let (variant, scale) = (r.integer(), r.integer());
    let transform = triangle_transform(r)?;
    if variant == 1 {
        fixture.volume = fixture
            .volume
            .scale_and_transform(scale, &transform, &fixture.builder)?
            .cast();
    }
    if variant == 2 || variant == 3 {
        fixture.volume = fixture.volume.copy_with_builder(&fixture.builder)?.cast();
    }
    if variant == 3 {
        fixture.volume.build()?;
    }
    if variant == 4 {
        fixture.volume.clear();
    }
    Ok(variant)
}
fn triangle_volume(
    r: &mut Reader<'_>,
) -> Result<(Handle<SelectMgrTriangularFrustum>, i32), Failure> {
    let mut fixture = triangle_configuration(r)?;
    let variant = triangle_variant(&mut fixture, r)?;
    let volume = fixture.volume;
    flag(volume.camera().object_address() == fixture.original_camera.object_address());
    flag(volume.camera().object_address() == fixture.builder.camera().object_address());
    flag(volume.is_kind_name("SelectMgr_BaseFrustum"));
    flag(volume.is_kind_name("SelectMgr_BaseIntersector"));
    flag(volume.is_instance_name("SelectMgr_TriangularFrustum"));
    Ok((volume, variant))
}

/// Compare state, rebuilt copies and all concrete overlap overloads; `triangular_case(&values)`.
pub fn triangular_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (volume, variant) = triangle_volume(&mut r)?;
        if r.integer() == 0 {
            triangular_state(&volume, &mut r, variant == 4)
        } else {
            triangular_query(&volume, &mut r)
        }
    });
    end();
}
