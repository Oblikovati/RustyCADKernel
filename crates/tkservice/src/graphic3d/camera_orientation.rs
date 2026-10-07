//! The eye, center, distance and axial-scale setters of `Graphic3d_Camera`.
use super::Graphic3dCamera;
use super::camera::Graphic3dCameraState;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkmath::gp::{Direction3, Point3, Xyz, resolution};

/// Re-aims the direction at `center` when the distance is not below the resolution.
fn aim_at(state: &mut Graphic3dCameraState, center: &Point3) -> Result<(), Failure> {
    if state.distance > resolution() {
        state.direction = Direction3::from_xyz(center.xyz() - state.eye.xyz())?;
    }
    Ok(())
}

impl Graphic3dCamera {
    /// `SetEyeAndCenter`: both points (the direction follows unless they coincide);
    /// `camera.set_eye_and_center(&eye, &center)?`.
    pub fn set_eye_and_center(&self, eye: &Point3, center: &Point3) -> Result<(), Failure> {
        self.write(|s| {
            if s.eye.is_equal(eye, 0.0) && s.center().is_equal(center, 0.0) {
                return Ok(());
            }
            s.eye = *eye;
            s.distance = eye.distance(center);
            aim_at(s, center)?;
            s.invalidate_orientation();
            Ok(())
        })
    }
    /// `SetEye`: the eye, keeping the center; `camera.set_eye(&eye)?`.
    pub fn set_eye(&self, eye: &Point3) -> Result<(), Failure> {
        self.write(|s| {
            if s.eye.is_equal(eye, 0.0) {
                return Ok(());
            }
            let center = s.center();
            s.eye = *eye;
            s.distance = s.eye.distance(&center);
            aim_at(s, &center)?;
            s.invalidate_orientation();
            Ok(())
        })
    }
    /// `SetCenter`: the center, keeping the eye. Native quirk: only a changed distance
    /// counts as a change. Example: `camera.set_center(&center)?`.
    pub fn set_center(&self, center: &Point3) -> Result<(), Failure> {
        self.write(|s| {
            let distance = s.eye.distance(center);
            if s.distance == distance {
                return Ok(());
            }
            s.distance = distance;
            aim_at(s, center)?;
            s.invalidate_orientation();
            Ok(())
        })
    }
    /// `SetDistance`: moves the eye along the direction, keeping the center;
    /// `camera.set_distance(100.0)`.
    pub fn set_distance(&self, distance: f64) {
        self.write(|s| {
            if s.distance == distance {
                return;
            }
            let center = s.center();
            s.distance = distance;
            s.eye = Point3::from(center.xyz() - s.direction.xyz() * distance);
            s.invalidate_orientation();
        });
    }
    /// `SetAxialScale`: a non-positive coefficient raises `Standard_OutOfRange`;
    /// `camera.set_axial_scale(Xyz::new(1.0, 2.0, 1.0))?`.
    pub fn set_axial_scale(&self, scale: Xyz) -> Result<(), Failure> {
        if scale.x() <= 0.0 || scale.y() <= 0.0 || scale.z() <= 0.0 {
            return Err(out_of_range(&format!(
                "Graphic3d_Camera::SetAxialScale, bad coefficient; got ({}, {}, {})",
                scale.x(),
                scale.y(),
                scale.z()
            )));
        }
        self.write(|s| {
            if !s.axial_scale.is_equal(&scale, 0.0) {
                s.axial_scale = scale;
                s.invalidate_orientation();
            }
        });
        Ok(())
    }
    /// `SetIdentityOrientation`: the eye at the origin looking along -Z with +Y up;
    /// `camera.set_identity_orientation()?`.
    pub fn set_identity_orientation(&self) -> Result<(), Failure> {
        self.set_eye_and_center(&Point3::new(0.0, 0.0, 0.0), &Point3::new(0.0, 0.0, -1.0))?;
        self.set_up(&rustycad_tkmath::gp::dy());
        Ok(())
    }
}
