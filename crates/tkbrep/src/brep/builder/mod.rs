// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_Builder`: builds and updates the BRep shapes (docs/topology-design.md section 9).
//!
//! The members are split by the shape they build: faces (`face.rs`), edge curves
//! (`edge.rs`), edge polygons (`edge_polygons.rs`), edge flags and ranges
//! (`edge_flags.rs`) and vertices (`vertex.rs`); `update_curves.rs` holds the static
//! list helpers of BRep_Builder.cxx.
mod edge;
mod edge_flags;
mod edge_polygons;
mod face;
pub mod update_curves;
mod vertex;

use crate::topods::{self, Shape, TShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;
use std::ops::Deref;

/// `BRep_Builder`: the TopoDS builder plus the members that set geometry on faces,
/// edges and vertices. It derefs to [`topods::Builder`], as the native class derives
/// from `TopoDS_Builder`.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkmath::gp::Point3;
/// let mut vertex = Vertex::default();
/// BRepBuilder::default().make_vertex_at(&mut vertex, Point3::new(1.0, 2.0, 3.0), 1e-7).unwrap();
/// assert_eq!(BRepTool::pnt(&vertex).unwrap(), Point3::new(1.0, 2.0, 3.0));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepBuilder {
    topods: topods::Builder,
}

impl Deref for BRepBuilder {
    type Target = topods::Builder;
    fn deref(&self) -> &topods::Builder {
        &self.topods
    }
}

/// `if (TS->Locked()) throw TopoDS_LockedShape(member)`.
fn check_unlocked(tshape: &dyn TShape, member: &str) -> Result<(), Failure> {
    if tshape.locked() {
        return Err(topods::errors::locked_shape(&format!(
            "{member}: the TShape is locked; expected an unlocked shape"
        )));
    }
    Ok(())
}

/// `if (!S.IsNull() && S.Locked()) throw TopoDS_LockedShape(member)`: the check of the
/// `Make*` members, which replace the TShape of `shape`.
fn check_unlocked_shape(shape: &Shape, member: &str) -> Result<(), Failure> {
    let Some(tshape) = shape.tshape().get() else {
        return Ok(());
    };
    check_unlocked(tshape, member)
}

/// `L.Predivided(S.Location())`: a location relative to the shape's own location.
fn relative_to(location: &Location, shape: &Shape) -> Result<Location, Failure> {
    location.predivided(shape.location())
}
