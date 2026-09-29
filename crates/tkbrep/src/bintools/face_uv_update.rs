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
// Rust translation of BRepTools::Update(const TopoDS_Face&) and
// BRepTools::UpdateFaceUVPoints (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TEMPORARY: the one BRepTools member the binary reader calls, for version 1 files.
//! BRepTools is not ported yet; when it is, `BinShapeSet::read_flags_and_subs` calls
//! its `update_face` and this module is deleted (docs/tkbrep-progress.md, "BinTools").
use crate::brep::BRepTool;
use crate::brep::access::edge_tshape;
use crate::brep::update_curves::as_gcurve;
use crate::topexp::Explorer;
use crate::topods::{Face, TShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `BRepTools::Update(const TopoDS_Face& F)`: an unchecked face gets the UV points of
/// its edges reset, then its TShape is marked Checked.
pub(crate) fn update_face(face: &Face) -> Result<(), Failure> {
    if face.checked()? {
        return Ok(());
    }
    update_face_uv_points(face)?;
    face.set_checked(true)
}

/// `BRepTools::UpdateFaceUVPoints(F)`: for each edge of the face, the first parametric
/// representation on the face's located surface recomputes its UV points; a locked
/// edge stops the whole update.
fn update_face_uv_points(face: &Face) -> Result<(), Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let mut edges = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while edges.more() {
        let edge = edges.current().clone();
        let tedge = edge_tshape(&edge, "BRepTools::UpdateFaceUVPoints")?;
        if tedge.locked() {
            return Ok(());
        }
        let edge_location = location.predivided(edge.location())?;
        for representation in tedge.curves() {
            let gcurve = as_gcurve(&representation);
            let Some(gcurve) = gcurve.get() else { continue };
            if gcurve.is_curve_on_surface_at(&surface, &edge_location) {
                gcurve.update()?;
                break;
            }
        }
        edges.next()?;
    }
    Ok(())
}
