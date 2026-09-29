// Copyright (c) 2021 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_OStream.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_OStream`: the output stream of the shape writer, which tracks the byte
//! position of every object so that a later use can refer back to it.
use super::object_type::ObjectType;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3, Transform3};
use rustycad_tkmath::poly::Triangle;

/// Writes the values of the binary formats and counts the bytes. Every value is
/// written in the platform byte order: `DO_INVERSE` is not defined at the pinned
/// revision. The position advances by the size of each value even when the stream
/// fails, as natively.
///
/// ```
/// use rustycad_tkbrep::bintools::BinOStream;
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(Vec::new());
/// let mut out = BinOStream::new(&mut stream);
/// out.write_real(1.0);
/// out.write_integer(3);
/// assert_eq!(out.position(), 12);
/// ```
pub struct BinOStream<'s> {
    stream: &'s mut IoStream,
    position: u64,
}

impl<'s> BinOStream<'s> {
    /// `BinTools_OStream(theStream)`: starts at `tellp()` (-1, as an unsigned 64-bit
    /// value, for a failed stream); `BinOStream::new(&mut stream)`.
    pub fn new(stream: &'s mut IoStream) -> Self {
        let position = stream.tellp() as u64;
        Self { stream, position }
    }

    /// `Position()`: the position relative to the start of the stream;
    /// `out.position()`.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// The wrapped stream (the native class keeps a pointer to it); `out.stream()`.
    pub fn stream(&mut self) -> &mut IoStream {
        self.stream
    }

    fn put(&mut self, bytes: &[u8]) {
        self.stream.write(bytes);
        self.position += bytes.len() as u64;
    }

    /// `WriteReference(thePosition)`: the distance from the current position back to
    /// `position`, as the narrowest of the 8, 16, 32 and 64-bit reference records;
    /// `out.write_reference(0)`.
    pub fn write_reference(&mut self, position: u64) {
        let delta = self.position.wrapping_sub(position);
        if delta <= 0xFF {
            self.put(&[ObjectType::REFERENCE8.0, delta as u8]);
        } else if delta <= 0xFFFF {
            self.put(&[ObjectType::REFERENCE16.0]);
            self.put(&(delta as u16).to_ne_bytes());
        } else if delta <= 0xFFFF_FFFF {
            self.put(&[ObjectType::REFERENCE32.0]);
            self.put(&(delta as u32).to_ne_bytes());
        } else {
            self.put(&[ObjectType::REFERENCE64.0]);
            self.put(&delta.to_ne_bytes());
        }
    }

    /// `WriteShape(theType, theOrientation)`: the byte `200 + (type << 2) + orientation`;
    /// `out.write_shape(ShapeEnum::Edge, Orientation::Forward)`.
    pub fn write_shape(&mut self, shape_type: ShapeEnum, orientation: Orientation) {
        let code = u32::from(ObjectType::END_SHAPE.0) + 1 + ((shape_type as u32) << 2);
        // `uint8_t aType` keeps the low byte of the sum.
        self.put(&[(code + orientation as u32) as u8]);
    }

    /// `operator<<(BinTools_ObjectType)`; `out.write_object_type(ObjectType::CURVE)`.
    pub fn write_object_type(&mut self, object_type: ObjectType) {
        self.put(&[object_type.0]);
    }

    /// `operator<<(uint8_t)`; `out.write_byte(2)`.
    pub fn write_byte(&mut self, value: u8) {
        self.put(&[value]);
    }

    /// `operator<<(double)`; `out.write_real(0.5)`.
    pub fn write_real(&mut self, value: f64) {
        self.put(&value.to_ne_bytes());
    }

    /// `operator<<(bool)`: one byte, 1 or 0; `out.write_bool(true)`.
    pub fn write_bool(&mut self, value: bool) {
        self.put(&[u8::from(value)]);
    }

    /// `operator<<(int)`; `out.write_integer(-1)`.
    pub fn write_integer(&mut self, value: i32) {
        self.put(&value.to_ne_bytes());
    }

    /// `operator<<(char16_t)`; `out.write_ext_char(3)`.
    pub fn write_ext_char(&mut self, value: u16) {
        self.put(&value.to_ne_bytes());
    }

    fn put_reals(&mut self, values: &[f64]) {
        let bytes: Vec<u8> = values
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect();
        self.put(&bytes);
    }

    /// `operator<<(gp_Pnt)`: X, Y, Z in one write; `out.write_point3(point)`.
    pub fn write_point3(&mut self, point: Point3) {
        self.put_reals(&[point.x(), point.y(), point.z()]);
    }

    /// `operator<<(gp_Dir)`; `out.write_direction3(direction)`.
    pub fn write_direction3(&mut self, direction: Direction3) {
        self.put_reals(&[direction.x(), direction.y(), direction.z()]);
    }

    /// `operator<<(gp_Pnt2d)`; `out.write_point2(point)`.
    pub fn write_point2(&mut self, point: Point2) {
        self.put_reals(&[point.x(), point.y()]);
    }

    /// `operator<<(gp_Dir2d)`; `out.write_direction2(direction)`.
    pub fn write_direction2(&mut self, direction: Direction2) {
        self.put_reals(&[direction.x(), direction.y()]);
    }

    /// `operator<<(gp_Trsf)`: the three rows of the vectorial part, each followed by
    /// its translation coordinate; `out.write_transform(&transform)`.
    pub fn write_transform(&mut self, transform: &Transform3) {
        self.put_reals(&transform_values(transform));
    }

    /// `operator<<(Poly_Triangle)`: the three node indices; `out.write_triangle(&triangle)`.
    pub fn write_triangle(&mut self, triangle: &Triangle) {
        let (a, b, c) = triangle.get();
        let bytes: Vec<u8> = [a, b, c]
            .iter()
            .flat_map(|node| node.to_ne_bytes())
            .collect();
        self.put(&bytes);
    }

    /// `operator<<(NCollection_Vec3<float>)`; `out.write_vec3f(Vec3::new(0.0, 0.0, 1.0))`.
    pub fn write_vec3f(&mut self, value: Vec3<f32>) {
        let bytes: Vec<u8> = [value.x, value.y, value.z]
            .iter()
            .flat_map(|component| component.to_ne_bytes())
            .collect();
        self.put(&bytes);
    }

    /// `PutBools(b1, b2, b3)`: one byte with the bits 1, 2 and 4;
    /// `out.put_bools3(true, false, true)`.
    pub fn put_bools3(&mut self, first: bool, second: bool, third: bool) {
        self.put(&[pack_bools(&[first, second, third])]);
    }

    /// `PutBools(b1, ..., b7)`: one byte with the bits 1 to 64;
    /// `out.put_bools7([true; 7])`.
    pub fn put_bools7(&mut self, values: [bool; 7]) {
        self.put(&[pack_bools(&values)]);
    }
}

/// The bit `1 << i` for each true value.
fn pack_bools(values: &[bool]) -> u8 {
    values
        .iter()
        .enumerate()
        .fold(0, |byte, (bit, value)| byte | (u8::from(*value) << bit))
}

/// `M(1,1) M(1,2) M(1,3) T(1) ... M(3,3) T(3)` of `VectorialPart()` and
/// `TranslationPart()`, the order of both binary transformation writers.
pub(crate) fn transform_values(transform: &Transform3) -> [f64; 12] {
    let matrix = transform.vectorial_part();
    let translation = transform.translation_part();
    let offsets = [translation.x(), translation.y(), translation.z()];
    let mut values = [0.0; 12];
    for (row, (coefficients, offset)) in matrix.coefficients().iter().zip(offsets).enumerate() {
        values[row * 4..row * 4 + 3].copy_from_slice(coefficients);
        values[row * 4 + 3] = offset;
    }
    values
}
