// Created on: 1993-07-23
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
// Rust translation of the edge copy and the free-end update of
// BRepLib_MakeWire::Add(E, IsCheckGeometryProximity) (BRepLib_MakeWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The copy of an edge onto the wire vertices it touches (the merged vertex moves to
//! a weighted point and grows its tolerance), and the update of the free ends of the
//! wire after an Add.
use super::BRepLibMakeWire;
use super::add_edge::children;
use crate::brep_lib::errors::BRepLibWireError;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// The merged vertex of a wire vertex `vw` and an edge vertex `ve` at distance `l`:
/// the point weighted towards the smaller tolerance and the tolerance that covers both.
fn merged_point(pw: Point3, pe: Point3, (tol_w, tol_e, l): (f64, f64, f64)) -> (Point3, f64) {
    let mut maxtol: f64 = 0.5 * (tol_w + tol_e + l);
    let (cw, ce): (f64, f64) = if maxtol > tol_w && maxtol > tol_e {
        let cw: f64 = (maxtol - tol_e) / l;
        (cw, 1. - cw)
    } else if maxtol > tol_w {
        maxtol = tol_e;
        (0., 1.)
    } else {
        maxtol = tol_w;
        (1., 0.)
    };
    let point = Point3::new(
        cw * pw.x() + ce * pe.x(),
        cw * pw.y() + ce * pe.y(),
        cw * pw.z() + ce * pe.z(),
    );
    (point, maxtol)
}

impl BRepLibMakeWire {
    /// "copy the edge": an empty copy of `ee` whose vertices close to a wire vertex are
    /// replaced by that vertex, moved and enlarged (`myVertex` is the last one, with the
    /// orientation of the edge vertex); the other vertices join the wire vertices. The
    /// wire vertices include those added by earlier vertices of the same edge.
    pub(super) fn copy_edge(&mut self, ee: &Edge) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        self.my_edge = Edge::unchecked(ee.empty_copied()?);
        for ve in children(ee)? {
            if let Some(vw) = self.merge_vertex(&ve)? {
                self.my_vertex = vw;
                self.my_vertex.set_orientation(ve.orientation());
                builder.add(&mut self.my_edge, &self.my_vertex)?;
                builder.transfert_vertex(ee, &self.my_edge, &ve, &self.my_vertex)?;
                continue;
            }
            self.my_vertices.add(ve.clone().into_shape());
            builder.add(&mut self.my_edge, &ve)?;
            builder.transfert_vertex(ee, &self.my_edge, &ve, &ve)?;
        }
        Ok(())
    }

    /// The first wire vertex within the tolerance of `ve` or its own, updated to the
    /// merged point and tolerance; None when there is none.
    fn merge_vertex(&self, ve: &Vertex) -> Result<Option<Vertex>, Failure> {
        let pe: Point3 = BRepTool::pnt(ve)?;
        for index in 1..=self.my_vertices.extent() {
            let vw: Vertex = topods::vertex(self.my_vertices.find_key(index)?)?;
            let pw: Point3 = BRepTool::pnt(&vw)?;
            let l: f64 = pe.distance(&pw);
            let tol_w: f64 = BRepTool::vertex_tolerance(&vw)?;
            let tol_e: f64 = BRepTool::vertex_tolerance(ve)?;
            if l < tol_e || l < tol_w {
                let (point, maxtol) = merged_point(pw, pe, (tol_w, tol_e, l));
                BRepBuilder::default().update_vertex_point(&vw, point, maxtol)?;
                return Ok(Some(vw));
            }
        }
        Ok(None)
    }

    /// Initialize VF, VL after a later edge: while the wire is manifold, the free end
    /// at the connecting vertex moves to the other vertex of the edge; a non-manifold
    /// wire forgets its free ends.
    pub(super) fn update_ends(&mut self) -> Result<(), Failure> {
        if self.my_error == BRepLibWireError::WireDone {
            // Update only
            let (v1, v2) = topexp::vertices(&self.my_edge, false);
            let vref: Vertex = if v1.is_same(&self.my_vertex) {
                v2
            } else if v2.is_same(&self.my_vertex) {
                v1
            } else {
                self.my_error = BRepLibWireError::NonManifoldWire;
                Vertex::default()
            };
            // Particular case VF == VL: the orientation is kept as it is.
            if !self.vf.is_same(&self.vl) {
                self.move_free_end(vref);
            }
        }
        if self.my_error == BRepLibWireError::NonManifoldWire {
            // nullify
            self.vf = Vertex::default();
            self.vl = Vertex::default();
        }
        Ok(())
    }

    /// The general case of `update_ends`: the free end at `myVertex` becomes `vref`.
    fn move_free_end(&mut self, vref: Vertex) {
        if self.vf.is_same(&self.my_vertex) {
            self.vf = vref;
        } else if self.vl.is_same(&self.my_vertex) {
            self.vl = vref;
        } else {
            self.my_error = BRepLibWireError::NonManifoldWire;
        }
    }
}
