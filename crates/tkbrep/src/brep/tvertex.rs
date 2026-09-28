// Created on: 1992-08-25
// Created by: Modelistation
// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_TVertex.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_TVertex`: the vertex TShape with a point, a tolerance and point representations.
use super::dump::{dump_base, dump_class_name, dump_field};
use super::point_representation::BRepPointRepresentation;
use crate::topods::{self, TShape, TShapeBase, dump_tshape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::gp::Point3;
use std::io;

/// The private fields of `BRep_TVertex`.
#[derive(Clone)]
struct TVertexFields {
    point: Point3,
    tolerance: f64,
    points: Vec<Handle<dyn BRepPointRepresentation>>,
}

/// `BRep_TVertex`: a vertex with its 3D point, its tolerance (`RealEpsilon()` when
/// new) and its parameters on curves and surfaces.
///
/// ```
/// use rustycad_tkbrep::brep::BRepTVertex;
/// let vertex = BRepTVertex::new();
/// vertex.update_tolerance(1e-3);
/// vertex.update_tolerance(1e-5);
/// assert_eq!(vertex.tolerance(), 1e-3);
/// ```
pub struct BRepTVertex {
    parent: topods::TVertex,
    fields: SharedState<TVertexFields>,
}
impl_transient!(BRepTVertex, "BRep_TVertex", topods::TVertex, parent, interfaces: [dyn TShape]);

impl BRepTVertex {
    /// `BRep_TVertex()`: the origin with the tolerance `RealEpsilon()`; `BRepTVertex::new()`.
    pub fn new() -> Self {
        Self {
            parent: topods::TVertex::new(),
            fields: SharedState::new(TVertexFields {
                point: Point3::default(),
                tolerance: real_epsilon(),
                points: Vec::new(),
            }),
        }
    }
    /// `Tolerance()`; `vertex.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.fields.read().tolerance
    }
    /// `Tolerance(T)`; `vertex.set_tolerance(1e-7)`.
    pub fn set_tolerance(&self, tolerance: f64) {
        self.fields.write().tolerance = tolerance;
    }
    /// `UpdateTolerance(T)`: keeps the larger tolerance; `vertex.update_tolerance(1e-7)`.
    pub fn update_tolerance(&self, tolerance: f64) {
        let mut fields = self.fields.write();
        if tolerance > fields.tolerance {
            fields.tolerance = tolerance;
        }
    }
    /// `Pnt()`; `vertex.pnt()`.
    pub fn pnt(&self) -> Point3 {
        self.fields.read().point
    }
    /// `Pnt(P)`; `vertex.set_pnt(point)`.
    pub fn set_pnt(&self, point: Point3) {
        self.fields.write().point = point;
    }
    /// `Points()`: a copy of the list of point representations; `vertex.points()`.
    pub fn points(&self) -> Vec<Handle<dyn BRepPointRepresentation>> {
        self.fields.read().points.clone()
    }
    /// `ChangePoints()`: lends the stored list to `edit`, which must not call this
    /// vertex; `vertex.change_points(|points| points.clear())`.
    pub fn change_points<R>(
        &self,
        edit: impl FnOnce(&mut Vec<Handle<dyn BRepPointRepresentation>>) -> R,
    ) -> R {
        edit(&mut self.fields.write().points)
    }
}

impl Default for BRepTVertex {
    fn default() -> Self {
        Self::new()
    }
}

impl TShape for BRepTVertex {
    fn tshape_base(&self) -> &TShapeBase {
        self.parent.tshape_base()
    }
    /// A new vertex with the same point and tolerance, without point representations.
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        let copy = Self::new();
        copy.set_pnt(self.pnt());
        copy.set_tolerance(self.tolerance());
        Ok(Handle::new(copy).into_cast())
    }
    /// The class name, the `TopoDS_TShape` dump, the point, the tolerance and each
    /// point representation.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) -> io::Result<()> {
        let address = std::ptr::from_ref(self).cast::<()>() as usize;
        dump_class_name(output, "BRep_TVertex")?;
        dump_base(output, depth, |nested, _| {
            dump_tshape(self.tshape_base(), address, nested)
        })?;
        let fields = self.fields.read().clone();
        dump_field(output, depth, "Pnt", |nested, _| {
            fields.point.dump_json(nested)
        })?;
        dump::dump_real(output, "Tolerance", fields.tolerance)?;
        for point in &fields.points {
            let Some(point) = point.get() else { continue };
            dump_field(output, depth, "PointRepresentation", |nested, d| {
                point.dump_json(nested, d)
            })?;
        }
        Ok(())
    }
}
