// Created on: 1993-07-19
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of the static `Print` helpers shared by GeomTools_CurveSet.cxx,
// GeomTools_Curve2dSet.cxx and GeomTools_SurfaceSet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text layout every GeomTools printer repeats: the dump (`compact` false) adds
//! labels and commas, the compact form (the BRep ASCII geometry sections) writes a
//! type code and bare numbers. Numbers go through `operator<<` of the emulated
//! iostream, so they follow the stream's `precision()` exactly as natively.
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};

/// The coordinates `Print(gp_Pnt)` writes.
pub(crate) fn point3(point: Point3) -> [f64; 3] {
    [point.x(), point.y(), point.z()]
}

/// The coordinates `Print(gp_Dir)` writes.
pub(crate) fn direction3(direction: Direction3) -> [f64; 3] {
    [direction.x(), direction.y(), direction.z()]
}

/// The coordinates `Print(gp_Pnt2d)` writes.
pub(crate) fn point2(point: Point2) -> [f64; 2] {
    [point.x(), point.y()]
}

/// The coordinates `Print(gp_Dir2d)` writes.
pub(crate) fn direction2(direction: Direction2) -> [f64; 2] {
    [direction.x(), direction.y()]
}

/// One `Print(..., OS, compact)` call: the stream and the layout switch.
pub(crate) struct TextOut<'s> {
    stream: &'s mut IoStream,
    compact: bool,
}

impl<'s> TextOut<'s> {
    /// Starts a record on `stream`; `TextOut::new(&mut stream, true)`.
    pub(crate) fn new(stream: &'s mut IoStream, compact: bool) -> Self {
        Self { stream, compact }
    }

    /// Whether the compact (readable back) layout is active.
    pub(crate) fn compact(&self) -> bool {
        self.compact
    }

    /// The stream, for a nested `PrintCurve`/`PrintSurface` call.
    pub(crate) fn stream(&mut self) -> &mut IoStream {
        self.stream
    }

    /// `OS << CODE << " "` in compact form, else `OS << "Name"`.
    pub(crate) fn tag(&mut self, code: i32, name: &[u8]) {
        if self.compact {
            self.stream
                .insert_integer(i64::from(code))
                .insert_bytes(b" ");
        } else {
            self.stream.insert_bytes(name);
        }
    }

    /// `if (!compact) OS << text;`: a dump-only label.
    pub(crate) fn label(&mut self, text: &[u8]) {
        if !self.compact {
            self.stream.insert_bytes(text);
        }
    }

    /// `if (compact) OS << text;`: a compact-only separator.
    pub(crate) fn compact_text(&mut self, text: &[u8]) {
        if self.compact {
            self.stream.insert_bytes(text);
        }
    }

    /// `OS << text` in both layouts.
    pub(crate) fn text(&mut self, text: &[u8]) {
        self.stream.insert_bytes(text);
    }

    /// `OS << value` for a double, at the stream precision.
    pub(crate) fn real(&mut self, value: f64) {
        self.stream.insert_real(value);
    }

    /// `OS << value` for an int.
    pub(crate) fn int(&mut self, value: i32) {
        self.stream.insert_integer(i64::from(value));
    }

    /// `OS << (flag ? 1 : 0) << " "` in compact form, else `OS << " word"` when set.
    pub(crate) fn flag(&mut self, flag: bool, word: &[u8]) {
        if self.compact {
            self.int(i32::from(flag));
            self.text(b" ");
        } else if flag {
            self.text(word);
        }
    }

    /// `if (!compact) OS << ","; OS << " ";`: the separator between two numbers.
    pub(crate) fn separator(&mut self) {
        self.label(b",");
        self.text(b" ");
    }

    /// `Print(gp_Pnt/gp_Dir/gp_Pnt2d/gp_Dir2d)`: each coordinate followed by the
    /// separator, the last one by a single space.
    pub(crate) fn coords(&mut self, coords: &[f64]) {
        let last = coords.len().saturating_sub(1);
        for (index, value) in coords.iter().enumerate() {
            self.real(*value);
            if index == last {
                self.text(b" ");
            } else {
                self.separator();
            }
        }
    }

    /// A dump label followed by the coordinates of a point or direction.
    pub(crate) fn field(&mut self, label: &[u8], coords: &[f64]) {
        self.label(label);
        self.coords(coords);
    }

    /// `if (!compact) OS << "\n  " << std::setw(2) << i << " : ";`: a pole or knot index.
    pub(crate) fn index(&mut self, index: i32) {
        if !self.compact {
            self.text(b"\n  ");
            self.padded_int(index, 2);
            self.text(b" : ");
        }
    }

    /// `if (!compact) OS << "\n  " << setw(2) << i << ", " << setw(2) << j << " : ";`.
    pub(crate) fn index_pair(&mut self, row: i32, col: i32) {
        if !self.compact {
            self.text(b"\n  ");
            self.padded_int(row, 2);
            self.text(b", ");
            self.padded_int(col, 2);
            self.text(b" : ");
        }
    }

    /// `OS << std::setw(width) << value`.
    pub(crate) fn padded_int(&mut self, value: i32, width: i64) {
        self.stream.set_width(width);
        self.int(value);
    }

    /// `OS << "\n"; if (!compact) OS << "\n";`: the end of a record (the curve
    /// printers write the two newlines in the other order, which is the same text).
    pub(crate) fn end(&mut self) {
        self.text(b"\n");
        self.label(b"\n");
    }
}
