//! Graphic3d_Camera, the twin of tools/oracle/tkservice_probe_camera.cxx.
use super::common::{Reader, dir, end, flag, pnt, real, xyz};
use super::tried;
use rustycad_tkernel::ncollection::{Mat4, Vec2};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Direction3;
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dCameraFocusType, Graphic3dCameraIodType, Graphic3dCameraProjection,
    Graphic3dCameraTile,
};

fn mat4<T: Copy + Into<f64>>(matrix: &Mat4<T>) {
    for row in 0..4 {
        for column in 0..4 {
            real(matrix.get_value(row, column).into());
        }
    }
}

fn projection(value: i32) -> Graphic3dCameraProjection {
    use Graphic3dCameraProjection as P;
    [
        P::Orthographic,
        P::Perspective,
        P::Stereo,
        P::MonoLeftEye,
        P::MonoRightEye,
    ][value as usize]
}

/// Reads a camera and applies the setters in the native probe's order.
pub fn read_camera(r: &mut Reader<'_>) -> Graphic3dCamera {
    let cam = Graphic3dCamera::default();
    let (proj, zero_to_one) = (projection(r.integer()), r.flag());
    let (eye, center, up, axial) = (r.pnt(), r.pnt(), r.xyz(), r.xyz());
    let [fov_y, aspect, z_near, z_far, scale] = [0; 5].map(|_| r.real());
    let (iod_type, iod) = (r.integer(), r.real());
    let (focus_type, focus) = (r.integer(), r.real());
    let tile: [i32; 7] = std::array::from_fn(|_| r.integer());
    cam.set_projection_type(proj);
    cam.set_zero_to_one_depth(zero_to_one);
    tried(cam.set_eye_and_center(&eye, &center));
    tried(Direction3::from_xyz(up).map(|up| cam.set_up(&up)));
    tried(cam.set_axial_scale(axial));
    cam.set_fov_y(fov_y);
    cam.set_aspect(aspect);
    tried(cam.set_z_range(z_near, z_far));
    cam.set_scale(scale);
    cam.set_iod(
        if iod_type == 0 {
            Graphic3dCameraIodType::Absolute
        } else {
            Graphic3dCameraIodType::Relative
        },
        iod,
    );
    cam.set_z_focus(
        if focus_type == 0 {
            Graphic3dCameraFocusType::Absolute
        } else {
            Graphic3dCameraFocusType::Relative
        },
        focus,
    );
    let tile = Graphic3dCameraTile {
        total_size: Vec2::new(tile[0], tile[1]),
        tile_size: Vec2::new(tile[2], tile[3]),
        offset: Vec2::new(tile[4], tile[5]),
        is_top_down: tile[6] != 0,
    };
    cam.set_tile(&tile);
    cam
}

fn print_matrices(cam: &Graphic3dCamera) {
    [
        cam.orientation_matrix(),
        cam.projection_matrix(),
        cam.projection_stereo_left(),
        cam.projection_stereo_right(),
    ]
    .iter()
    .for_each(mat4);
    [
        cam.orientation_matrix_f(),
        cam.projection_matrix_f(),
        cam.projection_stereo_left_f(),
        cam.projection_stereo_right_f(),
    ]
    .iter()
    .for_each(mat4);
    cam.stereo_projection(Mat4::identity(), Mat4::identity())
        .iter()
        .for_each(mat4);
}

fn print_geometry(cam: &Graphic3dCamera) {
    match cam.frustum() {
        Ok(planes) => planes.iter().for_each(|plane| {
            plane
                .coefficients()
                .unwrap_or_default()
                .into_iter()
                .for_each(real)
        }),
        Err(error) => print!("{} ", error.exception_type()),
    }
    for point in cam.frustum_points(&Mat4::identity()) {
        [point.x, point.y, point.z].into_iter().for_each(real);
    }
    xyz(cam.view_dimensions());
    [
        cam.scale(),
        cam.distance(),
        cam.fov_x(),
        cam.ndc_2d_offset_x(),
        cam.ndc_2d_offset_y(),
    ]
    .into_iter()
    .for_each(real);
    pnt(cam.eye());
    pnt(cam.center());
    dir(cam.direction());
    dir(cam.up());
    match cam.orthogonalized_up() {
        Ok(up) => dir(up),
        Err(error) => print!("{} ", error.exception_type()),
    }
}

/// `sv_camera Camera Point`.
pub fn camera_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let cam = read_camera(&mut r);
    let p = r.pnt();
    print_matrices(&cam);
    for converted in [
        cam.project(&p),
        cam.un_project(&p),
        cam.convert_world_to_view(&p),
        cam.convert_view_to_world(&p),
        cam.convert_view_to_proj(&p),
        cam.convert_proj_to_view(&p),
    ] {
        pnt(converted);
    }
    print_geometry(&cam);
    end();
}

fn read_box(r: &mut Reader<'_>) -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    bounds.add_point(r.pnt());
    bounds.add_point(r.pnt());
    bounds
}

/// `sv_camera_fit Camera Model(6) Graphic(6) Resolution ToEnlarge ScaleFactor`.
pub fn fit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let cam = read_camera(&mut r);
    let (model, graphic) = (read_box(&mut r), read_box(&mut r));
    let (resolution, enlarge, factor) = (r.real(), r.flag(), r.real());
    match cam.z_fit_all_range(factor, &model, &graphic) {
        Ok((fitted, near, far)) => {
            flag(fitted);
            real(near);
            real(far);
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
    match cam.fit_min_max(&graphic, resolution, enlarge) {
        Ok(fitted) => flag(fitted),
        Err(error) => print!("{} ", error.exception_type()),
    }
    pnt(cam.eye());
    pnt(cam.center());
    real(cam.scale());
    real(cam.distance());
    end();
}

/// `sv_camera_interp CameraA CameraB T`.
pub fn interp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (read_camera(&mut r), read_camera(&mut r));
    let t = r.real();
    let c = Graphic3dCamera::default();
    match Graphic3dCamera::interpolate(&a, &b, t, &c) {
        Ok(()) => {
            pnt(c.eye());
            pnt(c.center());
            dir(c.up());
            dir(c.direction());
            real(c.scale());
            real(c.distance());
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
    end();
}
