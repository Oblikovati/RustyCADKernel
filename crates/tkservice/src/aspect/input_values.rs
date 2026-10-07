//! Small input and tracking values: `Aspect_ScrollDelta`, `Aspect_Touch`,
//! `Aspect_FrustumLRBT` and `Aspect_TrackedDevicePose`.
use super::{ASPECT_VKEY_FLAGS_NONE, AspectVKeyFlags};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkmath::gp::{Transform3, Vector3};
use std::ops::MulAssign;

/// Parameters of a mouse scroll action (`Aspect_ScrollDelta`).
///
/// Example: `AspectScrollDelta::with_delta(1.0, 0).has_point() == false`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectScrollDelta {
    /// `Point`: the scale position, (-1, -1) when undefined.
    pub point: Vec2<i32>,
    /// `Delta`: the delta in pixels.
    pub delta: f64,
    /// `Flags`: the key flags.
    pub flags: AspectVKeyFlags,
}
impl Default for AspectScrollDelta {
    /// No point, zero delta, no flags.
    fn default() -> Self {
        Self::with_delta(0.0, ASPECT_VKEY_FLAGS_NONE)
    }
}
impl AspectScrollDelta {
    /// A delta at a point (the native flags default is `Aspect_VKeyFlags_NONE`).
    ///
    /// Example: `AspectScrollDelta::new(Vec2::new(3, 4), 1.0, 0)`.
    pub fn new(point: Vec2<i32>, delta: f64, flags: AspectVKeyFlags) -> Self {
        Self {
            point,
            delta,
            flags,
        }
    }
    /// A delta with an undefined point; `AspectScrollDelta::with_delta(1.0, 0)`.
    pub fn with_delta(delta: f64, flags: AspectVKeyFlags) -> Self {
        Self::new(Vec2::new(-1, -1), delta, flags)
    }
    /// True when the point is defined (both coordinates >= 0); `delta.has_point()`.
    pub fn has_point(&self) -> bool {
        self.point.x >= 0 && self.point.y >= 0
    }
    /// Makes the point undefined; `delta.reset_point()`.
    pub fn reset_point(&mut self) {
        self.point = Vec2::new(-1, -1);
    }
}

/// A touch: its original and current position (`Aspect_Touch`).
///
/// Example: `AspectTouch::at(1.0, 2.0, false).delta()` is zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectTouch {
    /// `From`: the original position.
    pub from: Vec2<f64>,
    /// `To`: the current position.
    pub to: Vec2<f64>,
    /// `IsPreciseDevice`: a precise device (mouse cursor), not emulated from a touch screen.
    pub is_precise_device: bool,
}
impl Default for AspectTouch {
    /// Both positions at the origin, not precise.
    fn default() -> Self {
        Self::new(Vec2::new(0.0, 0.0), false)
    }
}
impl AspectTouch {
    /// A touch that starts and stays at `point`; `AspectTouch::new(p, true)`.
    pub fn new(point: Vec2<f64>, is_precise_device: bool) -> Self {
        Self {
            from: point,
            to: point,
            is_precise_device,
        }
    }
    /// `Aspect_Touch(theX, theY, theIsPreciseDevice)`; `AspectTouch::at(1.0, 2.0, false)`.
    pub fn at(x: f64, y: f64, is_precise_device: bool) -> Self {
        Self::new(Vec2::new(x, y), is_precise_device)
    }
    /// `To - From`; `touch.delta()`.
    pub fn delta(&self) -> Vec2<f64> {
        self.to - self.from
    }
}

/// Frustum boundaries (`Aspect_FrustumLRBT<Elem_t>`).
///
/// Example: `AspectFrustumLrbt::<f64>::default().multiplied(2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AspectFrustumLrbt<T> {
    /// `Left`.
    pub left: T,
    /// `Right`.
    pub right: T,
    /// `Bottom`.
    pub bottom: T,
    /// `Top`.
    pub top: T,
}
impl<T: Copy + MulAssign> AspectFrustumLrbt<T> {
    /// Scales every boundary; `frustum.multiply(0.5)`.
    pub fn multiply(&mut self, scale: T) {
        self.left *= scale;
        self.right *= scale;
        self.bottom *= scale;
        self.top *= scale;
    }
    /// A scaled copy; `frustum.multiplied(0.5)`.
    pub fn multiplied(&self, scale: T) -> Self {
        let mut copy = *self;
        copy.multiply(scale);
        copy
    }
}
impl AspectFrustumLrbt<f64> {
    /// The copy/cast constructor from single precision (`static_cast<double>`).
    ///
    /// Example: `AspectFrustumLrbt::<f64>::from_f32(&frustum)`.
    pub fn from_f32(other: &AspectFrustumLrbt<f32>) -> Self {
        Self {
            left: f64::from(other.left),
            right: f64::from(other.right),
            bottom: f64::from(other.bottom),
            top: f64::from(other.top),
        }
    }
}
impl AspectFrustumLrbt<f32> {
    /// The copy/cast constructor to single precision (`static_cast<float>`).
    ///
    /// Example: `AspectFrustumLrbt::<f32>::from_f64(&frustum)`.
    pub fn from_f64(other: &AspectFrustumLrbt<f64>) -> Self {
        Self {
            left: other.left as f32,
            right: other.right as f32,
            bottom: other.bottom as f32,
            top: other.top as f32,
        }
    }
}

/// One pose of a tracked XR device (`Aspect_TrackedDevicePose`).
///
/// Example: `AspectTrackedDevicePose::default().is_valid_pose == false`.
#[derive(Clone, Copy, Debug)]
pub struct AspectTrackedDevicePose {
    /// `Orientation`: the device to absolute transformation.
    pub orientation: Transform3,
    /// `Velocity` in tracker space, m/s.
    pub velocity: Vector3,
    /// `AngularVelocity` in radians/s.
    pub angular_velocity: Vector3,
    /// `IsValidPose`.
    pub is_valid_pose: bool,
    /// `IsConnectedDevice`.
    pub is_connected_device: bool,
}
impl Default for AspectTrackedDevicePose {
    /// Identity pose, zero velocities, invalid and disconnected.
    fn default() -> Self {
        Self {
            orientation: Transform3::default(),
            velocity: Vector3::default(),
            angular_velocity: Vector3::default(),
            is_valid_pose: false,
            is_connected_device: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_delta_point_and_touch_delta() {
        let mut scroll = AspectScrollDelta::new(Vec2::new(0, 5), 2.0, 0);
        assert!(scroll.has_point());
        scroll.reset_point();
        assert!(!scroll.has_point() && !AspectScrollDelta::default().has_point());
        let mut touch = AspectTouch::at(1.0, 2.0, true);
        touch.to = Vec2::new(4.0, 0.0);
        assert_eq!(touch.delta(), Vec2::new(3.0, -2.0));
    }

    #[test]
    fn frustum_scales_and_casts() {
        let frustum = AspectFrustumLrbt {
            left: -1.0,
            right: 1.0,
            bottom: -0.5,
            top: 0.5,
        };
        let scaled = frustum.multiplied(2.0);
        assert_eq!((scaled.left, scaled.top), (-2.0, 1.0));
        let single = AspectFrustumLrbt::<f32>::from_f64(&scaled);
        assert_eq!(AspectFrustumLrbt::<f64>::from_f32(&single), scaled);
    }
}
