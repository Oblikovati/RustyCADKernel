//! `Interpolate`, `FrustumPoints` and `DumpJson` of `Graphic3d_Camera`.
use super::camera::Graphic3dCameraMatrices;
use super::camera_projection::Graphic3dCameraScalar;
use super::{GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB, Graphic3dCamera};
use crate::json_dump::{class_begin, field_dumped, field_integer, field_real};
use rustycad_tkernel::ncollection::{Mat4, Vec3, Vec4, VectorDumpComponent};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Axis3, Point3, Quaternion, QuaternionNLerp, Transform3, Xyz, xoy};
use std::io;

/// `cameraToAx3`: the frame whose Z is the backward direction and X the right side.
fn camera_to_ax3(camera: &Graphic3dCamera) -> Result<Axis3, Failure> {
    let back = -camera.direction();
    let x_axis = camera.up().crossed(back)?;
    let y_axis = back.crossed(x_axis)?;
    let z_axis = x_axis.crossed(y_axis)?;
    Axis3::new(Point3::new(0.0, 0.0, 0.0), z_axis, x_axis)
}

/// The rotation of a camera frame relative to `gp::XOY()`.
fn frame_rotation(camera: &Graphic3dCamera) -> Result<Quaternion, Failure> {
    let mut transform = Transform3::default();
    transform.set_transformation_between(camera_to_ax3(camera)?, Axis3::from_axis2(xoy()));
    Ok(transform.rotation_quaternion())
}

fn lerp_xyz(start: Xyz, end: Xyz, t: f64) -> Xyz {
    start * (1.0 - t) + end * t
}
fn lerp(start: f64, end: f64, t: f64) -> f64 {
    (1.0 - t) * start + t * end
}

impl Graphic3dCamera {
    /// `Interpolate(start, end, t, camera)`: copies the end (t = 1) or the start, then turns
    /// it by the interpolated rotation, moves eye and center about the anchor that keeps the
    /// motion smooth, and interpolates an orthographic scale. Example:
    /// `Graphic3dCamera::interpolate(&a, &b, 0.5, &out)?`.
    pub fn interpolate(
        start: &Graphic3dCamera,
        end: &Graphic3dCamera,
        t: f64,
        camera: &Graphic3dCamera,
    ) -> Result<(), Failure> {
        if (t - 1.0).abs() < precision::confusion() {
            camera.copy(end);
            return Ok(());
        }
        camera.copy(start);
        if (t - 0.0).abs() < precision::confusion() {
            return Ok(());
        }
        let (rot_start, rot_end) = (frame_rotation(start)?, frame_rotation(end)?);
        let rotation =
            QuaternionNLerp::interpolated(Quaternion::default(), rot_end * rot_start.inverted(), t);
        camera.transform(&Transform3::from_quaternion(rotation));
        Self::interpolate_eye_center(start, end, t, camera)?;
        if (start.scale() - end.scale()).abs() > precision::confusion() && start.is_orthographic() {
            camera.set_scale(lerp(start.scale(), end.scale(), t));
        }
        Ok(())
    }
    /// The eye and center step of `Interpolate`.
    fn interpolate_eye_center(
        start: &Graphic3dCamera,
        end: &Graphic3dCamera,
        t: f64,
        camera: &Graphic3dCamera,
    ) -> Result<(), Failure> {
        let (sc, ec, se, ee) = (start.center(), end.center(), start.eye(), end.eye());
        let center = lerp_xyz(sc.xyz(), ec.xyz(), t);
        let eye = lerp_xyz(se.xyz(), ee.xyz(), t);
        let (mut anchor, mut kc) = (center, 0.0);
        let (delta_center, delta_eye) = (sc.distance(&ec), se.distance(&ee));
        if delta_eye <= rustycad_tkmath::gp::resolution() {
            (anchor, kc) = (eye, 1.0);
        } else if delta_center > rustycad_tkmath::gp::resolution() {
            kc = delta_center / (delta_center + delta_eye);
            anchor = lerp_xyz(
                lerp_xyz(sc.xyz(), se.xyz(), kc),
                lerp_xyz(ec.xyz(), ee.xyz(), kc),
                t,
            );
        }
        let direction = camera.direction();
        let distance = lerp(se.distance(&sc), ee.distance(&ec), t);
        let new_center = anchor + direction.xyz() * distance * kc;
        let new_eye = anchor - direction.xyz() * distance * (1.0 - kc);
        camera.set_eye_and_center(&Point3::from(new_eye), &Point3::from(new_center))
    }
}

/// The near (`n`) and far (`f`) left, right, top, bottom extents of `FrustumPoints`.
fn frustum_extents(
    projection: &Mat4<f64>,
    near: f64,
    far: f64,
    orthographic: bool,
) -> [[f64; 4]; 2] {
    let p = |r: usize, c: usize| projection.get_value(r, c);
    if !orthographic {
        let at = |z: f64| {
            [
                z * (p(0, 2) - 1.0) / p(0, 0),
                z * (p(0, 2) + 1.0) / p(0, 0),
                z * (p(1, 2) + 1.0) / p(1, 1),
                z * (p(1, 2) - 1.0) / p(1, 1),
            ]
        };
        return [at(near), at(far)];
    }
    let flat = [
        (1.0 + p(0, 3)) / (-p(0, 0)),
        (1.0 - p(0, 3)) / p(0, 0),
        (1.0 - p(1, 3)) / p(1, 1),
        (-1.0 - p(1, 3)) / p(1, 1),
    ];
    [flat, flat]
}

impl Graphic3dCamera {
    /// `FrustumPoints`: the eight frustum corners in the model space of `model_world`, in the
    /// `FrustumVert_*` order (left/right, bottom/top, near/far). Example:
    /// `camera.frustum_points(&Mat4::identity())`.
    pub fn frustum_points(
        &self,
        model_world: &Mat4<f64>,
    ) -> [Vec3<f64>; GRAPHIC3D_CAMERA_FRUSTUM_VERTICES_NB] {
        let projection = self.projection_matrix();
        let world_view = self.orientation_matrix() * *model_world;
        let (near, far) = (self.z_near(), self.z_far());
        let [
            [n_left, n_right, n_top, n_bottom],
            [f_left, f_right, f_top, f_bottom],
        ] = frustum_extents(&projection, near, far, self.is_orthographic());
        let mut inverse = Mat4::identity();
        // A singular world-view leaves the inverse as computed so far, as natively.
        let _ = world_view.inverted_to(&mut inverse);
        let point = |x: f64, y: f64, z: f64| {
            let p = inverse * Vec4::new(x, y, z, 1.0);
            // `xyz() / w()`: the native double Vec3 division multiplies by `1 / w`.
            Vec3::new(p.x, p.y, p.z) / p.w
        };
        [
            point(n_left, n_bottom, -near),
            point(f_left, f_bottom, -far),
            point(n_left, n_top, -near),
            point(f_left, f_top, -far),
            point(n_right, n_bottom, -near),
            point(f_right, f_bottom, -far),
            point(n_right, n_top, -near),
            point(f_right, f_top, -far),
        ]
    }
}

/// `TransformMatrices<Elem_t>::DumpJson`.
fn dump_matrices<T: Graphic3dCameraScalar + VectorDumpComponent>(
    matrices: &Graphic3dCameraMatrices<T>,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    if matrices.is_orientation_valid {
        field_dumped(stream, depth, "&Orientation", &|s, _| {
            matrices.orientation.dump_json(s)
        })?;
    }
    if matrices.is_projection_valid {
        for (name, matrix) in [
            ("&MProjection", &matrices.m_projection),
            ("&LProjection", &matrices.l_projection),
            ("&RProjection", &matrices.r_projection),
        ] {
            field_dumped(stream, depth, name, &|s, _| matrix.dump_json(s))?;
        }
    }
    field_integer(
        stream,
        "myIsOrientationValid",
        i32::from(matrices.is_orientation_valid),
    )?;
    field_integer(
        stream,
        "myIsProjectionValid",
        i32::from(matrices.is_projection_valid),
    )
}

impl Graphic3dCamera {
    /// `DumpJson`; `camera.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let s = self.read(Clone::clone);
        class_begin(stream, "Graphic3d_Camera")?;
        field_dumped(stream, depth, "&myUp", &|o, _| s.up.dump_json(o))?;
        field_dumped(stream, depth, "&myDirection", &|o, _| {
            s.direction.dump_json(o)
        })?;
        field_dumped(stream, depth, "&myEye", &|o, _| s.eye.dump_json(o))?;
        field_real(stream, "myDistance", s.distance)?;
        field_dumped(stream, depth, "&myAxialScale", &|o, _| {
            s.axial_scale.dump_json(o)
        })?;
        field_integer(stream, "myProjType", s.proj_type as i32)?;
        for (name, value) in [
            ("myFOVy", s.fov_y),
            ("myZNear", s.z_near),
            ("myZFar", s.z_far),
            ("myAspect", s.aspect),
            ("myScale", s.scale),
            ("myZFocus", s.z_focus),
        ] {
            field_real(stream, name, value)?;
        }
        field_integer(stream, "myZFocusType", s.z_focus_type as i32)?;
        field_real(stream, "myIOD", s.iod)?;
        field_integer(stream, "myIODType", s.iod_type as i32)?;
        field_dumped(stream, depth, "&myTile", &|o, d| s.tile.dump_json(o, d))?;
        field_dumped(stream, depth, "&myMatricesD", &|o, d| {
            dump_matrices(&s.matrices_d, o, d)
        })?;
        field_dumped(stream, depth, "&myMatricesF", &|o, d| {
            dump_matrices(&s.matrices_f, o, d)
        })?;
        field_dumped(stream, depth, "&myWorldViewProjState", &|o, d| {
            s.world_view_proj_state.dump_json(o, d)
        })
    }
}
