// Created on: 2004-05-11
// Created by: Sergey ZARITCHNY <szy@opencascade.com>
// Copyright (c) 2004-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BinTools at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BinTools: the binary BRep formats.
//!
//! Two formats share the scalar encodings of the static `BinTools` members:
//! - the grouped format of [`BinShapeSet`] (what `BinTools::Write` and `BinTools::Read`
//!   use): a version line, then the `Locations`, `Curve2ds`, `Curves`, `Polygon3D`,
//!   `PolygonOnTriangulations`, `Surfaces`, `Triangulations` and `TShapes` sections,
//!   whose records refer to each other by index;
//! - the shape-reference format of [`BinShapeWriter`] and [`BinShapeReader`], where each
//!   object is written where it is first used and later uses refer back to its byte
//!   position (through [`BinOStream`] and [`BinIStream`]).
//!
//! The native `Standard_OStream`/`Standard_IStream` are the TKernel iostream model
//! [`IoStream`](rustycad_tkernel::storage::IoStream); the text native code writes to
//! `std::cout` goes to an injected GeomTools
//! [`ConsoleOutput`](rustycad_tkgeombase::geom_tools::ConsoleOutput). Values are written
//! in the platform byte order: `DO_INVERSE` is not defined at the pinned revision.
//! docs/tkbrep-progress.md lists the native quirks the port keeps.
mod curve2d_read;
mod curve2d_set;
mod curve2d_write;
mod curve_read;
mod curve_set;
mod curve_write;
mod errors;
mod face_uv_update;
pub mod format_version;
mod geometry_in;
mod handle_table;
mod istream;
mod location_set;
mod object_type;
mod ostream;
mod raw_enums;
mod scalars;
mod shape_io;
mod shape_set;
mod shape_set_add;
mod shape_set_base;
mod shape_set_mesh_read;
mod shape_set_mesh_write;
mod shape_set_read;
mod shape_set_read_edge;
mod shape_set_read_shape;
mod shape_set_write;
mod shape_set_write_shape;
mod stream_buffer_device;
mod surface_read;
mod surface_set;
mod surface_write;
mod table_io;

pub use curve2d_set::BinCurve2dSet;
pub use curve_set::BinCurveSet;
pub use istream::BinIStream;
pub use location_set::{BinLocationSet, put_transform};
pub use object_type::ObjectType;
pub use ostream::BinOStream;
pub use scalars::{
    get_bool, get_ext_char, get_integer, get_real, get_short_real, put_bool, put_ext_char,
    put_integer, put_real, put_short_real,
};
pub use shape_io::{
    WriteOptions, read, read_file, read_file_on, read_with_console, write, write_file,
    write_file_on,
};
pub use shape_set::BinShapeSet;
pub use shape_set_base::{BinShapeSetBase, ShapeSetBaseData, THE_ASCII_VERSIONS, put_point};
pub use surface_set::BinSurfaceSet;
