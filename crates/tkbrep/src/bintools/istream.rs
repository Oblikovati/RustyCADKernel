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
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_IStream.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_IStream`: the input stream of the shape reader, which knows its position
//! and the last object type it read, and follows back references.
use super::errors::byte_text;
use super::object_type::ObjectType;
use super::scalars::{read_exact, stream_get};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point3, Transform3};

/// Reads the values of the shape-reference format.
///
/// ```
/// use rustycad_tkbrep::bintools::{BinIStream, ObjectType};
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(vec![1, 1]);
/// let mut input = BinIStream::new(&mut stream);
/// assert_eq!(input.read_type(), ObjectType::REFERENCE8);
/// // A one-byte delta of 1 refers to the byte before the type byte.
/// assert_eq!(input.read_reference().unwrap(), u64::MAX);
/// ```
pub struct BinIStream<'s> {
    stream: &'s mut IoStream,
    position: u64,
    last_type: ObjectType,
}

impl<'s> BinIStream<'s> {
    /// `BinTools_IStream(theStream)`: starts at `tellg()` (-1, as an unsigned 64-bit
    /// value, when the stream is not good) with the Unknown last type;
    /// `BinIStream::new(&mut stream)`.
    pub fn new(stream: &'s mut IoStream) -> Self {
        let position = stream.tellg() as u64;
        Self {
            stream,
            position,
            last_type: ObjectType::UNKNOWN,
        }
    }

    /// `ReadType()`: the next byte as an object type (`EOF` gives 255); the position
    /// advances by one; `input.read_type()`.
    pub fn read_type(&mut self) -> ObjectType {
        self.last_type = ObjectType(stream_get(self.stream) as u8);
        self.position = self.position.wrapping_add(1);
        self.last_type
    }

    /// `LastType()`; `input.last_type()`.
    pub fn last_type(&self) -> ObjectType {
        self.last_type
    }

    /// `(LastType - EndShape - 1)` in `int` arithmetic: negative below the shape codes.
    fn shape_code(&self) -> i32 {
        i32::from(self.last_type.0) - i32::from(ObjectType::END_SHAPE.0) - 1
    }

    /// `ShapeType()`: the `TopAbs_ShapeEnum` value `(LastType - 200) >> 2`. It is
    /// returned as the raw enumerator: a last type below 200 gives a negative value and
    /// one above 235 a value past `TopAbs_SHAPE`, which the reader refuses;
    /// `input.shape_type()`.
    pub fn shape_type(&self) -> i32 {
        self.shape_code() >> 2
    }

    /// `ShapeOrientation()`: `(LastType - 200) & 3`; `input.shape_orientation()`.
    pub fn shape_orientation(&self) -> Orientation {
        Orientation::ALL[(self.shape_code() & 3) as usize]
    }

    /// `Position()`; `input.position()`.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// `GoTo(thePosition)`: `seekg` to the absolute position (a position above
    /// `i64::MAX` is a negative `std::streampos`, which fails the stream);
    /// `input.go_to(0)`.
    pub fn go_to(&mut self, position: u64) {
        self.stream.seekg(position as i64);
        self.position = position;
    }

    /// `IsReference()`: the last type is one of the four reference widths;
    /// `input.is_reference()`.
    pub fn is_reference(&self) -> bool {
        self.last_type.is_reference()
    }

    /// `ReadReference()`: the delta of the last reference type, and the position it
    /// refers to: `tellg()` after the type byte, minus the delta, minus the type byte.
    /// A zero delta raises Standard_Failure; an 8-bit delta at the end of the stream is
    /// `EOF` (-1) widened to 64 bits, as natively; `input.read_reference()?`.
    pub fn read_reference(&mut self) -> Result<u64, Failure> {
        let current = self.stream.tellg() as u64;
        let delta = self.reference_delta();
        if delta == 0 {
            return Err(Failure::new(&format!(
                "BinTools_IStream::ReadReference: invalid reference {}\n",
                byte_text(self.last_type.0)
            )));
        }
        Ok(current.wrapping_sub(delta).wrapping_sub(1))
    }

    /// The delta bytes of the last reference type; a short read keeps the bytes it
    /// got over the zero-initialised value, and a non-reference type reads nothing.
    fn reference_delta(&mut self) -> u64 {
        match self.last_type {
            ObjectType::REFERENCE8 => {
                self.position = self.position.wrapping_add(1);
                i64::from(stream_get(self.stream)) as u64
            }
            ObjectType::REFERENCE16 => u64::from(u16::from_ne_bytes(self.read_partial::<2>())),
            ObjectType::REFERENCE32 => u64::from(u32::from_ne_bytes(self.read_partial::<4>())),
            ObjectType::REFERENCE64 => u64::from_ne_bytes(self.read_partial::<8>()),
            _ => 0,
        }
    }

    /// `myStream->read(buffer, N)` without a check; the position advances by `N`.
    fn read_partial<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0_u8; N];
        self.stream.read(&mut bytes);
        self.position = self.position.wrapping_add(N as u64);
        bytes
    }

    /// `Stream()`: the wrapped stream, for the geometry readers; `input.stream()`.
    pub fn stream(&mut self) -> &mut IoStream {
        self.stream
    }

    /// `UpdatePosition()`: re-reads `tellg()` after the stream was used directly;
    /// `input.update_position()`.
    pub fn update_position(&mut self) {
        self.position = self.stream.tellg() as u64;
    }

    /// `operator bool()`: the stream has not failed; `input.is_ok()`.
    pub fn is_ok(&self) -> bool {
        !self.stream.fail()
    }

    /// `ReadReal()` / `operator>>(double&)`: `Storage_StreamTypeMismatchError` on a short
    /// read; `input.read_real()?`.
    pub fn read_real(&mut self) -> Result<f64, Failure> {
        let value = read_exact::<8>(self.stream).map(f64::from_ne_bytes)?;
        self.position = self.position.wrapping_add(8);
        Ok(value)
    }

    /// `ReadInteger()` / `operator>>(int&)`; `input.read_integer()?`.
    pub fn read_integer(&mut self) -> Result<i32, Failure> {
        let value = read_exact::<4>(self.stream).map(i32::from_ne_bytes)?;
        self.position = self.position.wrapping_add(4);
        Ok(value)
    }

    /// `ReadPnt()` / `operator>>(gp_Pnt&)`: three reals, the position advancing once all
    /// three are read; `input.read_pnt()?`.
    pub fn read_pnt(&mut self) -> Result<Point3, Failure> {
        let mut coordinates = [0.0; 3];
        for coordinate in &mut coordinates {
            *coordinate = read_exact::<8>(self.stream).map(f64::from_ne_bytes)?;
        }
        self.position = self.position.wrapping_add(24);
        Ok(Point3::new(coordinates[0], coordinates[1], coordinates[2]))
    }

    /// `ReadByte()` / `operator>>(uint8_t&)`: one byte without a check. At the end of the
    /// stream the native value is uninitialised; the port reads 0. The position always
    /// advances; `input.read_byte()`.
    pub fn read_byte(&mut self) -> u8 {
        self.read_partial::<1>()[0]
    }

    /// `ReadBool()` / `operator>>(bool&)`: a byte that is not 0; `input.read_bool()`.
    pub fn read_bool(&mut self) -> bool {
        self.read_byte() != 0
    }

    /// `ReadShortReal()` / `operator>>(float&)`: four bytes without a check (a short read
    /// keeps zero bytes where the native value is uninitialised);
    /// `input.read_short_real()`.
    pub fn read_short_real(&mut self) -> f32 {
        f32::from_ne_bytes(self.read_partial::<4>())
    }

    /// `operator>>(gp_Trsf&)`: twelve reals, row by row with the translation last, then
    /// `SetValues` (Standard_ConstructionError for a singular matrix);
    /// `input.read_transform()?`.
    pub fn read_transform(&mut self) -> Result<Transform3, Failure> {
        let mut rows = [[0.0; 4]; 3];
        for row in &mut rows {
            for value in row.iter_mut() {
                *value = self.read_real()?;
            }
        }
        let mut transform = Transform3::default();
        transform.set_values(rows)?;
        Ok(transform)
    }

    /// `ReadBools(b1, b2, b3)`: the bits 1, 2 and 4 of one byte; `input.read_bools3()`.
    pub fn read_bools3(&mut self) -> [bool; 3] {
        let byte = self.read_byte();
        [0, 1, 2].map(|bit| byte & (1 << bit) != 0)
    }

    /// `ReadBools(b1, ..., b7)`: the bits 1 to 64 of one byte; `input.read_bools7()`.
    pub fn read_bools7(&mut self) -> [bool; 7] {
        let byte = self.read_byte();
        [0, 1, 2, 3, 4, 5, 6].map(|bit| byte & (1 << bit) != 0)
    }
}
