//! The orientation members of `V3d_View`: rotations about the center, a gravity point or
//! the eye, the twist, the eye, depth, projection, center and up, and the default camera.
use super::view::bad_value_if;
use super::{V3d, V3dTypeOfAxe, V3dTypeOfOrientation, V3dView, v3d_bad_value};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis1, Direction3, Point3, Transform3, Vector3, Xyz, dx, dy, dz, origin, resolution,
};
use std::f64::consts::PI;

/// `THE_TWO_PI`.
const TWO_PI: f64 = 2.0 * PI;

/// Brings an angle into `[-2 PI, 2 PI]` by whole turns, with the native loops.
pub(crate) fn wrap_angle(angle: f64) -> f64 {
    let mut value = angle;
    if value > 0.0 {
        while value > TWO_PI {
            value -= TWO_PI;
        }
    } else if value < 0.0 {
        while value < -TWO_PI {
            value += TWO_PI;
        }
    }
    value
}

/// The screen axes of `screenAxis`: true with `(x, y, z)` when `up` and `vpn` are not
/// parallel. The native member writes its outputs as it goes; the partial values are returned
/// too. Example: `screen_axis(&vpn, &up)`.
pub(crate) fn screen_axis(vpn: &Direction3, vup: &Direction3) -> (bool, Vector3, Vector3, Vector3) {
    let mut x_axis = Vector3::from(vup.xyz().crossed(vpn.xyz()));
    let (mut y_axis, mut z_axis) = (Vector3::default(), Vector3::default());
    if x_axis.magnitude() <= resolution() {
        return (false, x_axis, y_axis, z_axis);
    }
    // The magnitude is above the resolution, so the normalization cannot fail.
    let _ = x_axis.normalize();
    y_axis = Vector3::from(vpn.xyz().crossed(*x_axis.xyz()));
    if y_axis.magnitude() <= resolution() {
        return (false, x_axis, y_axis, z_axis);
    }
    let _ = y_axis.normalize();
    z_axis = Vector3::from(vpn.xyz());
    let _ = z_axis.normalize();
    (true, x_axis, y_axis, z_axis)
}

/// The first successful `screenAxis` of `vpn` with the candidate up directions, as the native
/// `||` chains; the axes of the last attempt when none succeeds.
fn screen_axes_with(
    vpn: &Direction3,
    candidates: &[Direction3],
) -> (bool, Vector3, Vector3, Vector3) {
    let mut last = (
        false,
        Vector3::default(),
        Vector3::default(),
        Vector3::default(),
    );
    for up in candidates {
        last = screen_axis(vpn, up);
        if last.0 {
            break;
        }
    }
    last
}

/// The three rotation axes of `Rotate(Ax, Ay, Az)` about the center: X from up and back, Y,
/// then Z as their cross products.
fn center_axes(up: &Direction3, back: &Direction3) -> Result<[Direction3; 3], Failure> {
    let x_axis = up.crossed(*back)?;
    let y_axis = back.crossed(x_axis)?;
    let z_axis = x_axis.crossed(y_axis)?;
    Ok([x_axis, y_axis, z_axis])
}

/// The product of the three rotations `(-Ax about Y, Ay about X, Az about Z)` through a point.
fn three_rotations(
    center: Point3,
    axes: [Direction3; 3],
    ax: f64,
    ay: f64,
    az: f64,
) -> Result<Transform3, Failure> {
    let [x_axis, y_axis, z_axis] = axes;
    let mut rotations = [Transform3::default(); 3];
    rotations[0].set_rotation(Axis1::new(center, y_axis), -ax)?;
    rotations[1].set_rotation(Axis1::new(center, x_axis), ay)?;
    rotations[2].set_rotation(Axis1::new(center, z_axis), az)?;
    let mut trsf = Transform3::default();
    rotations
        .into_iter()
        .for_each(|rotation| trsf.multiply(rotation));
    Ok(trsf)
}

impl V3dView {
    /// Stores the camera of the start of an operation (`myCamStartOp*`).
    fn store_start_op(&self) {
        let camera = self.camera();
        let mut state = self.state.write();
        state.cam_start_op_up = camera.up();
        state.cam_start_op_dir = camera.direction();
        state.cam_start_op_eye = camera.eye();
        state.cam_start_op_center = camera.center();
    }
    /// Restores the camera of the start of an operation.
    fn restore_start_op(&self) -> Result<(), Failure> {
        let state = self.state.read().clone();
        let camera = self.camera();
        camera.set_up(&state.cam_start_op_up);
        camera.set_eye_and_center(&state.cam_start_op_eye, &state.cam_start_op_center)?;
        camera.set_direction_from_eye(&state.cam_start_op_dir);
        Ok(())
    }
    /// `SetFront`: looks along or against the privileged plane normal, alternately.
    /// Example: `view.set_front()?`.
    pub fn set_front(&self) -> Result<(), Failure> {
        let Some(plane) = self.viewer().get().map(|v| v.privileged_plane()) else {
            return Ok(());
        };
        let camera = self.camera();
        camera.set_center(&plane.location())?;
        let direction = plane.direction();
        let switch = self.state.read().switch_set_front;
        camera.set_direction(&if switch {
            direction
        } else {
            direction.reversed()
        });
        camera.set_up(&plane.y_direction());
        self.state.write().switch_set_front = !switch;
        self.immediate_update()
    }
    /// `Rotate(Ax, Ay, Az, Start)`: rotations about the screen axes through the center.
    /// Example: `view.rotate(0.1, 0.0, 0.0, true)?`.
    pub fn rotate(&self, ax: f64, ay: f64, az: f64, start: bool) -> Result<(), Failure> {
        let (ax, ay, az) = (wrap_angle(ax), wrap_angle(ay), wrap_angle(az));
        if start {
            self.store_start_op();
        }
        self.restore_start_op()?;
        let state = self.state.read().clone();
        let back = state.cam_start_op_dir.reversed();
        let axes = center_axes(&state.cam_start_op_up, &back)?;
        let trsf = three_rotations(state.cam_start_op_center, axes, ax, ay, az)?;
        self.camera().transform(&trsf);
        self.immediate_update()
    }
    /// `Rotate(Ax, Ay, Az, X, Y, Z, Start)`: rotations about the screen axes through a
    /// gravity point (stored at the start). Example: `view.rotate_about(0.1, 0.0, 0.0, 1.0, 2.0, 3.0, true)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn rotate_about(
        &self,
        ax: f64,
        ay: f64,
        az: f64,
        x: f64,
        y: f64,
        z: f64,
        start: bool,
    ) -> Result<(), Failure> {
        let (ax, ay, az) = (wrap_angle(ax), wrap_angle(ay), wrap_angle(az));
        if start {
            self.state
                .write()
                .gravity_reference_point
                .set_coord_f64(x, y, z);
            self.store_start_op();
        }
        self.restore_start_op()?;
        let (cx, cy, cz) = self.state.read().gravity_reference_point.coord_f64();
        let camera = self.camera();
        let z_axis = camera.direction().reversed();
        let y_axis = camera.up();
        let x_axis = y_axis.crossed(z_axis)?;
        let trsf = three_rotations(
            Point3::new(cx, cy, cz),
            [x_axis, y_axis, z_axis],
            ax,
            ay,
            az,
        )?;
        camera.transform(&trsf);
        self.immediate_update()
    }
    /// `Rotate(Axe, Angle, Start)`; `view.rotate_axe(V3dTypeOfAxe::Z, 0.2, true)?`.
    pub fn rotate_axe(&self, axe: V3dTypeOfAxe, angle: f64, start: bool) -> Result<(), Failure> {
        match axe {
            V3dTypeOfAxe::X => self.rotate(angle, 0.0, 0.0, start),
            V3dTypeOfAxe::Y => self.rotate(0.0, angle, 0.0, start),
            V3dTypeOfAxe::Z => self.rotate(0.0, 0.0, angle, start),
        }
    }
    /// `Rotate(Axe, Angle, X, Y, Z, Start)`: about a world axis through a point.
    /// Example: `view.rotate_axe_about(V3dTypeOfAxe::X, 0.2, 0.0, 0.0, 0.0, true)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn rotate_axe_about(
        &self,
        axe: V3dTypeOfAxe,
        angle: f64,
        x: f64,
        y: f64,
        z: f64,
        start: bool,
    ) -> Result<(), Failure> {
        let angle = wrap_angle(angle);
        if start {
            self.state
                .write()
                .gravity_reference_point
                .set_coord_f64(x, y, z);
            self.store_start_op();
            self.state.write().view_axis = match axe {
                V3dTypeOfAxe::X => dx(),
                V3dTypeOfAxe::Y => dy(),
                V3dTypeOfAxe::Z => dz(),
            };
        }
        self.restore_start_op()?;
        let (cx, cy, cz) = self.state.read().gravity_reference_point.coord_f64();
        let axis = Direction3::new(
            f64::from(u8::from(axe == V3dTypeOfAxe::X)),
            f64::from(u8::from(axe == V3dTypeOfAxe::Y)),
            f64::from(u8::from(axe == V3dTypeOfAxe::Z)),
        )?;
        let mut rotation = Transform3::default();
        rotation.set_rotation(Axis1::new(Point3::new(cx, cy, cz), axis), angle)?;
        self.camera().transform(&rotation);
        self.immediate_update()
    }
    /// `Rotate(Angle, Start)`: about the default view axis through the default view point.
    /// Example: `view.rotate_default_axis(0.3, true)?`.
    pub fn rotate_default_axis(&self, angle: f64, start: bool) -> Result<(), Failure> {
        let angle = wrap_angle(angle);
        if start {
            self.store_start_op();
        }
        self.restore_start_op()?;
        let state = self.state.read().clone();
        let mut rotation = Transform3::default();
        rotation.set_rotation(
            Axis1::new(state.default_view_point, state.default_view_axis),
            angle,
        )?;
        self.camera().transform(&rotation);
        self.immediate_update()
    }
    /// `Turn(Ax, Ay, Az, Start)`: rotations about the screen axes through the eye.
    /// Example: `view.turn(0.1, 0.0, 0.0, true)?`.
    pub fn turn(&self, ax: f64, ay: f64, az: f64, start: bool) -> Result<(), Failure> {
        let (ax, ay, az) = (wrap_angle(ax), wrap_angle(ay), wrap_angle(az));
        if start {
            self.store_start_op();
        }
        self.restore_start_op()?;
        let camera = self.camera();
        let center = camera.eye();
        let z_axis = camera.direction().reversed();
        let y_axis = camera.up();
        let x_axis = y_axis.crossed(z_axis)?;
        let trsf = three_rotations(center, [x_axis, y_axis, z_axis], ax, ay, az)?;
        camera.transform(&trsf);
        self.immediate_update()
    }
    /// `Turn(Axe, Angle, Start)`; `view.turn_axe(V3dTypeOfAxe::Y, 0.1, true)?`.
    pub fn turn_axe(&self, axe: V3dTypeOfAxe, angle: f64, start: bool) -> Result<(), Failure> {
        match axe {
            V3dTypeOfAxe::X => self.turn(angle, 0.0, 0.0, start),
            V3dTypeOfAxe::Y => self.turn(0.0, angle, 0.0, start),
            V3dTypeOfAxe::Z => self.turn(0.0, 0.0, angle, start),
        }
    }
    /// `Turn(Angle, Start)`: about the default view axis through the eye.
    /// Example: `view.turn_default_axis(0.1, true)?`.
    pub fn turn_default_axis(&self, angle: f64, start: bool) -> Result<(), Failure> {
        let angle = wrap_angle(angle);
        if start {
            self.store_start_op();
        }
        self.restore_start_op()?;
        let camera = self.camera();
        let axis = self.state.read().default_view_axis;
        let mut rotation = Transform3::default();
        rotation.set_rotation(Axis1::new(camera.eye(), axis), angle)?;
        camera.transform(&rotation);
        self.immediate_update()
    }
    /// `SetTwist`: rotates the up direction about the view axis from the screen Y axis;
    /// `V3d_BadValue` when eye, center and up are aligned. Example: `view.set_twist(0.5)?`.
    pub fn set_twist(&self, angle: f64) -> Result<(), Failure> {
        let angle = wrap_angle(angle);
        let camera = self.camera();
        let reference = camera.direction().reversed();
        let axes = screen_axes_with(&reference, &[dz(), dy(), dx()]);
        self.store_screen_axes(&axes);
        if !axes.0 {
            return Err(v3d_bad_value("V3d_ViewSetTwist, alignment of Eye,At,Up,"));
        }
        let mut trsf = Transform3::default();
        trsf.set_rotation(
            Axis1::new(camera.center(), camera.direction().reversed()),
            angle,
        )?;
        camera.set_up(&Direction3::from_xyz(*axes.2.xyz())?);
        camera.transform(&trsf);
        self.immediate_update()
    }
    fn store_screen_axes(&self, axes: &(bool, Vector3, Vector3, Vector3)) {
        let mut state = self.state.write();
        (
            state.x_screen_axis,
            state.y_screen_axis,
            state.z_screen_axis,
        ) = (axes.1, axes.2, axes.3);
    }
    /// Runs a camera change that keeps the twist, without intermediate updates.
    fn keeping_twist(&self, change: impl FnOnce() -> Result<(), Failure>) -> Result<(), Failure> {
        let twist_before = self.twist();
        let was_update_enabled = self.set_immediate_update(false);
        change()?;
        self.set_twist(twist_before)?;
        self.set_immediate_update(was_update_enabled);
        self.immediate_update()
    }
    /// `SetEye`: moves the eye, keeping the twist; `view.set_eye(&0.0, 0.0, 100.0)?`.
    pub fn set_eye(&self, x: f64, y: f64, z: f64) -> Result<(), Failure> {
        self.keeping_twist(|| self.camera().set_eye(&Point3::new(x, y, z)))
    }
    /// `SetDepth`: the distance from the eye (> 0), or moves the center (< 0); 0 raises
    /// `V3d_BadValue`. Example: `view.set_depth(500.0)?`.
    pub fn set_depth(&self, depth: f64) -> Result<(), Failure> {
        bad_value_if(
            depth == 0.0,
            &format!("V3d_View::SetDepth, bad depth {depth}; expected a non-zero value"),
        )?;
        let camera = self.camera();
        if depth > 0.0 {
            camera.set_distance(depth);
        } else {
            let direction = Vector3::from(camera.direction());
            let center = camera.eye().translated(direction * depth.abs());
            camera.set_center(&center)?;
        }
        self.immediate_update()
    }
    /// `SetProj(Vx, Vy, Vz)`: the projection vector (from center to eye), keeping the twist;
    /// a null vector raises `V3d_BadValue`. Example: `view.set_proj(1.0, 1.0, 1.0)?`.
    pub fn set_proj(&self, vx: f64, vy: f64, vz: f64) -> Result<(), Failure> {
        bad_value_if(
            (vx * vx + vy * vy + vz * vz).sqrt() <= 0.0,
            &format!("V3d_View::SetProj, null projection vector ({vx}, {vy}, {vz})"),
        )?;
        self.keeping_twist(|| {
            self.camera()
                .set_direction(&Direction3::new(vx, vy, vz)?.reversed());
            Ok(())
        })
    }
    /// `SetProj(Orientation, IsYup)`: a predefined orientation around the origin, keeping the
    /// projected origin in place. Example: `view.set_proj_orientation(V3dTypeOfOrientation::Zpos, false)?`.
    pub fn set_proj_orientation(
        &self,
        orientation: V3dTypeOfOrientation,
        is_yup: bool,
    ) -> Result<(), Failure> {
        let mut up = if is_yup {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };
        if is_yup {
            if matches!(
                orientation,
                V3dTypeOfOrientation::Ypos | V3dTypeOfOrientation::Yneg
            ) {
                up = Vec3::new(0.0, 0.0, -1.0);
            }
        } else if orientation == V3dTypeOfOrientation::Zpos {
            up = Vec3::new(0.0, 1.0, 0.0);
        } else if orientation == V3dTypeOfOrientation::Zneg {
            up = Vec3::new(0.0, -1.0, 0.0);
        }
        let back = V3d::get_proj_axis(orientation);
        let camera = self.camera();
        let origin_vcs = camera.convert_world_to_view(&origin());
        let new_dist = camera.eye().distance(&Point3::new(0.0, 0.0, 0.0));
        let eye = Point3::from(Xyz::new(0.0, 0.0, 0.0) + back.xyz() * new_dist);
        camera.set_eye_and_center(&eye, &Point3::from(Xyz::new(0.0, 0.0, 0.0)))?;
        camera.set_direction_from_eye(&back.reversed());
        camera.set_up(&Direction3::new(up.x, up.y, up.z)?);
        camera.orthogonalize_up()?;
        self.panning(origin_vcs.x(), origin_vcs.y(), 1.0, true)?;
        self.immediate_update()
    }
    /// `SetAt`: moves the center, keeping the twist; `view.set_at(0.0, 0.0, 0.0)?`.
    pub fn set_at(&self, x: f64, y: f64, z: f64) -> Result<(), Failure> {
        self.keeping_twist(|| self.camera().set_center(&Point3::new(x, y, z)))
    }
    /// The up of `SetUp`: the screen Y axis of the first candidate not parallel to the view.
    fn set_up_from(&self, up: Direction3, message: &str) -> Result<(), Failure> {
        let camera = self.camera();
        let reference = camera.direction().reversed();
        let axes = screen_axes_with(&reference, &[up, dz(), dy(), dx()]);
        self.store_screen_axes(&axes);
        if !axes.0 {
            return Err(v3d_bad_value(message));
        }
        camera.set_up(&Direction3::from_xyz(*axes.2.xyz())?);
        self.immediate_update()
    }
    /// `SetUp(Vx, Vy, Vz)`: the up direction, or Z, Y, X when parallel to the view; a null
    /// vector raises from `gp_Dir`. Example: `view.set_up(0.0, 0.0, 1.0)?`.
    pub fn set_up(&self, vx: f64, vy: f64, vz: f64) -> Result<(), Failure> {
        self.set_up_from(
            Direction3::new(vx, vy, vz)?,
            "V3d_View::Setup, alignment of Eye,At,Up",
        )
    }
    /// `SetUp(Orientation)`; `view.set_up_orientation(V3dTypeOfOrientation::Zpos)?`.
    pub fn set_up_orientation(&self, orientation: V3dTypeOfOrientation) -> Result<(), Failure> {
        self.set_up_from(
            V3d::get_proj_axis(orientation),
            "V3d_View::SetUp, alignment of Eye,At,Up",
        )
    }
    /// `SetViewOrientationDefault`: stores the orientation as default; `view.set_view_orientation_default()`.
    pub fn set_view_orientation_default(&self) {
        self.default_camera().copy_orientation_data(&self.camera());
    }
    /// `SetViewMappingDefault`: stores the mapping as default; `view.set_view_mapping_default()`.
    pub fn set_view_mapping_default(&self) {
        self.default_camera().copy_mapping_data(&self.camera());
    }
    /// `ResetViewOrientation`; `view.reset_view_orientation()?`.
    pub fn reset_view_orientation(&self) -> Result<(), Failure> {
        self.camera().copy_orientation_data(&self.default_camera());
        self.immediate_update()
    }
    /// `ResetViewMapping`; `view.reset_view_mapping()?`.
    pub fn reset_view_mapping(&self) -> Result<(), Failure> {
        self.camera().copy_mapping_data(&self.default_camera());
        self.immediate_update()
    }
    /// `Reset`: the default camera; `view.reset(true)?`.
    pub fn reset(&self, to_update: bool) -> Result<(), Failure> {
        self.camera().copy(&self.default_camera());
        self.state.write().switch_set_front = false;
        if self.state.read().immediate_update || to_update {
            self.update()?;
        }
        Ok(())
    }
    /// `SetAxis`: the default view point and axis of `Rotate(Angle)`, `Turn(Angle)`, `Move`
    /// and `Translate(Length)`; a null axis raises from `gp_Dir`. Example: `view.set_axis(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)?`.
    pub fn set_axis(
        &self,
        x: f64,
        y: f64,
        z: f64,
        vx: f64,
        vy: f64,
        vz: f64,
    ) -> Result<(), Failure> {
        let axis = Direction3::new(vx, vy, vz)?;
        let mut state = self.state.write();
        (state.default_view_point, state.default_view_axis) = (Point3::new(x, y, z), axis);
        Ok(())
    }
}
