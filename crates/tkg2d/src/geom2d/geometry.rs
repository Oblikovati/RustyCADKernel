// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Geometry.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Geometry`: the root of the 2D transient geometry hierarchy.
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkmath::gp::{Axis2d, Point2, Transform2, Vector2};
use std::io;

/// Common behavior of every Geom2d object: geometric transformations and copy.
///
/// The `*ed` members copy the object, move the copy and return it; the others
/// move the object itself, which every handle to it then sees.
/// `circle.translate(Vector2::new(1.0, 0.0))?`.
pub trait Geom2dGeometry: Transient {
    /// Apply `transform` to this object (`Geom2d_Geometry::Transform`, pure virtual);
    /// `curve.transform(Transform2::default())?`.
    fn transform(&self, transform: Transform2) -> Result<(), Failure>;
    /// A new object of the same class with the same state (`Copy`, pure virtual).
    ///
    /// Only a trimmed curve whose bounds were left invalid by a failed `SetTrim`
    /// fails, as its native copy constructor re-trims;
    /// `let other: Handle<dyn Geom2dGeometry> = curve.copy()?;`.
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure>;
    /// Write the native JSON fields; depth 0 writes the class name only (`DumpJson`);
    /// `curve.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_geometry(stream, depth)
    }
    /// Mirror about a point (`Mirror(gp_Pnt2d)`); `curve.mirror_point(Point2::new(0.0, 0.0))?`.
    fn mirror_point(&self, point: Point2) -> Result<(), Failure> {
        let mut transform = Transform2::default();
        transform.set_mirror_point(point);
        self.transform(transform)
    }
    /// Mirror about an axis (`Mirror(gp_Ax2d)`); `curve.mirror_axis(axis)?`.
    fn mirror_axis(&self, axis: Axis2d) -> Result<(), Failure> {
        let mut transform = Transform2::default();
        transform.set_mirror_axis(axis);
        self.transform(transform)
    }
    /// Rotate around a point (`Rotate`); `curve.rotate(Point2::new(0.0, 0.0), 0.5)?`.
    fn rotate(&self, point: Point2, angle: f64) -> Result<(), Failure> {
        let mut transform = Transform2::default();
        transform.set_rotation(point, angle);
        self.transform(transform)
    }
    /// Scale about a point (`Scale`); `curve.scale(Point2::new(0.0, 0.0), 2.0)?`.
    fn scale(&self, point: Point2, factor: f64) -> Result<(), Failure> {
        let mut transform = Transform2::default();
        transform.set_scale(point, factor);
        self.transform(transform)
    }
    /// Translate by a vector (`Translate(gp_Vec2d)`); `curve.translate(Vector2::new(1.0, 0.0))?`.
    fn translate(&self, vector: Vector2) -> Result<(), Failure> {
        let mut transform = Transform2::default();
        transform.set_translation(vector);
        self.transform(transform)
    }
    /// Translate from `from` to `to` (`Translate(gp_Pnt2d, gp_Pnt2d)`); `curve.translate_between(a, b)?`.
    fn translate_between(&self, from: Point2, to: Point2) -> Result<(), Failure> {
        self.translate(Vector2::between(from, to))
    }
    /// Mirrored copy about a point (`Mirrored(gp_Pnt2d)`); `curve.mirrored_point(p)?`.
    fn mirrored_point(&self, point: Point2) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: Handle<dyn Geom2dGeometry> = self.copy()?;
        copy.mirror_point(point)?;
        Ok(copy)
    }
    /// Mirrored copy about an axis (`Mirrored(gp_Ax2d)`); `curve.mirrored_axis(axis)?`.
    fn mirrored_axis(&self, axis: Axis2d) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: Handle<dyn Geom2dGeometry> = self.copy()?;
        copy.mirror_axis(axis)?;
        Ok(copy)
    }
    /// Rotated copy (`Rotated`); `curve.rotated(p, 0.5)?`.
    fn rotated(&self, point: Point2, angle: f64) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: Handle<dyn Geom2dGeometry> = self.copy()?;
        copy.rotate(point, angle)?;
        Ok(copy)
    }
    /// Scaled copy (`Scaled`); `curve.scaled(p, 2.0)?`.
    fn scaled(&self, point: Point2, factor: f64) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: Handle<dyn Geom2dGeometry> = self.copy()?;
        copy.scale(point, factor)?;
        Ok(copy)
    }
    /// Transformed copy (`Transformed`); `curve.transformed(transform)?`.
    fn transformed(&self, transform: Transform2) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: Handle<dyn Geom2dGeometry> = self.copy()?;
        copy.transform(transform)?;
        Ok(copy)
    }
    /// Translated copy (`Translated(gp_Vec2d)`); `curve.translated(v)?`.
    fn translated(&self, vector: Vector2) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: Handle<dyn Geom2dGeometry> = self.copy()?;
        copy.translate(vector)?;
        Ok(copy)
    }
    /// Copy translated from `from` to `to` (`Translated(gp_Pnt2d, gp_Pnt2d)`);
    /// `curve.translated_between(a, b)?`.
    fn translated_between(
        &self,
        from: Point2,
        to: Point2,
    ) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let copy: Handle<dyn Geom2dGeometry> = self.copy()?;
        copy.translate_between(from, to)?;
        Ok(copy)
    }
}
impl_transient_interface!(dyn Geom2dGeometry, "Geom2d_Geometry", StandardTransient);

/// `Geom2d_Geometry::DumpJson`: the class name only; `dump_geometry(&mut stream, -1)?`.
pub fn dump_geometry(stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
    dump_class_name(stream, "Geom2d_Geometry")
}

/// `OCCT_DUMP_TRANSIENT_CLASS_BEGIN`: the `className` field of one hierarchy level;
/// `dump_class_name(&mut stream, "Geom2d_Curve")?`.
pub fn dump_class_name(stream: &mut DumpStream, name: &str) -> io::Result<()> {
    rustycad_tkernel::standard::dump::add_values_separator(stream)?;
    stream.append(&format!("\"className\": \"{name}\""));
    Ok(())
}

/// `OCCT_DUMP_BASE_CLASS`: dump a base level unless the depth is exhausted;
/// `dump_base(&mut stream, depth, dump_geometry)?`.
pub fn dump_base(
    stream: &mut DumpStream,
    depth: i32,
    base: impl FnOnce(&mut DumpStream, i32) -> io::Result<()>,
) -> io::Result<()> {
    if depth == 0 {
        return Ok(());
    }
    rustycad_tkernel::standard::dump::add_values_separator(stream)?;
    base(stream, depth - 1)
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED`: a nested field dump unless the depth is exhausted;
/// `dump_field(&mut stream, depth, "pos", |s, _| frame.dump_json(s))?`.
pub fn dump_field(
    stream: &mut DumpStream,
    depth: i32,
    name: &str,
    field: impl FnOnce(&mut DumpStream, i32) -> io::Result<()>,
) -> io::Result<()> {
    if depth == 0 {
        return Ok(());
    }
    let mut nested = DumpStream::default();
    field(&mut nested, depth - 1)?;
    rustycad_tkernel::standard::dump::dump_key_to_class(stream, name, nested.text())
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` of a boolean, printed as the stream does (`1`/`0`);
/// `dump_flag(&mut stream, "Rational", true)?`.
pub fn dump_flag(stream: &mut DumpStream, name: &str, value: bool) -> io::Result<()> {
    rustycad_tkernel::standard::dump::dump_integer(stream, name, i32::from(value))
}
