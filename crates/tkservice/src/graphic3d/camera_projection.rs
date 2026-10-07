//! The matrix computations of `Graphic3d_Camera` (`computeProjection`, `orthoProj`,
//! `perspectiveProj`, `stereoEyeProj`, `LookOrientation`), generic over `float` and `double`
//! as the native templates, with the same operation order.
use super::{Graphic3dCameraProjection, Graphic3dCameraTile};
use crate::aspect::{AspectEye, AspectFrustumLrbt};
use rustycad_tkernel::ncollection::{
    ElementCast, Mat4, MatrixScalar, Vec3, VectorMagnitudeComponent,
};
use std::ops::Neg;

/// The element types of the camera matrices (`float`, `double`).
pub trait Graphic3dCameraScalar:
    MatrixScalar
    + Neg<Output = Self>
    + PartialOrd
    + VectorMagnitudeComponent
    + Default
    + ElementCast<Self>
{
    /// `static_cast<Elem_t>(double)`.
    fn from_f64(value: f64) -> Self;
}
impl Graphic3dCameraScalar for f64 {
    fn from_f64(value: f64) -> Self {
        value
    }
}
impl Graphic3dCameraScalar for f32 {
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

/// The camera values the projection reads (a snapshot of the camera fields).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ProjectionParams {
    pub(crate) kind: Graphic3dCameraProjection,
    pub(crate) is_zero_to_one_depth: bool,
    pub(crate) scale: f64,
    pub(crate) z_near: f64,
    pub(crate) z_far: f64,
    pub(crate) aspect: f64,
    pub(crate) fov_y_tan: f64,
    /// The IOD in world units (a relative IOD times the distance).
    pub(crate) iod: f64,
    /// The focus in world units (a relative focus times the distance).
    pub(crate) z_focus: f64,
    pub(crate) tile: Graphic3dCameraTile,
}

fn c<T: Graphic3dCameraScalar>(value: f64) -> T {
    T::from_f64(value)
}

/// `orthoProj`: the orthographic matrix of a frustum.
pub(crate) fn ortho_proj<T: Graphic3dCameraScalar>(
    out: &mut Mat4<T>,
    lrbt: &AspectFrustumLrbt<T>,
    near: T,
    far: T,
    zero_to_one: bool,
) {
    let (l, r, b, t) = (lrbt.left, lrbt.right, lrbt.bottom, lrbt.top);
    let rows = [
        [c::<T>(2.0) / (r - l), c(0.0), c(0.0), -(r + l) / (r - l)],
        [c(0.0), c::<T>(2.0) / (t - b), c(0.0), -(t + b) / (t - b)],
        if zero_to_one {
            [
                c(0.0),
                c(0.0),
                c::<T>(-1.0) / (far - near),
                -near / (far - near),
            ]
        } else {
            [
                c(0.0),
                c(0.0),
                c::<T>(-2.0) / (far - near),
                -(far + near) / (far - near),
            ]
        },
        [c(0.0), c(0.0), c(0.0), c(1.0)],
    ];
    set_rows(out, rows);
}

/// `perspectiveProj`: the perspective matrix of a frustum at the near plane.
pub(crate) fn perspective_proj<T: Graphic3dCameraScalar>(
    out: &mut Mat4<T>,
    lrbt: &AspectFrustumLrbt<T>,
    near: T,
    far: T,
    zero_to_one: bool,
) {
    let (l, r, b, t) = (lrbt.left, lrbt.right, lrbt.bottom, lrbt.top);
    let (z22, z23) = if zero_to_one {
        (far / (near - far), -(far * near) / (far - near))
    } else {
        (
            -(far + near) / (far - near),
            -(c::<T>(2.0) * far * near) / (far - near),
        )
    };
    let rows = [
        [
            (c::<T>(2.0) * near) / (r - l),
            c(0.0),
            (r + l) / (r - l),
            c(0.0),
        ],
        [
            c(0.0),
            (c::<T>(2.0) * near) / (t - b),
            (t + b) / (t - b),
            c(0.0),
        ],
        [c(0.0), c(0.0), z22, z23],
        [c(0.0), c(0.0), c(-1.0), c(0.0)],
    ];
    set_rows(out, rows);
}

fn set_rows<T: Graphic3dCameraScalar>(out: &mut Mat4<T>, rows: [[T; 4]; 4]) {
    for (row, values) in rows.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            *out.change_value(row, column) = *value;
        }
    }
}

/// `stereoEyeProj`: the perspective of one eye, shifted by half the IOD at the focus.
pub(crate) fn stereo_eye_proj<T: Graphic3dCameraScalar>(
    out: &mut Mat4<T>,
    lrbt: &AspectFrustumLrbt<T>,
    (near, far): (T, T),
    (iod, z_focus): (T, T),
    eye: AspectEye,
    zero_to_one: bool,
) {
    let dx = if eye == AspectEye::Left {
        c::<T>(0.5) * iod
    } else {
        c::<T>(-0.5) * iod
    };
    let shift = dx * near / z_focus;
    let shifted = AspectFrustumLrbt {
        left: lrbt.left + shift,
        right: lrbt.right + shift,
        ..*lrbt
    };
    perspective_proj(out, &shifted, near, far, zero_to_one);
}

/// `LookOrientation`: the view matrix of an eye, direction and up with an axial scale.
pub(crate) fn look_orientation<T: Graphic3dCameraScalar>(
    eye: Vec3<T>,
    forward: Vec3<T>,
    up: Vec3<T>,
    axial_scale: Vec3<T>,
) -> Mat4<T> {
    let mut forward = forward;
    forward.normalize();
    let mut side = Vec3::cross(&forward, &up);
    side.normalize();
    let up = Vec3::cross(&side, &forward);
    let mut look = Mat4::<T>::default();
    look.set_row3(0, side);
    look.set_row3(1, up);
    look.set_row3(2, Vec3::new(-forward.x, -forward.y, -forward.z));
    let mut out = Mat4::<T>::identity();
    out.multiply(&look);
    out.translate(Vec3::new(-eye.x, -eye.y, -eye.z));
    let mut scale = Mat4::<T>::default();
    *scale.change_value(0, 0) = axial_scale.x;
    *scale.change_value(1, 1) = axial_scale.y;
    *scale.change_value(2, 2) = axial_scale.z;
    out.multiply(&scale);
    out
}

/// The mono frustum at the near plane (half sizes from the scale or the field of view, then
/// the aspect, then the tile sub-area).
pub(crate) fn mono_frustum<T: Graphic3dCameraScalar>(
    params: &ProjectionParams,
) -> AspectFrustumLrbt<T> {
    let (z_near, aspect) = (c::<T>(params.z_near), c::<T>(params.aspect));
    let half = if params.kind == Graphic3dCameraProjection::Orthographic {
        c::<T>(params.scale) * c(0.5)
    } else {
        z_near * c(params.fov_y_tan)
    };
    let (mut dx, mut dy) = (half, half);
    if aspect > c(1.0) {
        dx *= aspect;
    } else {
        dy /= aspect;
    }
    let lrbt = AspectFrustumLrbt {
        left: -dx,
        right: dx,
        bottom: -dy,
        top: dy,
    };
    if !params.tile.is_valid() {
        return lrbt;
    }
    tiled_frustum(&params.tile, dx, dy)
}

/// The frustum of a tile of the full view (`myTile.IsValid()` branch).
fn tiled_frustum<T: Graphic3dCameraScalar>(
    tile: &Graphic3dCameraTile,
    dx: T,
    dy: T,
) -> AspectFrustumLrbt<T> {
    let (full_x, full_y) = (c::<T>(2.0) * dx, c::<T>(2.0) * dy);
    let offset = tile.offset_lower_left();
    let as_t = |value: i32| c::<T>(f64::from(value));
    AspectFrustumLrbt {
        left: -dx + full_x * as_t(offset.x) / as_t(tile.total_size.x),
        right: -dx + full_x * as_t(offset.x + tile.tile_size.x) / as_t(tile.total_size.x),
        bottom: -dy + full_y * as_t(offset.y) / as_t(tile.total_size.y),
        top: -dy + full_y * as_t(offset.y + tile.tile_size.y) / as_t(tile.total_size.y),
    }
}

/// The custom projections of a camera (`myCustomProjMat*`, `myCustomFrustum*` and their flags).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CustomProjection {
    /// `myCustomProjMatM` when `myIsCustomProjMatM`.
    pub(crate) mono: Option<Mat4<f64>>,
    /// Left projection, left head-to-eye, right projection, right head-to-eye when
    /// `myIsCustomProjMatLR`.
    pub(crate) stereo: Option<[Mat4<f64>; 4]>,
    /// Left and right frustums when `myIsCustomFrustomLR`.
    pub(crate) frustums: Option<(AspectFrustumLrbt<f64>, AspectFrustumLrbt<f64>)>,
}

/// `Aspect_FrustumLRBT<Elem_t>(const Aspect_FrustumLRBT<double>&)`: `static_cast` of each side.
fn cast_lrbt<T: Graphic3dCameraScalar>(frustum: &AspectFrustumLrbt<f64>) -> AspectFrustumLrbt<T> {
    AspectFrustumLrbt {
        left: c(frustum.left),
        right: c(frustum.right),
        bottom: c(frustum.bottom),
        top: c(frustum.top),
    }
}

fn converted<T: Graphic3dCameraScalar>(matrix: &Mat4<f64>) -> Mat4<T>
where
    f64: ElementCast<T>,
{
    Mat4::<T>::from_other(matrix)
}

/// The left and right projections of the stereo family.
fn stereo_eyes<T: Graphic3dCameraScalar>(
    params: &ProjectionParams,
    custom: &CustomProjection,
    lrbt: &AspectFrustumLrbt<T>,
    add_head_to_eye: bool,
) -> (Mat4<T>, Mat4<T>)
where
    f64: ElementCast<T>,
{
    let (near, far, zero_to_one) = (
        c::<T>(params.z_near),
        c::<T>(params.z_far),
        params.is_zero_to_one_depth,
    );
    let (mut left, mut right) = (Mat4::<T>::identity(), Mat4::<T>::identity());
    if let Some([pl, hl, pr, hr]) = custom.stereo {
        let (l, r) = if add_head_to_eye {
            (pl * hl, pr * hr)
        } else {
            (pl, pr)
        };
        return (converted(&l), converted(&r));
    }
    if let Some((fl, fr)) = custom.frustums {
        perspective_proj(
            &mut left,
            &cast_lrbt::<T>(&fl).multiplied(near),
            near,
            far,
            zero_to_one,
        );
        perspective_proj(
            &mut right,
            &cast_lrbt::<T>(&fr).multiplied(near),
            near,
            far,
            zero_to_one,
        );
    } else {
        let (iod, focus) = (c::<T>(params.iod), c::<T>(params.z_focus));
        stereo_eye_proj(
            &mut left,
            lrbt,
            (near, far),
            (iod, focus),
            AspectEye::Left,
            zero_to_one,
        );
        stereo_eye_proj(
            &mut right,
            lrbt,
            (near, far),
            (iod, focus),
            AspectEye::Right,
            zero_to_one,
        );
    }
    (left, right)
}

/// `computeProjection`: the mono, left and right projections; with `add_head_to_eye` the eye
/// projections include the half-IOD translation (or the custom head-to-eye matrices).
pub(crate) fn compute_projection<T: Graphic3dCameraScalar>(
    params: &ProjectionParams,
    custom: &CustomProjection,
    add_head_to_eye: bool,
) -> [Mat4<T>; 3]
where
    f64: ElementCast<T>,
{
    use Graphic3dCameraProjection as P;
    let lrbt = mono_frustum::<T>(params);
    let (near, far, zero_to_one) = (
        c::<T>(params.z_near),
        c::<T>(params.z_far),
        params.is_zero_to_one_depth,
    );
    let mut mono = custom
        .mono
        .map_or_else(Mat4::<T>::identity, |m| converted(&m));
    let (mut left, mut right) = (Mat4::<T>::identity(), Mat4::<T>::identity());
    if custom.mono.is_none() {
        let project = if params.kind == P::Orthographic {
            ortho_proj::<T>
        } else {
            perspective_proj::<T>
        };
        project(&mut mono, &lrbt, near, far, zero_to_one);
    }
    if params.kind != P::Orthographic && params.kind != P::Perspective {
        (left, right) = stereo_eyes(params, custom, &lrbt, add_head_to_eye);
        let iod = c::<T>(params.iod);
        if add_head_to_eye && custom.stereo.is_none() && iod != c(0.0) {
            left.translate(Vec3::new(c::<T>(0.5) * iod, c(0.0), c(0.0)));
            right.translate(Vec3::new(c::<T>(-0.5) * iod, c(0.0), c(0.0)));
        }
    }
    match params.kind {
        P::MonoLeftEye => mono = left,
        P::MonoRightEye => mono = right,
        _ => {}
    }
    [mono, left, right]
}
