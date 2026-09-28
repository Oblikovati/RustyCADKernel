// Copyright (c) 1996-1999 Matra Datavision
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

//! `FSD_File` scalars: each value in decimal followed by a space.
use super::{TextDialect, TextFile};
use crate::standard::{Failure, strtod};
use crate::storage::{DriverPrimitives, StreamException};

/// `char realbuffer[100]` of `GetReal`/`GetShortReal`: `>>` stores at most 99 bytes.
const REAL_BUFFER: usize = 100;

fn mismatch(member: &str) -> Failure {
    StreamException::TypeMismatch.raise(&format!("{member}: the text is not a valid value"))
}

/// `OSD::CStringToReal`: `Strtod` of the whole C string; the value is stored even when
/// characters remain, which is then a failure.
pub(crate) fn c_string_to_real(text: &[u8], value: &mut f64) -> bool {
    let text = &text[..text.iter().position(|byte| *byte == 0).unwrap_or(text.len())];
    let ascii = text.iter().position(|byte| !byte.is_ascii()).unwrap_or(text.len());
    let prefix = std::str::from_utf8(&text[..ascii]).unwrap_or_default();
    let (parsed, end) = strtod(prefix);
    *value = parsed;
    end == text.len()
}

impl<D: TextDialect> TextFile<D> {
    /// `myStream << value << " "` for any integer insertion, then the `bad()` check.
    fn put_decimal(&mut self, value: i64, member: &str) -> Result<(), Failure> {
        self.stream.insert_integer(value);
        self.stream.insert_bytes(b" ");
        self.check_written(member)
    }

    /// `myStream >> realbuffer` then `OSD::CStringToReal` into `value`.
    fn get_real_text(&mut self, value: &mut f64, member: &str) -> Result<(), Failure> {
        let word = self.stream.extract_word(REAL_BUFFER);
        if self.stream.fail() {
            return Err(mismatch(member));
        }
        if !c_string_to_real(&word, value) {
            return Err(mismatch(member));
        }
        Ok(())
    }
}

impl<D: TextDialect> DriverPrimitives for TextFile<D> {
    fn put_reference(&mut self, value: i32) -> Result<(), Failure> {
        self.put_decimal(i64::from(value), "FSD_File::PutReference")
    }

    /// The signed `char` widens to `unsigned short`: byte 0xB0 is written as 65456.
    fn put_character(&mut self, value: u8) -> Result<(), Failure> {
        let widened = i16::from(value as i8) as u16;
        self.put_decimal(i64::from(widened), "FSD_File::PutCharacter")
    }

    /// Written as `(short)value`: unit 0xFFFF is written as -1.
    fn put_ext_character(&mut self, value: u16) -> Result<(), Failure> {
        self.put_decimal(i64::from(value as i16), "FSD_File::PutExtCharacter")
    }

    fn put_integer(&mut self, value: i32) -> Result<(), Failure> {
        self.put_decimal(i64::from(value), "FSD_File::PutInteger")
    }

    fn put_boolean(&mut self, value: bool) -> Result<(), Failure> {
        self.put_decimal(i64::from(value), "FSD_File::PutBoolean")
    }

    fn put_real(&mut self, value: f64) -> Result<(), Failure> {
        self.stream.insert_real(value);
        self.stream.insert_bytes(b" ");
        self.check_written("FSD_File::PutReal")
    }

    fn put_short_real(&mut self, value: f32) -> Result<(), Failure> {
        self.stream.insert_real(f64::from(value));
        self.stream.insert_bytes(b" ");
        self.check_written("FSD_File::PutShortReal")
    }

    fn get_reference(&mut self, value: &mut i32) -> Result<(), Failure> {
        self.extract_integer(value, "FSD_File::GetReference")
    }

    /// Reads an `unsigned short`; a failed extraction that still stored a non-zero value
    /// clears the stream and keeps it (the SGI workaround of the native code).
    fn get_character(&mut self, value: &mut u8) -> Result<(), Failure> {
        let mut wide = 0_u16;
        if !self.stream.extract_unsigned_short(&mut wide) {
            if wide == 0 {
                return Err(mismatch("FSD_File::GetCharacter"));
            }
            self.stream.clear(0);
        }
        *value = wide as u8;
        Ok(())
    }

    /// Reads a `short`; `value` changes only on success.
    fn get_ext_character(&mut self, value: &mut u16) -> Result<(), Failure> {
        let mut short = 0_i16;
        if !self.stream.extract_short(&mut short) {
            return Err(mismatch("FSD_File::GetExtCharacter"));
        }
        *value = short as u16;
        Ok(())
    }

    fn get_integer(&mut self, value: &mut i32) -> Result<(), Failure> {
        self.extract_integer(value, "FSD_File::GetInteger")
    }

    fn get_boolean(&mut self, value: &mut bool) -> Result<(), Failure> {
        if !self.stream.extract_bool(value) {
            return Err(mismatch("FSD_File::GetBoolean"));
        }
        Ok(())
    }

    /// `CStringToReal` stores its parse even when the text has trailing characters.
    fn get_real(&mut self, value: &mut f64) -> Result<(), Failure> {
        self.get_real_text(value, "FSD_File::GetReal")
    }

    /// The parse goes through a `double` that is stored only on success.
    fn get_short_real(&mut self, value: &mut f32) -> Result<(), Failure> {
        let mut real = 0.0;
        self.get_real_text(&mut real, "FSD_File::GetShortReal")?;
        *value = real as f32;
        Ok(())
    }
}
