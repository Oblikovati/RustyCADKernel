//! The projection (mapping) members of `Graphic3d_Camera`: type, scale, field of view, Z
//! range, aspect, stereo focus and IOD, tile and custom projections.
use super::camera::{DEFAULT_ZFAR, DEFAULT_ZNEAR, DTR_HALF};
use super::{
    Graphic3dCamera, Graphic3dCameraFocusType, Graphic3dCameraIodType, Graphic3dCameraProjection,
    Graphic3dCameraTile,
};
use crate::aspect::AspectFrustumLrbt;
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::real::real_epsilon;

/// `Standard_ASSERT_RAISE` (raised in release builds too) with the offending values.
fn assert_raise(
    condition: bool,
    message: &str,
    values: std::fmt::Arguments<'_>,
) -> Result<(), Failure> {
    if condition {
        return Ok(());
    }
    Err(program_error(&format!("{message}; got {values}")))
}

impl Graphic3dCamera {
    /// `CopyMappingData`: every projection setting of another camera, then its custom
    /// projections; `camera.copy_mapping_data(&other)`.
    pub fn copy_mapping_data(&self, other: &Graphic3dCamera) {
        let o = other.read(Clone::clone);
        self.set_zero_to_one_depth(o.is_zero_to_one_depth);
        self.set_projection_type(o.proj_type);
        self.set_fov_y(o.fov_y);
        self.set_fov_2d(o.fov_2d);
        // The other camera's range already passed these checks.
        let _ = self.set_z_range(o.z_near, o.z_far);
        self.set_aspect(o.aspect);
        self.set_scale(other.scale());
        self.set_z_focus(o.z_focus_type, o.z_focus);
        self.set_iod(o.iod_type, o.iod);
        self.set_tile(&o.tile);
        self.reset_custom_projection();
        if let Some([pl, hl, pr, hr]) = o.custom.stereo {
            self.set_custom_stereo_projection(&pl, &hl, &pr, &hr);
        } else if let Some((left, right)) = o.custom.frustums {
            self.set_custom_stereo_frustums(&left, &right);
        }
        if let Some(mono) = o.custom.mono {
            self.set_custom_mono_projection(&mono);
        }
    }
    /// The parallel scale (orthographic), or `distance * 2 * tan(fovy / 2)`; `camera.scale()`.
    pub fn scale(&self) -> f64 {
        self.read(|s| {
            if s.proj_type == Graphic3dCameraProjection::Orthographic {
                s.scale
            } else {
                s.distance * 2.0 * s.fov_y_tan
            }
        })
    }
    /// `SetScale`: perspective cameras move the eye to the distance of the scale.
    /// Example: `camera.set_scale(100.0)`.
    pub fn set_scale(&self, scale: f64) {
        if self.scale() == scale {
            return;
        }
        let (kind, fov_y_tan) = self.write(|s| {
            s.scale = scale;
            (s.proj_type, s.fov_y_tan)
        });
        if kind != Graphic3dCameraProjection::Orthographic {
            self.set_distance(scale * 0.5 / fov_y_tan);
        }
        self.write(|s| s.invalidate_projection());
    }
    /// The projection type; `camera.projection_type()`.
    pub fn projection_type(&self) -> Graphic3dCameraProjection {
        self.read(|s| s.proj_type)
    }
    /// Whether the projection is orthographic; `camera.is_orthographic()`.
    pub fn is_orthographic(&self) -> bool {
        self.projection_type() == Graphic3dCameraProjection::Orthographic
    }
    /// Whether the projection is stereo; `camera.is_stereo()`.
    pub fn is_stereo(&self) -> bool {
        self.projection_type() == Graphic3dCameraProjection::Stereo
    }
    /// `SetProjectionType`: leaving orthographic restores non-positive Z limits to the defaults.
    /// Example: `camera.set_projection_type(Graphic3dCameraProjection::Perspective)`.
    pub fn set_projection_type(&self, kind: Graphic3dCameraProjection) {
        self.write(|s| {
            if s.proj_type == kind {
                return;
            }
            if s.proj_type == Graphic3dCameraProjection::Orthographic {
                if s.z_near <= real_epsilon() {
                    s.z_near = DEFAULT_ZNEAR;
                }
                if s.z_far <= real_epsilon() {
                    s.z_far = DEFAULT_ZFAR;
                }
            }
            s.proj_type = kind;
            s.invalidate_projection();
        });
    }
    /// The vertical field of view in degrees; `camera.fov_y()`.
    pub fn fov_y(&self) -> f64 {
        self.read(|s| s.fov_y)
    }
    /// The horizontal field of view, `fovy * aspect`; `camera.fov_x()`.
    pub fn fov_x(&self) -> f64 {
        self.read(|s| s.fov_x)
    }
    /// The field-of-view limit of 2D elements; `camera.fov_2d()`.
    pub fn fov_2d(&self) -> f64 {
        self.read(|s| s.fov_2d)
    }
    /// `SetFOVy`; `camera.set_fov_y(30.0)`.
    pub fn set_fov_y(&self, fov_y: f64) {
        self.write(|s| {
            if s.fov_y == fov_y {
                return;
            }
            (s.fov_y, s.fov_x, s.fov_y_tan) = (fov_y, fov_y * s.aspect, (DTR_HALF * fov_y).tan());
            s.invalidate_projection();
        });
    }
    /// `SetFOV2d`; `camera.set_fov_2d(90.0)`.
    pub fn set_fov_2d(&self, fov: f64) {
        self.write(|s| {
            if s.fov_2d != fov {
                s.fov_2d = fov;
                s.invalidate_projection();
            }
        });
    }
    /// `SetZRange`: far must exceed near, and both be positive for non-orthographic
    /// cameras, else `Standard_ProgramError`. Example: `camera.set_z_range(1.0, 100.0)?`.
    pub fn set_z_range(&self, z_near: f64, z_far: f64) -> Result<(), Failure> {
        assert_raise(
            z_far > z_near,
            "ZFar should be greater than ZNear",
            format_args!("near {z_near}, far {z_far}"),
        )?;
        if !self.is_orthographic() {
            assert_raise(
                z_near > 0.0,
                "Only positive Z-Near is allowed for perspective camera",
                format_args!("{z_near}"),
            )?;
            assert_raise(
                z_far > 0.0,
                "Only positive Z-Far is allowed for perspective camera",
                format_args!("{z_far}"),
            )?;
        }
        self.write(|s| {
            if s.z_near != z_near || s.z_far != z_far {
                (s.z_near, s.z_far) = (z_near, z_far);
                s.invalidate_projection();
            }
        });
        Ok(())
    }
    /// The near plane distance; `camera.z_near()`.
    pub fn z_near(&self) -> f64 {
        self.read(|s| s.z_near)
    }
    /// The far plane distance; `camera.z_far()`.
    pub fn z_far(&self) -> f64 {
        self.read(|s| s.z_far)
    }
}

/// Depth range, aspect, stereo parameters and tile.
impl Graphic3dCamera {
    /// Whether the depth range is [0, 1] (else [-1, 1]); `camera.is_zero_to_one_depth()`.
    pub fn is_zero_to_one_depth(&self) -> bool {
        self.read(|s| s.is_zero_to_one_depth)
    }
    /// Sets the depth range; `camera.set_zero_to_one_depth(true)`.
    pub fn set_zero_to_one_depth(&self, zero_to_one: bool) {
        self.write(|s| {
            if s.is_zero_to_one_depth != zero_to_one {
                s.is_zero_to_one_depth = zero_to_one;
                s.invalidate_projection();
            }
        });
    }
    /// The width-to-height ratio; `camera.aspect()`.
    pub fn aspect(&self) -> f64 {
        self.read(|s| s.aspect)
    }
    /// `SetAspect`: also sets the horizontal field of view; `camera.set_aspect(1.5)`.
    pub fn set_aspect(&self, aspect: f64) {
        self.write(|s| {
            if s.aspect != aspect {
                (s.aspect, s.fov_x) = (aspect, s.fov_y * aspect);
                s.invalidate_projection();
            }
        });
    }
    /// The stereo focus; `camera.z_focus()`.
    pub fn z_focus(&self) -> f64 {
        self.read(|s| s.z_focus)
    }
    /// The stereo focus type; `camera.z_focus_type()`.
    pub fn z_focus_type(&self) -> Graphic3dCameraFocusType {
        self.read(|s| s.z_focus_type)
    }
    /// `SetZFocus`; `camera.set_z_focus(Graphic3dCameraFocusType::Absolute, 10.0)`.
    pub fn set_z_focus(&self, kind: Graphic3dCameraFocusType, z_focus: f64) {
        self.write(|s| {
            if s.z_focus_type != kind || s.z_focus != z_focus {
                (s.z_focus_type, s.z_focus) = (kind, z_focus);
                s.invalidate_projection();
            }
        });
    }
    /// The intraocular distance; `camera.iod()`.
    pub fn iod(&self) -> f64 {
        self.read(|s| s.iod)
    }
    /// The IOD type (`GetIODType`); `camera.iod_type()`.
    pub fn iod_type(&self) -> Graphic3dCameraIodType {
        self.read(|s| s.iod_type)
    }
    /// `SetIOD`; `camera.set_iod(Graphic3dCameraIodType::Absolute, 0.1)`.
    pub fn set_iod(&self, kind: Graphic3dCameraIodType, iod: f64) {
        self.write(|s| {
            if s.iod_type != kind || s.iod != iod {
                (s.iod_type, s.iod) = (kind, iod);
                s.invalidate_projection();
            }
        });
    }
    /// The tile; `camera.tile()`.
    pub fn tile(&self) -> Graphic3dCameraTile {
        self.read(|s| s.tile)
    }
    /// `SetTile` (equal tiles by `operator==` are ignored); `camera.set_tile(&tile)`.
    pub fn set_tile(&self, tile: &Graphic3dCameraTile) {
        self.write(|s| {
            if !s.tile.is_same(tile) {
                s.tile = *tile;
                s.invalidate_projection();
            }
        });
    }
}

/// Custom projections.
impl Graphic3dCamera {
    /// Drops every custom projection; `camera.reset_custom_projection()`.
    pub fn reset_custom_projection(&self) {
        self.write(|s| {
            if s.custom.mono.is_some() || s.custom.stereo.is_some() || s.custom.frustums.is_some() {
                s.custom = Default::default();
                s.invalidate_projection();
            }
        });
    }
    /// Whether custom stereo frustums are set; `camera.is_custom_stereo_frustum()`.
    pub fn is_custom_stereo_frustum(&self) -> bool {
        self.read(|s| s.custom.frustums.is_some())
    }
    /// Sets the left and right frustums (dropping custom stereo matrices); `camera.set_custom_stereo_frustums(&l, &r)`.
    pub fn set_custom_stereo_frustums(
        &self,
        left: &AspectFrustumLrbt<f64>,
        right: &AspectFrustumLrbt<f64>,
    ) {
        self.write(|s| {
            (s.custom.frustums, s.custom.stereo) = (Some((*left, *right)), None);
            s.invalidate_projection();
        });
    }
    /// Whether custom stereo matrices are set; `camera.is_custom_stereo_projection()`.
    pub fn is_custom_stereo_projection(&self) -> bool {
        self.read(|s| s.custom.stereo.is_some())
    }
    /// Sets the stereo projections and head-to-eye matrices (dropping custom frustums);
    /// `camera.set_custom_stereo_projection(&pl, &hl, &pr, &hr)`.
    pub fn set_custom_stereo_projection(
        &self,
        proj_l: &Mat4<f64>,
        head_to_eye_l: &Mat4<f64>,
        proj_r: &Mat4<f64>,
        head_to_eye_r: &Mat4<f64>,
    ) {
        self.write(|s| {
            (s.custom.stereo, s.custom.frustums) = (
                Some([*proj_l, *head_to_eye_l, *proj_r, *head_to_eye_r]),
                None,
            );
            s.invalidate_projection();
        });
    }
    /// Whether a custom mono projection is set; `camera.is_custom_mono_projection()`.
    pub fn is_custom_mono_projection(&self) -> bool {
        self.read(|s| s.custom.mono.is_some())
    }
    /// Sets the mono projection; `camera.set_custom_mono_projection(&projection)`.
    pub fn set_custom_mono_projection(&self, projection: &Mat4<f64>) {
        self.write(|s| {
            s.custom.mono = Some(*projection);
            s.invalidate_projection();
        });
    }
}
