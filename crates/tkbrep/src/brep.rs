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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRep vertex construction and geometry extraction.
use crate::topods::Vertex;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
/// BRep_Builder's vertex construction entry point; `Builder::make_vertex(point,1e-7)`.
pub struct Builder;
impl Builder {
    /// Constructs independently owned geometry; `Builder::make_vertex(Point3::default(),1e-7)`.
    pub fn make_vertex(point: Point3, tolerance: f64) -> Vertex {
        Vertex::new(
            point,
            if tolerance > f64::EPSILON {
                tolerance
            } else {
                f64::EPSILON
            },
        )
    }
}
/// BRep_Tool geometry queries; `Tool::point(&vertex)?`.
pub struct Tool;
impl Tool {
    /// Applies the shape location to the underlying vertex point; `Tool::point(&vertex)?`.
    pub fn point(vertex: &Vertex) -> Result<Point3, Failure> {
        let geometry = vertex.shape().vertex.as_ref().ok_or_else(null_vertex)?;
        Ok(vertex
            .shape()
            .location()
            .transformation()
            .apply(geometry.point))
    }
    /// Reports the stored tolerance with native confusion floor; `Tool::tolerance(&vertex)?`.
    pub fn tolerance(vertex: &Vertex) -> Result<f64, Failure> {
        let geometry = vertex.shape().vertex.as_ref().ok_or_else(null_vertex)?;
        Ok(geometry.tolerance.max(1e-7))
    }
}
fn null_vertex() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRep_Tool:: TopoDS_Vertex hasn't gp_Pnt",
        "",
    )
}
