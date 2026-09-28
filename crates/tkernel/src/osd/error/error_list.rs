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

//! `OSD_ErrorList.hxx`: the portable `ERR_*` codes returned by `OSD_Error::Error()`.
//!
//! The values are the native macro values; `error_list::ERR_NOENT == 9`.
/// `ERR_SURPRISE`.
pub const ERR_SURPRISE: i32 = -2;
/// `ERR_UNKNOWN`.
pub const ERR_UNKNOWN: i32 = -1;
/// `ERR_NONE`.
pub const ERR_NONE: i32 = 0;
/// `ERR_ACCESS`.
pub const ERR_ACCESS: i32 = 1;
/// `ERR_EXIST`.
pub const ERR_EXIST: i32 = 2;
/// `ERR_FAULT`.
pub const ERR_FAULT: i32 = 3;
/// `ERR_INTR`.
pub const ERR_INTR: i32 = 4;
/// `ERR_INVAL`.
pub const ERR_INVAL: i32 = 5;
/// `ERR_IO`.
pub const ERR_IO: i32 = 6;
/// `ERR_ISDIR`.
pub const ERR_ISDIR: i32 = 7;
/// `ERR_NAMETOOLONG`.
pub const ERR_NAMETOOLONG: i32 = 8;
/// `ERR_NOENT`.
pub const ERR_NOENT: i32 = 9;
/// `ERR_NOMEM`.
pub const ERR_NOMEM: i32 = 10;
/// `ERR_NOTDIR`.
pub const ERR_NOTDIR: i32 = 11;
/// `ERR_PERM`.
pub const ERR_PERM: i32 = 12;
/// `ERR_QUOT`.
pub const ERR_QUOT: i32 = 13;
/// `ERR_RANGE`.
pub const ERR_RANGE: i32 = 14;
/// `ERR_ROFS`.
pub const ERR_ROFS: i32 = 15;
/// `ERR_TOOBIG`.
pub const ERR_TOOBIG: i32 = 16;
/// `ERR_DMLINK`.
pub const ERR_DMLINK: i32 = 17;
/// `ERR_DNOENT`.
pub const ERR_DNOENT: i32 = 18;
/// `ERR_FAGAIN`.
pub const ERR_FAGAIN: i32 = 19;
/// `ERR_FBADF`.
pub const ERR_FBADF: i32 = 20;
/// `ERR_FBADMSG`.
pub const ERR_FBADMSG: i32 = 21;
/// `ERR_FDEADLK`.
pub const ERR_FDEADLK: i32 = 22;
/// `ERR_FEXIST`.
pub const ERR_FEXIST: i32 = 23;
/// `ERR_FFBIG`.
pub const ERR_FFBIG: i32 = 24;
/// `ERR_FINVAL`.
pub const ERR_FINVAL: i32 = 25;
/// `ERR_FIO`.
pub const ERR_FIO: i32 = 26;
/// `ERR_FLOCKED`.
pub const ERR_FLOCKED: i32 = 27;
/// `ERR_FMFILE`.
pub const ERR_FMFILE: i32 = 28;
/// `ERR_FNOLCK`.
pub const ERR_FNOLCK: i32 = 29;
/// `ERR_FPERM`.
pub const ERR_FPERM: i32 = 30;
/// `ERR_FRANGE`.
pub const ERR_FRANGE: i32 = 31;
/// `ERR_FWFD`.
pub const ERR_FWFD: i32 = 32;
/// `ERR_FNBUSY`.
pub const ERR_FNBUSY: i32 = 33;
/// `ERR_FNFILE`.
pub const ERR_FNFILE: i32 = 34;
/// `ERR_FNINVAL`.
pub const ERR_FNINVAL: i32 = 35;
/// `ERR_FNOSPC`.
pub const ERR_FNOSPC: i32 = 36;
/// `ERR_FNNOTEMPTY`.
pub const ERR_FNNOTEMPTY: i32 = 37;
/// `ERR_FNXDEV`.
pub const ERR_FNXDEV: i32 = 38;
/// `ERR_PPERM`.
pub const ERR_PPERM: i32 = 50;
