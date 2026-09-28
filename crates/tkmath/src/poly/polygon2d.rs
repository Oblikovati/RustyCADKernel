// Created on: 1995-03-06
// Created by: Laurent PAINNOT
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/Poly/Poly_Polygon2D.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{
    errors::array_failure, owned::owned_array, shared_state::SharedState, triangulation::dump_size,
};
use crate::gp::Point2;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{
        Failure, Handle, StandardTransient,
        dump::{self, DumpOutput},
    },
};
use std::io;
struct Polygon2DFields {
    deflection: f64,
    nodes: Array1<'static, Point2>,
}
/// `Poly_Polygon2D`: one-based 2D nodes, the closing node repeated; `Polygon2D::new(4)?`.
pub struct Polygon2D {
    base: StandardTransient,
    fields: SharedState<Polygon2DFields>,
}
rustycad_tkernel::impl_transient!(Polygon2D, "Poly_Polygon2D", StandardTransient, base);
impl Polygon2D {
    /// Allocates `nb_nodes` origin nodes at `1..=nb_nodes`; `Polygon2D::new(4)?`.
    pub fn new(nb_nodes: i32) -> Result<Self, Failure> {
        let nodes = Array1::with_bounds(1, nb_nodes)
            .map_err(|e| array_failure("Poly_Polygon2D::Poly_Polygon2D", e))?;
        Ok(Self::from_fields(nodes))
    }
    /// Copies nodes of any bounds to `1..=len`; `Polygon2D::from_nodes(&nodes)`.
    pub fn from_nodes(nodes: &Array1<'_, Point2>) -> Self {
        Self::from_fields(owned_array(nodes.as_slice(), 1))
    }
    fn from_fields(nodes: Array1<'static, Point2>) -> Self {
        Self {
            base: StandardTransient::default(),
            fields: SharedState::new(Polygon2DFields {
                deflection: 0.0,
                nodes,
            }),
        }
    }
    /// A new handle with the same nodes and deflection; `polygon.copy()`.
    pub fn copy(&self) -> Handle<Self> {
        let fields = self.fields.read();
        let copy = Self::from_fields(owned_array(fields.nodes.as_slice(), 1));
        copy.set_deflection(fields.deflection);
        Handle::new(copy)
    }
    /// Returns the deflection; `polygon.deflection()`.
    pub fn deflection(&self) -> f64 {
        self.fields.read().deflection
    }
    /// Sets the deflection; `polygon.set_deflection(0.01)`.
    pub fn set_deflection(&self, deflection: f64) {
        self.fields.write().deflection = deflection;
    }
    /// Number of nodes, closing node included; `polygon.nb_nodes()`.
    pub fn nb_nodes(&self) -> i32 {
        self.fields.read().nodes.len() as i32
    }
    /// Copy of the node table, as `Nodes()`; `polygon.nodes()`.
    pub fn nodes(&self) -> Array1<'static, Point2> {
        self.fields.read().nodes.clone()
    }
    /// Lends the node table for editing, as `ChangeNodes()`; `polygon.change_nodes(|n| n.len())`.
    pub fn change_nodes<R>(&self, edit: impl FnOnce(&mut Array1<'static, Point2>) -> R) -> R {
        edit(&mut self.fields.write().nodes)
    }
    /// Writes the class name, deflection and node count; `polygon.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        let fields = self.fields.read();
        dump::dump_class_begin(output, "Poly_Polygon2D")?;
        dump::dump_real(output, "Deflection", fields.deflection)?;
        dump_size(output, "Nodes.Size", fields.nodes.len())
    }
}
