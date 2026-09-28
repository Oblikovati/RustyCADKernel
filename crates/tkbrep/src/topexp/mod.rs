// Created on: 1990-12-20
// Created by: Remi Lequette
// Copyright (c) 1990-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/TopExp/TopExp.hxx and TopExp.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TopExp: exploration of the topological data structure.
//!
//! The static members of the `TopExp` class are the free functions of this module;
//! `TopExp_Explorer` is [`Explorer`]. `TopExp.cxx` defines `No_Standard_TypeMismatch`,
//! so its casts to vertices and edges never check the shape type (they are
//! [`crate::topods::Vertex::unchecked`]).
mod ancestors;
mod explorer;
mod vertices;

pub use ancestors::{map_shapes_and_ancestors, map_shapes_and_unique_ancestors};
pub use explorer::Explorer;
pub use vertices::{common_vertex, first_vertex, last_vertex, vertices, wire_vertices};

use crate::topods::{Shape, ShapeIterator};
use crate::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `MapShapes(S, T, M)`: adds to `map` every sub-shape of `shape` of type `kind`, in
/// exploration order; the map is not cleared first.
///
/// ```
/// use rustycad_tkbrep::topexp;
/// use rustycad_tkbrep::topods::{Builder, Compound};
/// use rustycad_tkbrep::toptools::IndexedMapOfShape;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let mut compound = Compound::default();
/// Builder.make_compound(&mut compound);
/// let mut map = IndexedMapOfShape::default();
/// topexp::map_shapes_of_type(&compound, ShapeEnum::Compound, &mut map).unwrap();
/// assert_eq!(map.extent(), 1);
/// ```
pub fn map_shapes_of_type(
    shape: &Shape,
    kind: ShapeEnum,
    map: &mut IndexedMapOfShape,
) -> Result<(), Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        map.add(explorer.current().clone());
        explorer.next()?;
    }
    Ok(())
}

/// `MapShapes(S, M, cumOri, cumLoc)` into an indexed map: `shape` then, depth first,
/// every sub-shape. Only the first level uses `cum_ori` and `cum_loc`: the recursion
/// calls the native defaults (both true); `topexp::map_shapes(&shape, &mut map, true, true)`.
pub fn map_shapes(shape: &Shape, map: &mut IndexedMapOfShape, cum_ori: bool, cum_loc: bool) {
    map.add(shape.clone());
    let mut iterator = ShapeIterator::new(shape, cum_ori, cum_loc);
    while let Ok(child) = iterator.value() {
        map_shapes(child, map, true, true);
        iterator.next();
    }
}

/// `MapShapes(S, M, cumOri, cumLoc)` into a plain map (iteration order follows the
/// hash, hence the TShape addresses); `topexp::map_shapes_into_map(&shape, &mut map, true, true)`.
pub fn map_shapes_into_map(shape: &Shape, map: &mut MapOfShape, cum_ori: bool, cum_loc: bool) {
    map.add(shape.clone());
    let mut iterator = ShapeIterator::new(shape, cum_ori, cum_loc);
    while let Ok(child) = iterator.value() {
        map_shapes_into_map(child, map, true, true);
        iterator.next();
    }
}
