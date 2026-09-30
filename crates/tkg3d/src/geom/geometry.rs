// Rust translation of ModelingData/TKG3d/Geom/Geom_Geometry.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Geometry`: the root of the 3D transient geometry hierarchy.
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkg2d::geom2d::dump_class_name;
use rustycad_tkmath::gp::{Axis1, Axis2, Point3, Transform3, Vector3};
use std::io;

/// Common behavior of every Geom object: geometric transformations and copy.
///
/// The `*ed` members copy the object, move the copy and return it; the others
/// move the object itself, which every handle to it then sees.
/// `circle.translate(Vector3::new(1.0, 0.0, 0.0))?`.
pub trait GeomGeometry: Transient {
    /// Apply `transform` to this object (`Geom_Geometry::Transform`, pure virtual);
    /// `curve.transform(Transform3::default())?`.
    fn transform(&self, transform: Transform3) -> Result<(), Failure>;
    /// A new object of the same class with the same state (`Copy`, pure virtual).
    ///
    /// Only classes whose native copy constructor can raise fail (a trimmed curve
    /// re-trims its bounds); `let other: Handle<dyn GeomGeometry> = curve.copy()?;`.
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure>;
    /// Write the native JSON fields; depth 0 writes the class name only (`DumpJson`);
    /// `curve.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_geometry(stream, depth)
    }
    /// Mirror about a point (`Mirror(gp_Pnt)`); `curve.mirror_point(Point3::new(0.0, 0.0, 0.0))?`.
    fn mirror_point(&self, point: Point3) -> Result<(), Failure> {
        let mut transform = Transform3::default();
        transform.set_mirror_point(point);
        self.transform(transform)
    }
    /// Mirror about an axis (`Mirror(gp_Ax1)`); `curve.mirror_axis(axis)?`.
    fn mirror_axis(&self, axis: Axis1) -> Result<(), Failure> {
        let mut transform = Transform3::default();
        transform.set_mirror_axis(axis);
        self.transform(transform)
    }
    /// Mirror about a plane (`Mirror(gp_Ax2)`); `curve.mirror_plane(frame)?`.
    fn mirror_plane(&self, frame: Axis2) -> Result<(), Failure> {
        let mut transform = Transform3::default();
        transform.set_mirror_plane(frame);
        self.transform(transform)
    }
    /// Rotate around an axis (`Rotate`); `curve.rotate(axis, 0.5)?`.
    fn rotate(&self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        let mut transform = Transform3::default();
        transform.set_rotation(axis, angle)?;
        self.transform(transform)
    }
    /// Scale about a point (`Scale`); `curve.scale(Point3::new(0.0, 0.0, 0.0), 2.0)?`.
    fn scale(&self, point: Point3, factor: f64) -> Result<(), Failure> {
        let mut transform = Transform3::default();
        transform.set_scale(point, factor)?;
        self.transform(transform)
    }
    /// Translate by a vector (`Translate(gp_Vec)`); `curve.translate(Vector3::new(1.0, 0.0, 0.0))?`.
    fn translate(&self, vector: Vector3) -> Result<(), Failure> {
        let mut transform = Transform3::default();
        transform.set_translation(vector);
        self.transform(transform)
    }
    /// Translate from `from` to `to` (`Translate(gp_Pnt, gp_Pnt)`); `curve.translate_between(a, b)?`.
    fn translate_between(&self, from: Point3, to: Point3) -> Result<(), Failure> {
        self.translate(Vector3::between(from, to))
    }
    /// Mirrored copy about a point (`Mirrored(gp_Pnt)`); `curve.mirrored_point(p)?`.
    fn mirrored_point(&self, point: Point3) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.mirror_point(point)?;
        Ok(copy)
    }
    /// Mirrored copy about an axis (`Mirrored(gp_Ax1)`); `curve.mirrored_axis(axis)?`.
    fn mirrored_axis(&self, axis: Axis1) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.mirror_axis(axis)?;
        Ok(copy)
    }
    /// Mirrored copy about a plane (`Mirrored(gp_Ax2)`); `curve.mirrored_plane(frame)?`.
    fn mirrored_plane(&self, frame: Axis2) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.mirror_plane(frame)?;
        Ok(copy)
    }
    /// Rotated copy (`Rotated`); `curve.rotated(axis, 0.5)?`.
    fn rotated(&self, axis: Axis1, angle: f64) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.rotate(axis, angle)?;
        Ok(copy)
    }
    /// Scaled copy (`Scaled`); `curve.scaled(p, 2.0)?`.
    fn scaled(&self, point: Point3, factor: f64) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.scale(point, factor)?;
        Ok(copy)
    }
    /// Transformed copy (`Transformed`); `curve.transformed(transform)?`.
    fn transformed(&self, transform: Transform3) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.transform(transform)?;
        Ok(copy)
    }
    /// Translated copy (`Translated(gp_Vec)`); `curve.translated(v)?`.
    fn translated(&self, vector: Vector3) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.translate(vector)?;
        Ok(copy)
    }
    /// Copy translated from `from` to `to` (`Translated(gp_Pnt, gp_Pnt)`);
    /// `curve.translated_between(a, b)?`.
    fn translated_between(
        &self,
        from: Point3,
        to: Point3,
    ) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let copy: Handle<dyn GeomGeometry> = self.copy()?;
        copy.translate_between(from, to)?;
        Ok(copy)
    }
}
impl_transient_interface!(dyn GeomGeometry, "Geom_Geometry", StandardTransient);

/// `Geom_Geometry::DumpJson`: the class name only; `dump_geometry(&mut stream, -1)?`.
pub fn dump_geometry(stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
    dump_class_name(stream, "Geom_Geometry")
}
