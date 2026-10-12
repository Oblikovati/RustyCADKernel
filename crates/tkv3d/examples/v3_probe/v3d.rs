//! V3d view camera operations, the twin of tools/oracle/tkv3d_probe_v3d.cxx.
use super::common::{Reader, end, int, real};
use super::tried;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectWindow};
use rustycad_tkservice::graphic3d::Graphic3dCameraProjection;
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::v3d::V3dViewSource;
use rustycad_tkv3d::v3d::{V3dTypeOfAxe, V3dTypeOfOrientation, V3dView, V3dViewer};

/// `v3_axe`: the axis of an integer code (0 X, 1 Y, otherwise Z).
fn axe(code: f64) -> V3dTypeOfAxe {
    match code as i32 {
        0 => V3dTypeOfAxe::X,
        1 => V3dTypeOfAxe::Y,
        _ => V3dTypeOfAxe::Z,
    }
}

/// `v3_orientation`: the orientation of an integer code, clamped to the 26 orientations.
fn orientation(code: f64) -> V3dTypeOfOrientation {
    V3dTypeOfOrientation::ALL[(code as i32).clamp(0, 25) as usize]
}

/// Prints three reals.
fn triple((x, y, z): (f64, f64, f64)) {
    [x, y, z].into_iter().for_each(real);
}

/// `v3_view_state`: the camera queries of the view after an operation.
fn view_state(v: &V3dView, w: i32, h: i32) -> Result<(), Failure> {
    [v.eye(), v.at(), v.up(), v.proj()]
        .into_iter()
        .for_each(triple);
    real(v.twist());
    real(v.scale());
    let (sw, sh) = v.size();
    [sw, sh, v.z_size(), v.depth()].into_iter().for_each(real);
    triple(v.axial_scale());
    real(v.focale());
    let (xv, yv) = v.convert_to_view(w / 3, h / 4)?;
    [xv, yv].into_iter().for_each(real);
    let (xp, yp) = v.convert_view_to_pixels(0.3 * sw, -0.2 * sh)?;
    [xp, yp].into_iter().for_each(int);
    triple(v.convert_to_world(w / 5, h / 3)?);
    v.convert_with_proj(w / 2, h / 5)?
        .into_iter()
        .for_each(real);
    let (px, py) = v.project(1.0, 2.0, 3.0);
    [px, py].into_iter().for_each(real);
    triple(v.project3(1.0, 2.0, 3.0));
    let (xp, yp) = v.convert_world_to_pixels(1.0, 2.0, 3.0)?;
    [xp, yp].into_iter().for_each(int);
    real(v.convert_pixel_to_view(17)?);
    int(v.convert_view_to_pixel(1.5)?);
    Ok(())
}

/// The box of operation 18: centered on `(d, d, d)` with half sizes `a`, `b`, `c`.
fn centered_box(a: f64, b: f64, c: f64, d: f64) -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    bounds.update_bounds([d - a, d - b, d - c], [d + a, d + b, d + c]);
    bounds
}

/// Operations 0 to 20 of `v3_view_op`.
fn view_op_low(v: &V3dView, code: i32, [a, b, c, d]: [f64; 4]) -> Result<(), Failure> {
    let [ia, ib, ic, id] = [a as i32, b as i32, c as i32, d as i32];
    match code {
        0 => v.set_proj_orientation(orientation(a), b != 0.0),
        1 => v.set_size(a),
        2 => v.set_zoom(a, b != 0.0),
        3 => v.rotate(a, b, c, d != 0.0),
        4 => v.turn(a, b, c, d != 0.0),
        5 => v.set_twist(a),
        6 => v.set_eye(a, b, c),
        7 => v.set_at(a, b, c),
        8 => v.set_depth(a),
        9 => v.set_proj(a, b, c),
        10 => v.panning(a, b, c, d != 0.0),
        11 => v.pan(ia, ib, c, d != 0.0),
        12 => {
            v.start_zoom_at_point(ia, ib);
            v.zoom_at_point(ia, ib, ic, id)
        }
        13 => v
            .start_rotation(ia, ib, 0.0)
            .and_then(|()| v.rotation(ic, id)),
        14 => v.move_eye(a, b, c, d != 0.0),
        15 => v.translate(a, b, c, d != 0.0),
        16 => v.place(ia, ib, c),
        17 => v.window_fit(ia, ib, ic, id),
        18 => v.fit_all_box(&centered_box(a, b, c, d), 0.01, false),
        19 => v.set_scale(a),
        _ => v.set_axial_scale(a, b, c),
    }
}

/// Operations 21 and above of `v3_view_op`.
fn view_op_high(v: &V3dView, code: i32, [a, b, c, d]: [f64; 4]) -> Result<(), Failure> {
    let [ia, ib, ic, id] = [a as i32, b as i32, c as i32, d as i32];
    match code {
        21 => v.rotate_axe_about(axe(b), a, c, d, 0.0, true),
        22 => v.set_z_size(a),
        23 => v.depth_fit_all(a, b),
        24 => v.fit_all_rect(a, b, c, d),
        25 => v.axial_scale_drag(ia, ib, axe(c)),
        26 => v.set_center(ia, ib),
        27 => v.set_up(a, b, c),
        28 => v.z_fit_all(a),
        29 => v.translate_axe(axe(b), a, c != 0.0),
        30 => v.move_axe(axe(b), a, c != 0.0),
        31 => v.zoom(ia, ib, ic, id),
        32 => {
            v.camera()
                .set_projection_type(Graphic3dCameraProjection::Perspective);
            Ok(())
        }
        33 => v.set_focale(a),
        34 => v.reset(false),
        35 => v.set_front(),
        36 => v.turn_axe(axe(b), a, c != 0.0),
        37 => v.rotate_axe(axe(b), a, c != 0.0),
        38 => v.rotate_about(a, b, c, d, 0.0, 0.0, true),
        39 => v.window_fit_all(ia, ib, ic, id),
        _ => v.set_up_orientation(orientation(a)),
    }
}

/// A viewer and its view of a `w` x `h` window on the null driver, with the default lights
/// on; the view only refers to its viewer, so the caller keeps both.
fn open_view(w: i32, h: i32) -> Result<(Handle<V3dViewer>, Handle<dyn V3dViewSource>), Failure> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle())?;
    viewer.set_default_lights()?;
    viewer.set_light_on()?;
    let view = viewer.create_view()?;
    let window = AspectNeutralWindow::default();
    window.set_size(w, h);
    let window: Handle<dyn AspectWindow> = Handle::new(window).cast();
    view.set_window(&window, 0)?;
    Ok((viewer, view))
}

/// `v3_view W H N (Code A B C D)*N`.
pub fn view_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (w, h, n) = (r.integer(), r.integer(), r.integer());
    let (_viewer, view) = open_view(w, h).expect("probe view on the null driver");
    for _ in 0..n {
        let code = r.integer();
        let args: [f64; 4] = std::array::from_fn(|_| r.real());
        let op = if code <= 20 {
            view_op_low(&view, code, args)
        } else {
            view_op_high(&view, code, args)
        };
        tried(op);
        tried(view_state(&view, w, h));
    }
    end();
}
