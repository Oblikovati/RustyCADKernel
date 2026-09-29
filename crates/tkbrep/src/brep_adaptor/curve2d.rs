// Created on: 1993-07-13
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
// Rust translation of ModelingData/TKBRep/BRepAdaptor/BRepAdaptor_Curve2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepAdaptor_Curve2d`: the p-curve of an edge on a face seen as a 2D curve.
use super::forward::inherited_curve2d_members;
use crate::brep::BRepTool;
use crate::topods::{Edge, Face};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

/// The edge and the face of the adaptor.
#[derive(Clone, Default)]
struct EdgeOnFace {
    edge: Edge,
    face: Face,
}

/// The p-curve of an edge on a face seen as a 2D curve (`BRepAdaptor_Curve2d`).
///
/// The native class derives from `Geom2dAdaptor_Curve`; the Rust type embeds it as its
/// parent field and forwards every inherited member.
///
/// ```
/// use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
/// use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
/// let adaptor = BRepAdaptorCurve2d::default();
/// assert!(adaptor.edge().is_null());
/// ```
#[derive(Default)]
pub struct BRepAdaptorCurve2d {
    parent: Geom2dAdaptorCurve,
    shapes: SharedState<EdgeOnFace>,
}
impl_transient!(
    BRepAdaptorCurve2d,
    "BRepAdaptor_Curve2d",
    Geom2dAdaptorCurve,
    parent,
    interfaces: [dyn Adaptor2dCurve2d]
);

impl BRepAdaptorCurve2d {
    /// `BRepAdaptor_Curve2d(E, F)`; `BRepAdaptorCurve2d::with_edge_on_face(&edge, &face)?`.
    pub fn with_edge_on_face(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.initialize(edge, face)?;
        Ok(adaptor)
    }

    /// The embedded parent adaptor (the `Geom2dAdaptor_Curve` base).
    pub fn geom2d_adaptor(&self) -> &Geom2dAdaptorCurve {
        &self.parent
    }

    /// `Initialize(E, F)`: stores the edge and the face, then loads the p-curve of the
    /// edge on the face on its range when there is one (a missing p-curve leaves the
    /// previous curve loaded); `adaptor.initialize(&edge, &face)?`.
    pub fn initialize(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        *self.shapes.write() = EdgeOnFace {
            edge: edge.clone(),
            face: face.clone(),
        };
        let found = BRepTool::curve_on_face(edge, face)?;
        if !found.pcurve.is_null() {
            self.parent
                .load_range(&found.pcurve, found.first, found.last)?;
        }
        Ok(())
    }

    /// `Edge()`; `adaptor.edge()`.
    pub fn edge(&self) -> Edge {
        self.shapes.read().edge.clone()
    }

    /// `Face()`; `adaptor.face()`.
    pub fn face(&self) -> Face {
        self.shapes.read().face.clone()
    }
}

impl Adaptor2dCurve2d for BRepAdaptorCurve2d {
    /// `ShallowCopy()`: the curve, its kind and range copied (the caches rebuilt on
    /// demand), with the edge and the face.
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let copy = Self {
            parent: self.parent.shallow_adaptor()?,
            shapes: SharedState::new(self.shapes.read().clone()),
        };
        Ok(Handle::new(copy).cast())
    }

    inherited_curve2d_members!();
}
