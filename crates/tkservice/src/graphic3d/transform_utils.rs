//! `Graphic3d_TransformUtils`: the fixed-function style matrix helpers (ortho, rotate, scale,
//! translate, project and unproject) over float and double matrices, in the native operation
//! order. The `MatrixType`/`VectorType` traits of the namespace are the plain
//! `Mat4<T>`/`Vec2<T>`/`Vec3<T>`/`Vec4<T>` types.
use super::Graphic3dCameraScalar;
use rustycad_tkernel::ncollection::{ElementCast, Mat4, Vec4};
use rustycad_tkmath::gp::Transform3;
use std::ops::{Mul, Neg};

/// The scalars of the helpers (`float` and `double`) with the C math functions of each.
pub trait Graphic3dTransformScalar: Graphic3dCameraScalar + Neg<Output = Self> {
    /// `std::sin` of the type; `0.0_f32.sin_native()`.
    fn sin_native(self) -> Self;
    /// `std::cos` of the type; `0.0_f64.cos_native()`.
    fn cos_native(self) -> Self;
    /// `std::sqrt` of the type; `4.0_f64.sqrt_native()`.
    fn sqrt_native(self) -> Self;
    /// `static_cast<double>`; `1.5_f32.to_f64_native()`.
    fn to_f64_native(self) -> f64;
    /// The implicit `int` to `T` conversion; `f32::from_i32_native(3)`.
    fn from_i32_native(value: i32) -> Self;
}
macro_rules! transform_scalar {
    ($ty:ty) => {
        impl Graphic3dTransformScalar for $ty {
            fn sin_native(self) -> Self {
                self.sin()
            }
            fn cos_native(self) -> Self {
                self.cos()
            }
            fn sqrt_native(self) -> Self {
                self.sqrt()
            }
            fn to_f64_native(self) -> f64 {
                self.into()
            }
            fn from_i32_native(value: i32) -> Self {
                value as $ty
            }
        }
    };
}
transform_scalar!(f32);
transform_scalar!(f64);

fn c<T: Graphic3dTransformScalar>(value: f64) -> T {
    T::from_f64(value)
}

/// `Convert`: the 4x4 matrix of a transformation (scaled 3x3 part and translation).
/// Example: `convert::<f64>(&trsf, &mut matrix)`.
pub fn convert<T: Graphic3dTransformScalar>(transformation: &Transform3, output: &mut Mat4<T>)
where
    f64: ElementCast<T>,
{
    output.init_identity();
    transformation.get_mat4(output);
}

/// `Ortho`: an orthographic projection (OpenGL `glOrtho`).
/// Example: `ortho(&mut matrix, -1.0, 1.0, -1.0, 1.0, 0.1, 10.0)`.
pub fn ortho<T: Graphic3dTransformScalar>(
    output: &mut Mat4<T>,
    [left, right, bottom, top, z_near, z_far]: [T; 6],
) {
    output.init_identity();
    let one = c::<T>(1.0);
    let inv_dx = one / (right - left);
    let inv_dy = one / (top - bottom);
    let inv_dz = one / (z_far - z_near);
    output.set_value(0, 0, c::<T>(2.0) * inv_dx);
    output.set_value(1, 1, c::<T>(2.0) * inv_dy);
    output.set_value(2, 2, c::<T>(-2.0) * inv_dz);
    output.set_value(0, 3, -(right + left) * inv_dx);
    output.set_value(1, 3, -(top + bottom) * inv_dy);
    output.set_value(2, 3, -(z_far + z_near) * inv_dz);
}

/// `Ortho2D`: [`ortho`] with the depth range [-1, 1]; `ortho_2d(&mut m, [-1.0, 1.0, -1.0, 1.0])`.
pub fn ortho_2d<T: Graphic3dTransformScalar>(output: &mut Mat4<T>, [l, r, b, t]: [T; 4]) {
    ortho(output, [l, r, b, t, c(-1.0), c(1.0)]);
}

/// `x * 0.5 + 0.5`, the NDC to [0, 1] mapping of `Project`.
fn half<T: Graphic3dTransformScalar>(value: T) -> T {
    value * c(0.5) + c(0.5)
}

/// `Project`: the window coordinates of an object point, `None` when its clip w is 0.
/// Example: `project([1.0, 2.0, 3.0], &model_view, &projection, [0, 0, 800, 600])`.
pub fn project<T: Graphic3dTransformScalar>(
    [x, y, z]: [T; 3],
    model_view: &Mat4<T>,
    projection: &Mat4<T>,
    viewport: [i32; 4],
) -> Option<[T; 3]>
where
    Mat4<T>: Mul<Vec4<T>, Output = Vec4<T>>,
{
    let out = *projection * (*model_view * Vec4::new(x, y, z, c(1.0)));
    if out.w == c(0.0) {
        return None;
    }
    let w = c::<T>(1.0) / out.w;
    let [ox, oy, oz] = [half(out.x * w), half(out.y * w), half(out.z * w)];
    let vp = viewport.map(T::from_i32_native);
    Some([ox * vp[2] + vp[0], oy * vp[3] + vp[1], oz])
}

/// `UnProject`: the object point of window coordinates, `None` for a singular matrix or a
/// clip w of 0. Example: `un_project([400.0, 300.0, 0.5], &model_view, &projection, viewport)`.
pub fn un_project<T: Graphic3dTransformScalar>(
    [x, y, z]: [T; 3],
    model_view: &Mat4<T>,
    projection: &Mat4<T>,
    viewport: [i32; 4],
) -> Option<[T; 3]>
where
    Mat4<T>: Mul<Vec4<T>, Output = Vec4<T>> + Mul<Output = Mat4<T>>,
{
    let mut unview = Mat4::<T>::identity();
    if !(*projection * *model_view).inverted_to(&mut unview) {
        return None;
    }
    let vp = viewport.map(T::from_i32_native);
    let (one, two) = (c::<T>(1.0), c::<T>(2.0));
    let nx = (x - vp[0]) / vp[2] * two - one;
    let ny = (y - vp[1]) / vp[3] * two - one;
    let out = unview * Vec4::new(nx, ny, z * two - one, one);
    if out.w == c(0.0) {
        return None;
    }
    let w = one / out.w;
    Some([out.x * w, out.y * w, out.z * w])
}

/// The single-axis rotation of `ConstructRotate` in the plane `(i, j)`.
fn axis_rotation<T: Graphic3dTransformScalar>(
    output: &mut Mat4<T>,
    [i, j]: [usize; 2],
    negative: bool,
    [sin, cos]: [T; 2],
) {
    output.set_value(i, i, cos);
    output.set_value(j, j, cos);
    let s = if negative { -sin } else { sin };
    output.set_value(i, j, -s);
    output.set_value(j, i, s);
}

/// The sole non-zero axis of `(x, y, z)`, if exactly one compares non-zero.
fn sole_axis<T: Graphic3dTransformScalar>(axis: [T; 3]) -> Option<usize> {
    let zero = c::<T>(0.0);
    let nonzero: Vec<usize> = (0..3).filter(|&k| axis[k] != zero).collect();
    (nonzero.len() == 1).then(|| nonzero[0])
}

/// `ConstructRotate`: writes the rotation by `angle` degrees around `(x, y, z)` into the
/// 3x3 part of `output`, leaving the other coefficients. A single-axis rotation uses the
/// closed form; an axis shorter than 1e-4 leaves the matrix. Example:
/// `construct_rotate(&mut m, 90.0, [0.0, 0.0, 1.0])`.
pub fn construct_rotate<T: Graphic3dTransformScalar>(output: &mut Mat4<T>, angle: T, axis: [T; 3]) {
    let radians = angle * c::<T>(std::f64::consts::PI / 180.0);
    let (sin, cos) = (radians.sin_native(), radians.cos_native());
    if let Some(k) = sole_axis(axis) {
        // X turns (y, z), Z turns (x, y); Y turns (x, z) with the opposite sign.
        let (pair, flip) = [([1, 2], false), ([0, 2], true), ([0, 1], false)][k];
        let negative = axis[k] < c(0.0);
        axis_rotation(output, pair, negative != flip, [sin, cos]);
        return;
    }
    let [x, y, z] = axis;
    let norm = (x * x + y * y + z * z).sqrt_native();
    if norm <= c(1.0e-4) {
        return;
    }
    let inv = c::<T>(1.0) / norm;
    general_rotation(output, [x * inv, y * inv, z * inv], [sin, cos]);
}

/// The arbitrary-axis rotation of `ConstructRotate` for a unit axis.
fn general_rotation<T: Graphic3dTransformScalar>(
    output: &mut Mat4<T>,
    [x, y, z]: [T; 3],
    [sin, cos]: [T; 2],
) {
    let (xx, yy, zz, xy, yz, zx) = (x * x, y * y, z * z, x * y, y * z, z * x);
    let (sin_x, sin_y, sin_z) = (x * sin, y * sin, z * sin);
    let k = c::<T>(1.0) - cos;
    let rows = [
        [k * xx + cos, k * xy - sin_z, k * zx + sin_y],
        [k * xy + sin_z, k * yy + cos, k * yz - sin_x],
        [k * zx - sin_y, k * yz + sin_x, k * zz + cos],
    ];
    for (r, row) in rows.into_iter().enumerate() {
        row.into_iter()
            .enumerate()
            .for_each(|(col, v)| output.set_value(r, col, v));
    }
}

/// `Rotate`: post-multiplies `output` by the rotation; `rotate(&mut m, 45.0, [1.0, 1.0, 0.0])`.
pub fn rotate<T: Graphic3dTransformScalar>(output: &mut Mat4<T>, angle: T, axis: [T; 3])
where
    Mat4<T>: std::ops::MulAssign,
{
    let mut rotation = Mat4::<T>::identity();
    construct_rotate(&mut rotation, angle, axis);
    *output *= rotation;
}

/// `Scale`: scales the first three columns; `scale(&mut m, [2.0, 2.0, 2.0])`.
pub fn scale<T: Graphic3dTransformScalar>(output: &mut Mat4<T>, factors: [T; 3]) {
    for (column, factor) in factors.into_iter().enumerate() {
        for row in 0..4 {
            output.set_value(row, column, output.get_value(row, column) * factor);
        }
    }
}

/// `Translate`: post-multiplies `output` by a translation, row by row in the native order.
/// Example: `translate(&mut m, [1.0, 0.0, 0.0])`.
pub fn translate<T: Graphic3dTransformScalar>(output: &mut Mat4<T>, [x, y, z]: [T; 3]) {
    for row in 0..4 {
        let value = output.get_value(row, 0) * x
            + output.get_value(row, 1) * y
            + output.get_value(row, 2) * z
            + output.get_value(row, 3);
        output.set_value(row, 3, value);
    }
}

/// `ScaleFactor`: the cube root of the 3x3 determinant, in double.
/// Example: `scale_factor(&Mat4::<f64>::identity()) == 1.0`.
pub fn scale_factor<T: Graphic3dTransformScalar>(matrix: &Mat4<T>) -> f64 {
    matrix.determinant_mat3().to_f64_native().powf(1.0 / 3.0)
}
