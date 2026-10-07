//! Graphic3d_TransformPers and Graphic3d_TransformUtils, the twin of
//! tools/oracle/tkservice_probe_tpers.cxx.
use super::camera::{mat4, read_camera};
use super::common::{Reader, end, flag, int, pnt, real};
use super::print::show;
use rustycad_tkernel::ncollection::{ElementCast, Mat4, Vec2, Vec3};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};
use rustycad_tkservice::aspect::AspectTypeOfTriedronPosition;
use rustycad_tkservice::graphic3d::transform_utils as tu;
use rustycad_tkservice::graphic3d::{
    Graphic3dBndBox3d, Graphic3dCamera, Graphic3dTransModeFlags, Graphic3dTransformPers,
    Graphic3dTransformPersApply, Graphic3dTransformPersMembers, Graphic3dTransformPersScaledAbove,
    Graphic3dTransformPersSource, Graphic3dTransformScalar,
};

/// The persistence of `Kind Mode X Y Z Corner OffsetX OffsetY Scale`.
fn make(
    r: &mut Reader<'_>,
) -> (
    Result<Handle<dyn Graphic3dTransformPersSource>, Failure>,
    Point3,
) {
    let (kind, mode) = (r.integer(), Graphic3dTransModeFlags(r.integer()));
    let point = r.pnt();
    let (corner, ox, oy) = (r.integer(), r.integer(), r.integer());
    let min_scale = r.real();
    let pers: Result<Handle<dyn Graphic3dTransformPersSource>, Failure> = match kind {
        0 => Graphic3dTransformPers::new(mode).map(|p| Handle::new(p).cast()),
        1 => Graphic3dTransformPers::with_point(mode, &point).map(|p| Handle::new(p).cast()),
        2 => Graphic3dTransformPers::with_corner(
            mode,
            AspectTypeOfTriedronPosition(corner),
            Vec2::new(ox, oy),
        )
        .map(|p| Handle::new(p).cast()),
        _ => {
            Graphic3dTransformPersScaledAbove::new(min_scale, &point).map(|p| Handle::new(p).cast())
        }
    };
    (pers, point)
}

fn accessors(tp: &dyn Graphic3dTransformPersSource, camera: &Graphic3dCamera, [w, h]: [i32; 2]) {
    int(tp.mode().0);
    show(tp.anchor_point(), pnt);
    show(tp.corner_2d(), |corner| int(corner.0));
    show(tp.offset_2d(), |o| {
        int(o.x);
        int(o.y);
    });
    if tp.is_zoom_or_rotate() || tp.is_axial() {
        real(tp.persistent_scale(camera, w, h));
    }
}

fn box_corners(bounds: &Graphic3dBndBox3d) {
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    [min.x, min.y, min.z, max.x, max.y, max.z]
        .into_iter()
        .for_each(real);
}

fn applies(
    tp: &dyn Graphic3dTransformPersSource,
    cam: &Handle<Graphic3dCamera>,
    size: [i32; 2],
    anchor: &Point3,
    [b0, b1]: [Point3; 2],
) {
    show(apply_compute(tp, cam, size), |_| ());
    show(
        {
            let mut wv = cam.orientation_matrix();
            tp.apply_world_view(cam, &cam.projection_matrix(), &mut wv, size, None, true)
                .map(|()| wv)
        },
        |wv| mat4(&wv),
    );
    show(
        {
            let mut wv = cam.orientation_matrix_f();
            tp.apply_world_view(
                cam,
                &cam.projection_matrix_f(),
                &mut wv,
                size,
                Some(anchor),
                true,
            )
            .map(|()| wv)
        },
        |wv| mat4(&wv),
    );
    show(
        tp.compute(
            cam,
            &cam.projection_matrix(),
            &cam.orientation_matrix(),
            size,
            true,
        ),
        |m| mat4(&m),
    );
    let mut bounds = Graphic3dBndBox3d::from_corners(
        Vec3::new(b0.x(), b0.y(), b0.z()),
        Vec3::new(b1.x(), b1.y(), b1.z()),
    );
    show(
        tp.apply_bvh(
            cam,
            &cam.projection_matrix(),
            &cam.orientation_matrix(),
            size,
            &mut bounds,
        ),
        |()| box_corners(&bounds),
    );
    let mut bnd = BoundingBox3::default();
    bnd.update_bounds([b0.x(), b0.y(), b0.z()], [b1.x(), b1.y(), b1.z()]);
    let applied = tp.apply_bnd(
        cam,
        &cam.projection_matrix(),
        &cam.orientation_matrix(),
        size,
        &mut bnd,
    );
    show(applied.and_then(|()| bnd.get()), |v| {
        v.into_iter().for_each(real)
    });
}

/// `ComputeApply` on a camera copy, printing the matrix and whether the copy became orthographic.
fn apply_compute(
    tp: &dyn Graphic3dTransformPersSource,
    cam: &Handle<Graphic3dCamera>,
    [w, h]: [i32; 2],
) -> Result<(), Failure> {
    let mut copy = Handle::new(cam.copy_members());
    let matrix = tp.compute_apply(&mut copy, w, h, None)?;
    mat4(&matrix);
    flag(copy.is_orthographic());
    Ok(())
}

/// `sv_tpers`: a camera and a persistence, its accessors and the camera-dependent members.
pub fn tpers_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let camera = Handle::new(read_camera(&mut r));
    let (pers, anchor) = make(&mut r);
    let size = [r.integer(), r.integer()];
    let corners = [r.pnt(), r.pnt()];
    let pers = match pers {
        Ok(pers) => pers,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    accessors(&*pers, &camera, size);
    applies(&*pers, &camera, size, &anchor, corners);
    end();
}

fn read_scalars<T: Graphic3dTransformScalar, const N: usize>(r: &mut Reader<'_>) -> [T; N] {
    std::array::from_fn(|_| T::from_f64(r.real()))
}

/// The operations of `sv_tutils` on one scalar type.
fn tutils<T: Graphic3dTransformScalar + Into<f64>>(r: &mut Reader<'_>)
where
    Mat4<T>: std::ops::Mul<Output = Mat4<T>>
        + std::ops::Mul<
            rustycad_tkernel::ncollection::Vec4<T>,
            Output = rustycad_tkernel::ncollection::Vec4<T>,
        >,
{
    let [angle, x, y, z] = read_scalars::<T, 4>(r);
    let (s, t, o, pt) = (
        read_scalars::<T, 3>(r),
        read_scalars::<T, 3>(r),
        read_scalars::<T, 6>(r),
        read_scalars::<T, 3>(r),
    );
    let vp: [i32; 4] = std::array::from_fn(|_| r.integer());
    let mut m = Mat4::<T>::identity();
    tu::rotate(&mut m, angle, [x, y, z]);
    mat4(&m);
    tu::scale(&mut m, s);
    mat4(&m);
    tu::translate(&mut m, t);
    mat4(&m);
    let mut c = Mat4::<T>::identity();
    tu::construct_rotate(&mut c, angle, [x, y, z]);
    mat4(&c);
    let (mut proj, mut proj2) = (Mat4::<T>::identity(), Mat4::<T>::identity());
    tu::ortho(&mut proj, o);
    mat4(&proj);
    tu::ortho_2d(&mut proj2, [o[0], o[1], o[2], o[3]]);
    mat4(&proj2);
    let win = tu::project(pt, &m, &proj, vp);
    flag(win.is_some());
    win.iter().flatten().for_each(|v| real((*v).into()));
    let zero = T::from_f64(0.0);
    let obj = tu::un_project(win.unwrap_or([zero; 3]), &m, &proj, vp);
    flag(obj.is_some());
    obj.iter().flatten().for_each(|v| real((*v).into()));
    real(tu::scale_factor(&m));
}

fn convert<T: Graphic3dTransformScalar + Into<f64>>(transform: &Transform3)
where
    f64: ElementCast<T>,
{
    let mut matrix = Mat4::<T>::identity();
    tu::convert(transform, &mut matrix);
    mat4(&matrix);
}

/// `sv_tutils`: the transform utilities in double or float, then `Convert` of a fixed trsf.
pub fn tutils_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        tutils::<f64>(&mut r);
    } else {
        tutils::<f32>(&mut r);
    }
    let mut transform = Transform3::default();
    let axis = Direction3::new(0.0, 0.6, 0.8).map(|d| Axis1::new(Point3::new(1.0, 2.0, 3.0), d));
    let set = axis
        .and_then(|axis| transform.set_rotation(axis, 0.7))
        .and_then(|()| transform.set_scale_factor(2.5));
    show(set, |()| ());
    convert::<f64>(&transform);
    convert::<f32>(&transform);
    end();
}
