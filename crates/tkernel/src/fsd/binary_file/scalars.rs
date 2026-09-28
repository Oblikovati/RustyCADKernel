// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! `FSD_BinaryFile` scalar I/O: one `fwrite`/`fread` of the native representation.
use super::{BinaryFile, type_mismatch, write_error};
use crate::standard::Failure;
use crate::storage::DriverPrimitives;

impl BinaryFile {
    /// `fwrite(bytes, size, 1)`, a `Storage_StreamWriteError` when nothing is written.
    pub(crate) fn write_item(&mut self, bytes: &[u8], member: &str) -> Result<(), Failure> {
        if bytes.is_empty() || self.file()?.write(bytes, bytes.len()) == 0 {
            return Err(write_error(member));
        }
        Ok(())
    }

    /// `fread(value, N, 1)` into the current bytes of the value: a short read keeps the
    /// bytes it got, as glibc does, and is a `Storage_StreamTypeMismatchError`.
    fn read_item<const N: usize>(
        &mut self,
        value: &mut [u8; N],
        member: &str,
    ) -> Result<(), Failure> {
        if self.file()?.read(value, N) == 0 {
            return Err(type_mismatch(member));
        }
        Ok(())
    }

    /// Reads a native `int` in place; shared by `GetInteger` and `GetReference`.
    pub(crate) fn read_int(&mut self, value: &mut i32, member: &str) -> Result<(), Failure> {
        let mut bytes = value.to_ne_bytes();
        let outcome = self.read_item(&mut bytes, member);
        *value = i32::from_ne_bytes(bytes);
        outcome
    }
}

impl DriverPrimitives for BinaryFile {
    fn put_reference(&mut self, value: i32) -> Result<(), Failure> {
        self.write_item(&value.to_ne_bytes(), "FSD_BinaryFile::PutReference")
    }

    fn put_character(&mut self, value: u8) -> Result<(), Failure> {
        self.write_item(&[value], "FSD_BinaryFile::PutCharacter")
    }

    fn put_ext_character(&mut self, value: u16) -> Result<(), Failure> {
        self.write_item(&value.to_ne_bytes(), "FSD_BinaryFile::PutExtCharacter")
    }

    fn put_integer(&mut self, value: i32) -> Result<(), Failure> {
        self.write_item(&value.to_ne_bytes(), "FSD_BinaryFile::PutInteger")
    }

    /// Booleans are written as the `int` 1 or 0.
    fn put_boolean(&mut self, value: bool) -> Result<(), Failure> {
        self.write_item(
            &i32::from(value).to_ne_bytes(),
            "FSD_BinaryFile::PutBoolean",
        )
    }

    fn put_real(&mut self, value: f64) -> Result<(), Failure> {
        self.write_item(&value.to_ne_bytes(), "FSD_BinaryFile::PutReal")
    }

    fn put_short_real(&mut self, value: f32) -> Result<(), Failure> {
        self.write_item(&value.to_ne_bytes(), "FSD_BinaryFile::PutShortReal")
    }

    fn get_reference(&mut self, value: &mut i32) -> Result<(), Failure> {
        self.read_int(value, "FSD_BinaryFile::GetReference")
    }

    fn get_character(&mut self, value: &mut u8) -> Result<(), Failure> {
        let mut bytes = [*value];
        let outcome = self.read_item(&mut bytes, "FSD_BinaryFile::GetCharacter");
        *value = bytes[0];
        outcome
    }

    fn get_ext_character(&mut self, value: &mut u16) -> Result<(), Failure> {
        let mut bytes = value.to_ne_bytes();
        let outcome = self.read_item(&mut bytes, "FSD_BinaryFile::GetExtCharacter");
        *value = u16::from_ne_bytes(bytes);
        outcome
    }

    fn get_integer(&mut self, value: &mut i32) -> Result<(), Failure> {
        self.read_int(value, "FSD_BinaryFile::GetInteger")
    }

    /// Any non-zero `int` reads as true; a short read leaves `value` unchanged.
    fn get_boolean(&mut self, value: &mut bool) -> Result<(), Failure> {
        let mut bytes = 0_i32.to_ne_bytes();
        self.read_item(&mut bytes, "FSD_BinaryFile::GetBoolean")?;
        *value = i32::from_ne_bytes(bytes) != 0;
        Ok(())
    }

    fn get_real(&mut self, value: &mut f64) -> Result<(), Failure> {
        let mut bytes = value.to_ne_bytes();
        let outcome = self.read_item(&mut bytes, "FSD_BinaryFile::GetReal");
        *value = f64::from_ne_bytes(bytes);
        outcome
    }

    fn get_short_real(&mut self, value: &mut f32) -> Result<(), Failure> {
        let mut bytes = value.to_ne_bytes();
        let outcome = self.read_item(&mut bytes, "FSD_BinaryFile::GetShortReal");
        *value = f32::from_ne_bytes(bytes);
        outcome
    }
}
