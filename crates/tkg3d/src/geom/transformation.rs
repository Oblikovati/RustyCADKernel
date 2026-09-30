// Rust translation of ModelingData/TKG3d/Geom/Geom_Transformation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Transformation`: a shared gp_Trsf.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::dump_field;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Point3, Transform3, TransformForm, Vector3};
use std::io;

/// A transient similarity of space; `Handle::new(GeomTransformation::default())`.
#[derive(Default)]
pub struct GeomTransformation {
    parent: StandardTransient,
    state: SharedState<Transform3>,
}
impl_transient!(
    GeomTransformation,
    "Geom_Transformation",
    StandardTransient,
    parent
);
impl Clone for GeomTransformation {
    fn clone(&self) -> Self {
        Self::new(self.trsf())
    }
}
impl GeomTransformation {
    /// Transformation from a gp one (`Geom_Transformation(gp_Trsf)`);
    /// `GeomTransformation::new(transform)`.
    pub fn new(transform: Transform3) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(transform),
        }
    }
    /// Point symmetry (`SetMirror(gp_Pnt)`); `t.set_mirror_point(p)`.
    pub fn set_mirror_point(&self, point: Point3) {
        self.state.write().set_mirror_point(point);
    }
    /// Axis symmetry (`SetMirror(gp_Ax1)`); `t.set_mirror_axis(axis)`.
    pub fn set_mirror_axis(&self, axis: Axis1) {
        self.state.write().set_mirror_axis(axis);
    }
    /// Plane symmetry (`SetMirror(gp_Ax2)`); `t.set_mirror_plane(frame)`.
    pub fn set_mirror_plane(&self, frame: Axis2) {
        self.state.write().set_mirror_plane(frame);
    }
    /// Rotation (`SetRotation`); `t.set_rotation(axis, 0.5)?`.
    pub fn set_rotation(&self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        self.state.write().set_rotation(axis, angle)
    }
    /// Scaling; Standard_ConstructionError for a null factor (`SetScale`); `t.set_scale(p, 2.0)?`.
    pub fn set_scale(&self, point: Point3, factor: f64) -> Result<(), Failure> {
        self.state.write().set_scale(point, factor)
    }
    /// Change of coordinate system (`SetTransformation(FromSystem1, ToSystem2)`);
    /// `t.set_transformation_between(from, to)`.
    pub fn set_transformation_between(&self, from: Axis3, to: Axis3) {
        self.state.write().set_transformation_between(from, to);
    }
    /// Change to a coordinate system (`SetTransformation(ToSystem)`); `t.set_transformation_to(frame)`.
    pub fn set_transformation_to(&self, frame: Axis3) {
        self.state.write().set_transformation_to(frame);
    }
    /// Translation (`SetTranslation(gp_Vec)`); `t.set_translation(v)`.
    pub fn set_translation(&self, vector: Vector3) {
        self.state.write().set_translation(vector);
    }
    /// Translation between points (`SetTranslation(P1, P2)`); `t.set_translation_between(a, b)`.
    pub fn set_translation_between(&self, from: Point3, to: Point3) {
        self.state.write().set_translation_between(from, to);
    }
    /// Replace by a gp transformation (`SetTrsf`); `t.set_trsf(transform)`.
    pub fn set_trsf(&self, transform: Transform3) {
        *self.state.write() = transform;
    }
    /// Whether the determinant is negative (`IsNegative`); `t.is_negative()`.
    pub fn is_negative(&self) -> bool {
        self.state.read().is_negative()
    }
    /// The form (`Form`); `t.form()`.
    pub fn form(&self) -> TransformForm {
        self.state.read().form()
    }
    /// The scale factor (`ScaleFactor`); `t.scale_factor()`.
    pub fn scale_factor(&self) -> f64 {
        self.state.read().scale_factor()
    }
    /// The gp transformation (`Trsf`); `t.trsf()`.
    pub fn trsf(&self) -> Transform3 {
        *self.state.read()
    }
    /// Coefficient at row 1..3, column 1..4 (`Value`); `t.value(1, 4)?`.
    pub fn value(&self, row: i32, col: i32) -> Result<f64, Failure> {
        self.state.read().value(row, col)
    }
    /// Invert in place; fails for a null scale (`Invert`); `t.invert()?`.
    pub fn invert(&self) -> Result<(), Failure> {
        self.state.write().invert()
    }
    /// Inverse as a new object (`Inverted`); `t.inverted()?`.
    pub fn inverted(&self) -> Result<Handle<GeomTransformation>, Failure> {
        let inverse: Transform3 = self.trsf().inverted()?;
        Ok(Handle::new(Self::new(inverse)))
    }
    /// `self * other` as a new object (`Multiplied`); `t.multiplied(&other)`.
    pub fn multiplied(&self, other: &GeomTransformation) -> Handle<GeomTransformation> {
        let own: Transform3 = self.trsf();
        Handle::new(Self::new(own.multiplied(other.trsf())))
    }
    /// `self = self * other` (`Multiply`); `t.multiply(&other)`.
    pub fn multiply(&self, other: &GeomTransformation) {
        let right: Transform3 = other.trsf();
        self.state.write().multiply(right);
    }
    /// Raise to the power `n` in place (`Power`); `t.power(3)?`.
    pub fn power(&self, n: i32) -> Result<(), Failure> {
        self.state.write().power(n)
    }
    /// Power as a new object (`Powered`); `t.powered(3)?`.
    pub fn powered(&self, n: i32) -> Result<Handle<GeomTransformation>, Failure> {
        let mut temp: Transform3 = self.trsf();
        temp.power(n)?;
        Ok(Handle::new(Self::new(temp)))
    }
    /// `self = other * self` (`PreMultiply`); `t.pre_multiply(&other)`.
    pub fn pre_multiply(&self, other: &GeomTransformation) {
        let left: Transform3 = other.trsf();
        self.state.write().pre_multiply(left);
    }
    /// Transform coordinates (`Transforms(X, Y, Z)`); `let (x, y, z) = t.transforms(1.0, 2.0, 3.0);`.
    pub fn transforms(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        self.trsf().transforms_coordinates(x, y, z)
    }
    /// A new object with the same transformation (`Copy`); `t.copy()`.
    pub fn copy(&self) -> Handle<GeomTransformation> {
        Handle::new(self.clone())
    }
    /// The native JSON dump: the gpTrsf field (`DumpJson`); `t.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let transform: Transform3 = self.trsf();
        dump_field(stream, depth, "gpTrsf", |nested, _| {
            transform.dump_json(nested)
        })
    }
}
