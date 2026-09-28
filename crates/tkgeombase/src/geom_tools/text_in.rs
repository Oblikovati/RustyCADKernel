// Created on: 1993-01-21
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
// Rust translation of GeomTools::GetReal (GeomTools.cxx) and of the static
// `operator>>` for gp_Pnt, gp_Dir, gp_Pnt2d and gp_Dir2d repeated in
// GeomTools_CurveSet.cxx, GeomTools_Curve2dSet.cxx and GeomTools_SurfaceSet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The token readers of the GeomTools text formats.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, strtod};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};

/// `char aBuffer[THE_BUFFER_SIZE]` of GetReal: "the most optimal buffer length not
/// less then 25" for an IEEE-754 double in a standard stream.
const REAL_BUFFER: usize = 32;

/// `GeomTools::GetReal`: one word of at most 30 characters (the stream width is set
/// to 31 for the extraction, then restored), converted by `Strtod`. The value is 0
/// when the stream is already at end of file, when no word can be extracted, and when
/// the word has no numeric prefix; a longer word leaves its tail for the next read.
///
/// ```
/// use rustycad_tkernel::storage::IoStream;
/// use rustycad_tkgeombase::geom_tools;
/// let mut stream = IoStream::from_bytes(b" 1.5e3 x".to_vec());
/// assert_eq!(geom_tools::get_real(&mut stream), 1500.0);
/// assert_eq!(geom_tools::get_real(&mut stream), 0.0);
/// ```
pub fn get_real(stream: &mut IoStream) -> f64 {
    if stream.eof() {
        return 0.0;
    }
    let previous = stream.set_width(REAL_BUFFER as i64 - 1);
    let word = stream.extract_word(REAL_BUFFER);
    stream.set_width(previous);
    c_strtod(&word)
}

/// `Strtod(aBuffer, nullptr)` on the extracted word. A non-ASCII byte cannot extend a
/// C-locale number, so only the ASCII prefix is parsed.
fn c_strtod(word: &[u8]) -> f64 {
    let end = word
        .iter()
        .position(|byte| !byte.is_ascii())
        .unwrap_or(word.len());
    let text = std::str::from_utf8(&word[..end]).expect("an ASCII prefix is UTF-8");
    strtod(text).0
}

/// `IS >> gp_Pnt`: three GetReal, then `SetCoord`.
pub(crate) fn read_point3(stream: &mut IoStream) -> Point3 {
    let x = get_real(stream);
    let y = get_real(stream);
    let z = get_real(stream);
    Point3::new(x, y, z)
}

/// `IS >> gp_Dir`: three GetReal, then `SetCoord`, which raises
/// Standard_ConstructionError for a null vector once the three words are consumed.
pub(crate) fn read_direction3(stream: &mut IoStream) -> Result<Direction3, Failure> {
    let x = get_real(stream);
    let y = get_real(stream);
    let z = get_real(stream);
    let mut direction = Direction3::default();
    direction.set_coordinates(x, y, z)?;
    Ok(direction)
}

/// `IS >> gp_Pnt2d`: two GetReal, then `SetCoord`.
pub(crate) fn read_point2(stream: &mut IoStream) -> Point2 {
    let x = get_real(stream);
    let y = get_real(stream);
    Point2::new(x, y)
}

/// `IS >> gp_Dir2d`: two GetReal, then `SetCoord`, which raises
/// Standard_ConstructionError for a null vector.
pub(crate) fn read_direction2(stream: &mut IoStream) -> Result<Direction2, Failure> {
    let x = get_real(stream);
    let y = get_real(stream);
    let mut direction = Direction2::default();
    direction.set_coordinates(x, y)?;
    Ok(direction)
}

/// `IS >> int` into a variable the native code initialises to 0: a failed extraction
/// stores 0 (or the clamped bound) and a stream that already failed leaves it at 0.
pub(crate) fn read_int(stream: &mut IoStream) -> i32 {
    let mut value = 0;
    stream.extract_int(&mut value);
    value
}

/// `IS >> bool` into a variable initialised to false: only 0 and 1 are valid, any
/// other number reads as true and fails the stream.
pub(crate) fn read_flag(stream: &mut IoStream) -> bool {
    let mut value = false;
    stream.extract_bool(&mut value);
    value
}

/// `NCollection_Array1<T> array(1, upper)`: a negative extent is the allocation
/// failure the native size_t wrap-around produces (Standard_RangeError here).
pub(crate) fn array1<T: Default>(upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, upper).map_err(|error| error.into_failure("NCollection_Array1"))
}

/// `array(index) = value` inside the bounds the reader allocated.
pub(crate) fn store<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| error.into_failure("NCollection_Array1::ChangeValue"))
}

/// `IS >> rational; IS >> degree;` of a Bezier curve record: the flag and the pole
/// count `degree + 1`. The native `degree + 1` of an INT_MAX degree overflows into a
/// size no allocation meets; wrapping keeps that refusal instead of a panic.
pub(crate) fn read_bezier_header(stream: &mut IoStream) -> (bool, i32) {
    let rational = read_flag(stream);
    let degree = read_int(stream);
    (rational, degree.wrapping_add(1))
}

/// The counts of a B-spline curve record.
pub(crate) struct BSplineHeader {
    pub(crate) rational: bool,
    pub(crate) periodic: bool,
    pub(crate) degree: i32,
    pub(crate) nb_poles: i32,
    pub(crate) nb_knots: i32,
}

/// `IS >> rational >> periodic; IS >> degree >> nbpoles >> nbknots;`.
pub(crate) fn read_bspline_header(stream: &mut IoStream) -> BSplineHeader {
    let rational = read_flag(stream);
    let periodic = read_flag(stream);
    let degree = read_int(stream);
    let nb_poles = read_int(stream);
    BSplineHeader {
        rational,
        periodic,
        degree,
        nb_poles,
        nb_knots: read_int(stream),
    }
}

/// The pole loop of the curve readers: `count` poles, each followed by its weight
/// when `rational` (the weights stay 0 otherwise, and are not used).
pub(crate) fn read_poles<P: Default>(
    stream: &mut IoStream,
    count: i32,
    rational: bool,
    read_point: fn(&mut IoStream) -> P,
) -> Result<(Array1<'static, P>, Array1<'static, f64>), Failure> {
    let mut poles: Array1<'static, P> = array1(count)?;
    let mut weights: Array1<'static, f64> = array1(count)?;
    for index in 1..=count {
        store(&mut poles, index, read_point(stream))?;
        if rational {
            store(&mut weights, index, get_real(stream))?;
        }
    }
    Ok((poles, weights))
}

/// The knot loop of every B-spline reader: `count` pairs of `GetReal(knot)` and
/// `IS >> mult`; a multiplicity that cannot be read stays 0.
pub(crate) fn read_knots(
    stream: &mut IoStream,
    count: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut knots: Array1<'static, f64> = array1(count)?;
    let mut mults: Array1<'static, i32> = array1(count)?;
    for index in 1..=count {
        store(&mut knots, index, get_real(stream))?;
        store(&mut mults, index, read_int(stream))?;
    }
    Ok((knots, mults))
}
