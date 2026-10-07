//! The XR members of `Graphic3d_CView`: the session, the base and posed cameras, and the
//! conversions between the head pose and the world.
use super::{Graphic3dCView, Graphic3dCamera};
use crate::aspect::{AspectXRSession, AspectXRSessionMembers as _};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{self, Axis1, Axis3, Direction3, Point3, Transform3, Vector3, Xyz};

/// `gp_Vec(gp_XYZ)`.
fn vector(xyz: Xyz) -> Vector3 {
    Vector3::new(xyz.x(), xyz.y(), xyz.z())
}

/// The transformation from the VR axes (Z forward, X right) to the camera axes at `origin`.
fn camera_cs_transform(camera: &Graphic3dCamera, origin: Point3) -> Result<Transform3, Failure> {
    let vr_axes = Axis3::new(gp::origin(), gp::dz(), gp::dx())?;
    let camera_axes = Axis3::new(
        origin,
        camera.direction().reversed(),
        camera.side_right()?.reversed(),
    )?;
    let mut trsf = Transform3::default();
    trsf.set_transformation_between(camera_axes, vr_axes);
    Ok(trsf)
}

/// The required base XR camera, as a failure when it is null (natively a null dereference).
fn required(
    camera: &Handle<Graphic3dCamera>,
    member: &str,
) -> Result<Handle<Graphic3dCamera>, Failure> {
    if camera.is_null() {
        return Err(rustycad_tkernel::standard::exceptions::program_error(
            &format!("Graphic3d_CView::{member}: null base XR camera; expected a camera"),
        ));
    }
    Ok(camera.clone())
}

/// The XR members of every view.
pub trait Graphic3dCViewXr: Graphic3dCView {
    /// `XRSession`; `view.xr_session()`.
    fn xr_session(&self) -> Handle<dyn AspectXRSession> {
        self.cview_base().fields.read().xr_session.clone()
    }
    /// `SetXRSession`; `view.set_xr_session(&session)`.
    fn set_xr_session(&self, session: &Handle<dyn AspectXRSession>) {
        self.cview_base().fields.write().xr_session = session.clone();
    }
    /// `IsActiveXR`: an open session; `view.is_active_xr()`.
    fn is_active_xr(&self) -> bool {
        self.xr_session().get().is_some_and(|s| s.is_open())
    }
    /// `PosedXRCamera`; `view.posed_xr_camera()`.
    fn posed_xr_camera(&self) -> Handle<Graphic3dCamera> {
        self.cview_base().fields.read().posed_xr_camera.clone()
    }
    /// `SetPosedXRCamera`; `view.set_posed_xr_camera(&camera)`.
    fn set_posed_xr_camera(&self, camera: &Handle<Graphic3dCamera>) {
        self.cview_base().fields.write().posed_xr_camera = camera.clone();
    }
    /// `BaseXRCamera`; `view.base_xr_camera()`.
    fn base_xr_camera(&self) -> Handle<Graphic3dCamera> {
        self.cview_base().fields.read().base_xr_camera.clone()
    }
    /// `SetBaseXRCamera`; `view.set_base_xr_camera(&camera)`.
    fn set_base_xr_camera(&self, camera: &Handle<Graphic3dCamera>) {
        self.cview_base().fields.write().base_xr_camera = camera.clone();
    }
    /// `PoseXRToWorld`: a head pose in the world, relative to the base camera eye;
    /// `view.pose_xr_to_world(&pose)?`.
    fn pose_xr_to_world(&self, pose: &Transform3) -> Result<Transform3, Failure> {
        let base = required(&self.base_xr_camera(), "PoseXRToWorld")?;
        let Some(base) = base.get() else {
            return Ok(*pose);
        };
        Ok(camera_cs_transform(base, base.eye())? * *pose)
    }
    /// `ViewAxisInWorld`: the -Z axis of a head pose in the world; `view.view_axis_in_world(&pose)?`.
    fn view_axis_in_world(&self, pose: &Transform3) -> Result<Axis1, Failure> {
        Ok(Axis1::new(gp::origin(), gp::dz().reversed()).transformed(self.pose_xr_to_world(pose)?))
    }
    /// `SetupXRPosedCamera`: the posed camera becomes the view camera (keeping a copy);
    /// `view.setup_xr_posed_camera()`.
    fn setup_xr_posed_camera(&self) {
        let mut fields = self.cview_base().fields.write();
        let Some(posed) = fields
            .posed_xr_camera
            .get()
            .map(|_| fields.posed_xr_camera.clone())
        else {
            return;
        };
        fields.camera = posed.clone();
        if fields.posed_xr_camera_copy.is_null() {
            fields.posed_xr_camera_copy = Handle::new(Graphic3dCamera::default());
        }
        if let (Some(copy), Some(posed)) = (fields.posed_xr_camera_copy.get(), posed.get()) {
            copy.copy(posed);
        }
    }
    /// `UnsetXRPosedCamera`: back to the base camera, after carrying the posed camera
    /// changes to it; `view.unset_xr_posed_camera()?`.
    fn unset_xr_posed_camera(&self) -> Result<(), Failure> {
        let (camera, posed, base) = {
            let fields = self.cview_base().fields.read();
            (
                fields.camera.clone(),
                fields.posed_xr_camera.clone(),
                fields.base_xr_camera.clone(),
            )
        };
        if camera != posed || base.is_null() {
            return Ok(());
        }
        self.synchronize_xr_posed_to_base_camera()?;
        self.cview_base().fields.write().camera = base;
        Ok(())
    }
    /// `SynchronizeXRBaseToPosedCamera`: the posed camera from the base camera and the head
    /// pose; `view.synchronize_xr_base_to_posed_camera()?`.
    fn synchronize_xr_base_to_posed_camera(&self) -> Result<(), Failure> {
        let posed = self.posed_xr_camera();
        let session = self.xr_session();
        let (Some(posed), Some(session)) = (posed.get(), session.get()) else {
            return Ok(());
        };
        self.compute_xr_posed_camera_from_base(posed, &session.head_pose())
    }
    /// `ComputeXRPosedCameraFromBase`: the base camera moved by a head pose;
    /// `view.compute_xr_posed_camera_from_base(&camera, &pose)?`.
    fn compute_xr_posed_camera_from_base(
        &self,
        camera: &Graphic3dCamera,
        xr_trsf: &Transform3,
    ) -> Result<(), Failure> {
        let base = required(&self.base_xr_camera(), "ComputeXRPosedCameraFromBase")?;
        let Some(base) = base.get() else {
            return Ok(());
        };
        camera.copy(base);
        let trsf_cs = camera_cs_transform(base, gp::origin())?;
        let trsf_to_camera = trsf_cs * *xr_trsf * trsf_cs.inverted()?;
        let mut trsf_to_eye = Transform3::default();
        let eye = base.eye();
        trsf_to_eye.set_translation(Vector3::new(eye.x(), eye.y(), eye.z()));
        let trsf = trsf_to_eye * trsf_to_camera;
        camera.set_up(&base.up().transformed(trsf));
        camera.set_direction_from_eye(&base.direction().transformed(trsf));
        camera.move_eye_to(&gp::origin().translated(vector(trsf.translation_part())));
        Ok(())
    }
    /// `SynchronizeXRPosedToBaseCamera`: carries the changes of the posed camera (since
    /// `SetupXRPosedCamera`) to the base camera; `view.synchronize_xr_posed_to_base_camera()?`.
    fn synchronize_xr_posed_to_base_camera(&self) -> Result<(), Failure> {
        let (copy, posed, base, camera, session) = {
            let f = self.cview_base().fields.read();
            (
                f.posed_xr_camera_copy.clone(),
                f.posed_xr_camera.clone(),
                f.base_xr_camera.clone(),
                f.camera.clone(),
                f.xr_session.clone(),
            )
        };
        let (Some(copy_ref), Some(posed_ref), false) = (copy.get(), posed.get(), base.is_null())
        else {
            return Ok(());
        };
        if camera != posed || unchanged(copy_ref, posed_ref) {
            return Ok(());
        }
        let Some(session) = session.get() else {
            return Ok(());
        };
        self.compute_xr_base_camera_from_posed(posed_ref, &session.head_pose())?;
        copy_ref.copy(posed_ref);
        Ok(())
    }
    /// `ComputeXRBaseCameraFromPosed`: the base camera from a posed camera and its head pose;
    /// `view.compute_xr_base_camera_from_posed(&posed, &pose)?`.
    fn compute_xr_base_camera_from_posed(
        &self,
        posed: &Graphic3dCamera,
        pose_trsf: &Transform3,
    ) -> Result<(), Failure> {
        let base = required(&self.base_xr_camera(), "ComputeXRBaseCameraFromPosed")?;
        let Some(base) = base.get() else {
            return Ok(());
        };
        let trsf_cs = camera_cs_transform(base, gp::origin())?;
        let trsf_to_camera = trsf_cs * *pose_trsf * trsf_cs.inverted()?;
        let cam_to_head = trsf_to_camera.inverted()?;
        let up: Direction3 = posed.up().transformed(cam_to_head);
        let direction: Direction3 = posed.direction().transformed(cam_to_head);
        let eye = posed
            .eye()
            .translated(vector(trsf_to_camera.translation_part().reversed()));
        base.set_up(&up);
        base.set_direction_from_eye(&direction);
        base.move_eye_to(&eye);
        Ok(())
    }
    /// `TurnViewXRCamera`: turns the base camera around the head; `view.turn_view_xr_camera(&turn)?`.
    fn turn_view_xr_camera(&self, turn: &Transform3) -> Result<(), Failure> {
        let base = required(&self.base_xr_camera(), "TurnViewXRCamera")?;
        let session = self.xr_session();
        let (Some(base_ref), Some(session)) = (base.get(), session.get()) else {
            return Ok(());
        };
        let mut head_local = Transform3::default();
        head_local.set_translation_part(vector(session.head_pose().translation_part()));
        let anchor = self.pose_xr_to_world(&head_local)?.translation_part();
        base_ref.set_direction_from_eye(&base_ref.direction().transformed(*turn));
        let trsf_cs = camera_cs_transform(base_ref, gp::origin())?;
        let trsf_to_camera = trsf_cs * head_local * trsf_cs.inverted()?;
        let anchor = Point3::new(anchor.x(), anchor.y(), anchor.z());
        base_ref
            .move_eye_to(&anchor.translated(vector(trsf_to_camera.translation_part().reversed())));
        self.synchronize_xr_base_to_posed_camera()
    }
}
impl<T: Graphic3dCView + ?Sized> Graphic3dCViewXr for T {}

/// Whether the posed camera is still at its copy: eye, distance, direction and up within
/// `gp::Resolution()` (the distance compared one way, as natively).
fn unchanged(copy: &Graphic3dCamera, posed: &Graphic3dCamera) -> bool {
    let resolution = gp::resolution();
    copy.eye().is_equal(&posed.eye(), resolution)
        && (copy.distance() - posed.distance()) <= resolution
        && copy.direction().is_equal(&posed.direction(), resolution)
        && copy.up().is_equal(&posed.up(), resolution)
}
