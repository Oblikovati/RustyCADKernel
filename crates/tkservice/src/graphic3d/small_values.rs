//! Small value classes of Graphic3d: `Graphic3d_Vertex`, `Graphic3d_PolygonOffset` and
//! `Graphic3d_BufferRange`.
use crate::aspect::AspectPolygonOffsetMode;
use crate::json_dump::{class_begin, field_dumped, field_integer, field_real};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::Xyz;
use std::io;

/// A single-precision 3D vertex (`Graphic3d_Vertex`).
///
/// Example: `Graphic3dVertex::new(1.0, 2.0, 3.0).x() == 1.0`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Graphic3dVertex {
    /// `xyz`: the coordinates.
    pub xyz: [f32; 3],
}
impl Graphic3dVertex {
    /// A vertex from single-precision coordinates; `Graphic3dVertex::new(1.0, 2.0, 3.0)`.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { xyz: [x, y, z] }
    }
    /// A vertex from double coordinates, rounded to float; `Graphic3dVertex::from_f64(1.0, 2.0, 3.0)`.
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self {
            xyz: [x as f32, y as f32, z as f32],
        }
    }
    /// Sets the coordinates; `vertex.set_coord(1.0, 2.0, 3.0)`.
    pub fn set_coord(&mut self, x: f32, y: f32, z: f32) {
        self.xyz = [x, y, z];
    }
    /// Sets the coordinates from doubles, rounded to float; `vertex.set_coord_f64(1.0, 2.0, 3.0)`.
    pub fn set_coord_f64(&mut self, x: f64, y: f64, z: f64) {
        self.xyz = [x as f32, y as f32, z as f32];
    }
    /// The coordinates; `let (x, y, z) = vertex.coord();`.
    pub fn coord(&self) -> (f32, f32, f32) {
        (self.xyz[0], self.xyz[1], self.xyz[2])
    }
    /// The coordinates as doubles; `let (x, y, z) = vertex.coord_f64();`.
    pub fn coord_f64(&self) -> (f64, f64, f64) {
        (
            f64::from(self.xyz[0]),
            f64::from(self.xyz[1]),
            f64::from(self.xyz[2]),
        )
    }
    /// X; `vertex.x()`.
    pub fn x(&self) -> f32 {
        self.xyz[0]
    }
    /// Y; `vertex.y()`.
    pub fn y(&self) -> f32 {
        self.xyz[1]
    }
    /// Z; `vertex.z()`.
    pub fn z(&self) -> f32 {
        self.xyz[2]
    }
    /// The distance to another vertex, computed in single precision as natively.
    ///
    /// Example: `Graphic3dVertex::default().distance(&Graphic3dVertex::new(3.0, 4.0, 0.0)) == 5.0`.
    pub fn distance(&self, other: &Graphic3dVertex) -> f32 {
        let (dx, dy, dz) = (
            self.x() - other.x(),
            self.y() - other.y(),
            self.z() - other.z(),
        );
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    /// `DumpJson`: the coordinates as a `gp_XYZ` (key `Coord`); `vertex.dump_json(&mut s, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let (x, y, z) = self.coord_f64();
        let coord = Xyz::new(x, y, z);
        field_dumped(stream, depth, "&aCoord", &|s, _| coord.dump_json(s))
    }
}

/// Polygon offset parameters (`Graphic3d_PolygonOffset`).
///
/// Example: `Graphic3dPolygonOffset::default().factor == 1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Graphic3dPolygonOffset {
    /// `Mode`.
    pub mode: AspectPolygonOffsetMode,
    /// `Factor`.
    pub factor: f32,
    /// `Units`.
    pub units: f32,
}
impl Default for Graphic3dPolygonOffset {
    /// Fill mode, factor and units 1.
    fn default() -> Self {
        Self {
            mode: AspectPolygonOffsetMode::FILL,
            factor: 1.0,
            units: 1.0,
        }
    }
}
impl Graphic3dPolygonOffset {
    /// `DumpJson`: class name, mode, factor and units; `offset.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_PolygonOffset")?;
        field_integer(stream, "Mode", self.mode.0)?;
        field_real(stream, "Factor", f64::from(self.factor))?;
        field_real(stream, "Units", f64::from(self.units))
    }
}

/// A range of buffer elements (`Graphic3d_BufferRange`).
///
/// Example: `Graphic3dBufferRange::new(2, 3).upper() == 4`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Graphic3dBufferRange {
    /// `Start`: the first element.
    pub start: i32,
    /// `Length`: the number of elements.
    pub length: i32,
}
impl Graphic3dBufferRange {
    /// A range; `Graphic3dBufferRange::new(2, 3)`.
    pub fn new(start: i32, length: i32) -> Self {
        Self { start, length }
    }
    /// True for an empty range; `range.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    /// The last element; `range.upper()`.
    pub fn upper(&self) -> i32 {
        self.start.wrapping_add(self.length).wrapping_sub(1)
    }
    /// Empties the range; `range.clear()`.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Extends the range to cover another one; `range.unite(&other)`.
    pub fn unite(&mut self, other: &Graphic3dBufferRange) {
        if self.is_empty() {
            *self = *other;
            return;
        }
        if other.is_empty() {
            return;
        }
        let start = self.start.min(other.start);
        let last = self.upper().max(other.upper());
        self.start = start;
        self.length = last - start + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_and_range_follow_the_native_arithmetic() {
        let vertex = Graphic3dVertex::from_f64(3.0, 4.0, 0.0);
        assert_eq!(Graphic3dVertex::default().distance(&vertex), 5.0);
        assert_eq!(vertex.coord(), (3.0, 4.0, 0.0));
        let mut range = Graphic3dBufferRange::default();
        range.unite(&Graphic3dBufferRange::new(5, 2));
        range.unite(&Graphic3dBufferRange::new(1, 1));
        assert_eq!((range.start, range.length, range.upper()), (1, 6, 6));
        range.clear();
        assert!(range.is_empty());
        assert_eq!(
            Graphic3dPolygonOffset::default().mode,
            AspectPolygonOffsetMode::FILL
        );
    }
}
