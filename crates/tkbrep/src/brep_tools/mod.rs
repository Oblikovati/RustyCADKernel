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
// Rust translation of ModelingData/TKBRep/BRepTools at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepTools: tools on BRep shapes.
//!
//! The static members of the native `BRepTools` class are the free functions of this
//! module (`BRepTools::UVBounds(F, ...)` is [`uv_bounds`]; the overloads get names that
//! say what they take). The classes are [`WireExplorer`] (the edges of a wire in their
//! connection order) and the others listed in docs/tkbrep-progress.md.
mod clean;
mod copy_modification;
mod debug;
mod eval_tol;
mod gtrsf_modification;
mod history;
mod internals;
mod io;
mod modification;
mod modifier;
mod nurbs_convert;
mod purge_locations;
mod queries;
mod quilt;
mod reshape;
mod shape_set;
mod substitution;
mod triangulation;
mod trsf_modification;
mod update;
mod uv_bounds;
mod uv_periodicity;
mod wire_explorer;

pub use clean::{clean, clean_geometry, remove_unused_pcurves};
pub use copy_modification::CopyModification;
pub use debug::{debug_dump, debug_dump_location, debug_write};
pub use eval_tol::eval_and_update_tol;
pub use gtrsf_modification::GTrsfModification;
pub use history::{History, HistoryAlgorithm, RelationType};
pub use internals::{check_locations, remove_internals};
pub use io::{
    BRepWriteOptions, dump, read, read_file, read_file_on, read_with_console, write, write_file,
    write_file_on, write_file_with_options, write_with_options,
};
pub use modification::{BRepToolsModification, NewCurve, NewSurface};
pub use modifier::Modifier;
pub use nurbs_convert::NurbsConvertModification;
pub use purge_locations::PurgeLocations;
pub use queries::{
    compare_edges, compare_vertices, detect_closedness, is_really_closed, map_3d_edges,
    ori_edge_in_face, outer_wire,
};
pub use quilt::Quilt;
pub use reshape::{
    ApplyGuard, BRepToolsReShape, ReShape, ReShapeData, ReShapeFields, Replacement, ReplacementKind,
};
pub use shape_set::BRepToolsShapeSet;
pub use substitution::Substitution;
pub use triangulation::{
    activate_triangulation, load_all_triangulations, load_triangulation, triangulation,
    unload_all_triangulations, unload_triangulation,
};
pub use trsf_modification::TrsfModification;
pub use update::{
    update, update_comp_solid, update_compound, update_edge, update_face, update_face_uv_points,
    update_shell, update_solid, update_vertex, update_wire,
};
pub use uv_bounds::{
    UVBounds, add_uv_bounds, add_uv_bounds_of_edge, add_uv_bounds_of_wire, uv_bounds,
    uv_bounds_of_edge, uv_bounds_of_wire,
};
pub use wire_explorer::WireExplorer;
