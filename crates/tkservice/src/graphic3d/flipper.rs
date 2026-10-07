//! `Graphic3d_Flipper`: the transformation that keeps a reference plane (and its text or
//! labels) readable by flipping it when the view looks at it from behind or upside down.
use super::Graphic3dCameraScalar;
use crate::json_dump::{class_begin, field_dumped};
use rustycad_tkernel::ncollection::{Mat4, Vec3, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::BvhBox;
use rustycad_tkmath::gp::{Axis2, Direction3};
use std::io;
use std::ops::{Div, Sub};

/// A flipper of a reference plane; `Graphic3dFlipper::new(axis2)`.
pub struct Graphic3dFlipper {
    transient: StandardTransient,
    ref_plane: SharedState<Axis2>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dFlipper,
    "Graphic3d_Flipper",
    StandardTransient,
    transient
);

fn vec4<T: Graphic3dCameraScalar>(direction: Direction3) -> Vec4<T> {
    Vec4::new(
        T::from_f64(direction.x()),
        T::from_f64(direction.y()),
        T::from_f64(direction.z()),
        T::from_f64(1.0),
    )
}
fn xyz<T: Copy>(v: Vec4<T>) -> Vec3<T> {
    Vec3::new(v.x, v.y, v.z)
}
/// `Vec4(a.xyz() + b.xyz(), 1)`.
fn shifted<T: Graphic3dCameraScalar>(a: Vec4<T>, b: Vec4<T>) -> Vec4<T> {
    Vec4::new(a.x + b.x, a.y + b.y, a.z + b.z, T::from_f64(1.0))
}
fn negated_column<T: Graphic3dCameraScalar>(matrix: &mut Mat4<T>, column: usize) {
    let c = matrix.get_column(column);
    matrix.set_column3(column, Vec3::new(-c.x, -c.y, -c.z));
}

impl Graphic3dFlipper {
    /// A flipper of `ref_plane`; `Graphic3dFlipper::new(axis2)`.
    pub fn new(ref_plane: Axis2) -> Self {
        Self {
            transient: StandardTransient::default(),
            ref_plane: SharedState::new(ref_plane),
        }
    }
    /// The reference plane; `flipper.ref_plane()`.
    pub fn ref_plane(&self) -> Axis2 {
        *self.ref_plane.read()
    }
    /// Sets the reference plane; `flipper.set_ref_plane(axis2)`.
    pub fn set_ref_plane(&self, ref_plane: Axis2) {
        *self.ref_plane.write() = ref_plane;
    }
    /// The reference axes as homogeneous vectors: origin, X, Y, Z.
    fn reference<T: Graphic3dCameraScalar>(&self) -> [Vec4<T>; 4] {
        let plane = self.ref_plane();
        let o = plane.location();
        let origin = Vec4::new(
            T::from_f64(o.x()),
            T::from_f64(o.y()),
            T::from_f64(o.z()),
            T::from_f64(1.0),
        );
        [
            origin,
            vec4(plane.x_direction()),
            vec4(plane.y_direction()),
            vec4(plane.direction()),
        ]
    }
    /// `Compute`: the flip of the reference plane in the world-view, identity when its axes
    /// keep their orientation. Example: `flipper.compute(&world_view)`.
    pub fn compute<T: Graphic3dCameraScalar>(&self, world_view: &Mat4<T>) -> Mat4<T>
    where
        Vec4<T>: Sub<Output = Vec4<T>>,
    {
        let [origin, x, y, z] = self.reference::<T>();
        let mv_origin = *world_view * origin;
        let dir = |axis: Vec4<T>| xyz((*world_view * shifted(axis, origin)) - mv_origin);
        let (zero, one) = (T::from_f64(0.0), T::from_f64(1.0));
        let units = [
            Vec3::new(one, zero, zero),
            Vec3::new(zero, one, zero),
            Vec3::new(zero, zero, one),
        ];
        let reversed = [
            dir(x).dot(&units[0]) < zero,
            dir(y).dot(&units[1]) < zero,
            dir(z).dot(&units[2]) < zero,
        ];
        let mut transform = Mat4::<T>::identity();
        let columns = match reversed {
            [rx, ry, false] if rx || ry => [0, 1],
            [_, true, true] => [1, 2],
            [_, _, true] => [0, 2],
            _ => return Mat4::identity(),
        };
        columns
            .into_iter()
            .for_each(|c| negated_column(&mut transform, c));
        let mut axes = Mat4::<T>::identity();
        [x, y, z, origin]
            .into_iter()
            .enumerate()
            .for_each(|(c, v)| axes.set_column3(c, xyz(v)));
        let mut inverse = Mat4::<T>::identity();
        // A singular reference frame leaves the inverse as computed, as natively.
        let _ = axes.inverted_to(&mut inverse);
        axes * transform * inverse
    }
    /// `Apply(theWorldView, BVH_Box<T, 3>&)`: the box of the flipped corners (an invalid box or
    /// an identity flip leaves it). Example: `flipper.apply_bvh(&world_view, &mut bounds)`.
    pub fn apply_bvh<T: Graphic3dCameraScalar>(
        &self,
        world_view: &Mat4<T>,
        bounds: &mut BvhBox<Vec3<T>>,
    ) where
        Vec4<T>: Sub<Output = Vec4<T>> + Div<T, Output = Vec4<T>>,
        Vec3<T>: rustycad_tkmath::bvh::BvhVector,
    {
        let flip = self.compute(world_view);
        if flip.is_identity() || !bounds.is_valid() {
            return;
        }
        let (min, max) = (*bounds.corner_min(), *bounds.corner_max());
        let one = T::from_f64(1.0);
        let corners = [
            (min.x, min.y, min.z),
            (min.x, min.y, max.z),
            (min.x, max.y, min.z),
            (min.x, max.y, max.z),
            (max.x, min.y, min.z),
            (max.x, min.y, max.z),
            (max.x, max.y, min.z),
            (max.x, max.y, max.z),
        ];
        bounds.clear();
        for (x, y, z) in corners {
            let corner = flip * Vec4::new(x, y, z, one);
            let corner = corner / corner.w;
            bounds.add(&xyz(corner));
        }
    }
    /// `Apply(theWorldView, Bnd_Box&)`: the same on a `Bnd_Box` (void boxes are left).
    /// Example: `flipper.apply_bnd(&world_view, &mut bounds)?`.
    pub fn apply_bnd(
        &self,
        world_view: &Mat4<f64>,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        if bounds.is_void() {
            return Ok(());
        }
        let [x0, y0, z0, x1, y1, z1] = bounds.get()?;
        let mut bvh = BvhBox::from_corners(Vec3::new(x0, y0, z0), Vec3::new(x1, y1, z1));
        self.apply_bvh(world_view, &mut bvh);
        let (min, max) = (bvh.corner_min(), bvh.corner_max());
        *bounds = BoundingBox3::default();
        bounds.update_bounds([min.x, min.y, min.z], [max.x, max.y, max.z]);
        Ok(())
    }
    /// `DumpJson`; `flipper.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_Flipper")?;
        let plane = self.ref_plane();
        field_dumped(stream, depth, "&myRefPlane", &|s, _| plane.dump_json(s))
    }
}
