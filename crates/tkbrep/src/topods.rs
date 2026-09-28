// Created on: 1990-12-11
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
// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_Shape.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Shared vertex geometry and independent per-shape locations.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{gp::Point3, toploc::Location};
use std::sync::Arc;
#[derive(Debug)]
pub(crate) struct VertexGeometry {
    pub point: Point3,
    pub tolerance: f64,
}
/// TopoDS shape ownership and location, initially null; `Shape::default()`.
#[derive(Clone, Debug, Default)]
pub struct Shape {
    pub(crate) vertex: Option<Arc<VertexGeometry>>,
    location: Location,
}
impl Shape {
    pub(crate) fn from_vertex(point: Point3, tolerance: f64) -> Self {
        Self {
            vertex: Some(Arc::new(VertexGeometry { point, tolerance })),
            location: Location::default(),
        }
    }
    /// Tests whether geometry is absent; `Shape::default().is_null()`.
    pub fn is_null(&self) -> bool {
        self.vertex.is_none()
    }
    /// Borrows the per-shape coordinate system; `shape.location()`.
    pub const fn location(&self) -> &Location {
        &self.location
    }
    /// Copies the shape and prepends a location without copying geometry; `shape.moved(&location)`.
    pub fn moved(&self, position: &Location) -> Result<Self, Failure> {
        Ok(Self {
            vertex: self.vertex.clone(),
            location: position.multiplied(&self.location)?,
        })
    }
    /// Tests shared topological identity regardless of location; `shape.is_partner(&other)`.
    pub fn is_partner(&self, other: &Self) -> bool {
        match (&self.vertex, &other.vertex) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}
/// Vertex-typed shape; `Vertex::default()` is null like TopoDS_Vertex.
#[derive(Clone, Debug, Default)]
pub struct Vertex {
    shape: Shape,
}
impl Vertex {
    pub(crate) fn new(point: Point3, tolerance: f64) -> Self {
        Self {
            shape: Shape::from_vertex(point, tolerance),
        }
    }
    /// Borrows the underlying located shape; `vertex.shape()`.
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
    /// Converts a vertex or null shape; `Vertex::from_shape(&shape)`.
    pub fn from_shape(shape: &Shape) -> Self {
        Self {
            shape: shape.clone(),
        }
    }
}
