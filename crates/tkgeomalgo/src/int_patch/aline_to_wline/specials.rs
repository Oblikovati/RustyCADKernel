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
// Rust translation of the static functions AddPointIntoLine, AddVertexPoint and
// IsPoleOrSeam of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ALineToWLine.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of the walking line: plain points, vertices and special points.
use super::super::IntPatchSpecPntType;
use super::super::point::IntPatchPoint;
use super::super::special_points::IntPatchSpecialPoints;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;

/// `AddPointIntoLine(theLine, theArrPeriods, thePoint, theVertex)`: appends the point
/// unless it is the last one; before, adjusts it (and the vertex) to the periods of the
/// last point.
pub(super) fn add_point_into_line(
    line: &IntSurfLineOn2S,
    arr_periods: &[f64; 4],
    point: &mut IntSurfPntOn2S,
    vertex: Option<&mut IntPatchPoint>,
) -> Result<(), Failure> {
    if line.nb_points() > 0 {
        let last: IntSurfPntOn2S = line.value(line.nb_points())?;
        if point.is_same(&last, precision::confusion(), -1.0) {
            return Ok(());
        }
        IntPatchSpecialPoints::adjust_point_and_vertex(&last, arr_periods, point, vertex);
    }
    line.add(point);
    Ok(())
}

/// `AddVertexPoint(theLine, theVertex, theArrPeriods)`: appends the point of the vertex.
pub(super) fn add_vertex_point(
    line: &IntSurfLineOn2S,
    vertex: &mut IntPatchPoint,
    arr_periods: &[f64; 4],
) -> Result<(), Failure> {
    let mut apex_point: IntSurfPntOn2S = vertex.pnt_on_2s();
    add_point_into_line(line, arr_periods, &mut apex_point, Some(vertex))
}

/// The surfaces and settings of `IsPoleOrSeam`.
pub(super) struct PoleOrSeamInput<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) arr_periods: &'a [f64; 4],
    pub(super) tol_3d: f64,
}

/// `IsPoleOrSeam(theS1, theS2, thePIsoRef, theLine, theVertex, theArrPeriods, theTol3d,
/// theSingularSurfaceID)`: adds the vertex to the line as a special point of the first
/// surface that has one (the pole of a sphere or cone; the crossing of the seams of a
/// torus; the seam of a cylinder, and of a sphere, cone or torus without the former).
/// Returns the kind of the point and the surface (0, 1 or 2) that holds it.
pub(super) fn is_pole_or_seam(
    input: &PoleOrSeamInput<'_>,
    p_iso_ref: &IntSurfPntOn2S,
    line: &IntSurfLineOn2S,
    vertex: &mut IntPatchPoint,
) -> Result<(IntPatchSpecPntType, i32), Failure> {
    for i in 0..2 {
        let is_reversed: bool = i > 0;
        let (q, p) = if is_reversed {
            (input.s2, input.s1)
        } else {
            (input.s1, input.s2)
        };
        let mut apex_point = IntSurfPntOn2S::new();
        let added = match q.get_type()? {
            SurfaceType::Sphere | SurfaceType::Cone => {
                let found = IntPatchSpecialPoints::add_singular_pole(
                    q, p, p_iso_ref, vertex, &mut apex_point, is_reversed, true,
                )?;
                found.then_some(IntPatchSpecPntType::SPntPole)
            }
            SurfaceType::Torus => {
                let tol: f64 = input.tol_3d;
                let found = IntPatchSpecialPoints::add_cross_uv_iso_point(
                    q, p, p_iso_ref, tol, &mut apex_point, is_reversed,
                )?;
                found.then_some(IntPatchSpecPntType::SPntSeamUV)
            }
            SurfaceType::Cylinder => None,
            _ => continue,
        };
        // A sphere, cone or torus without its special point falls through to the seam.
        let Some(added) = added else {
            add_vertex_point(line, vertex, input.arr_periods)?;
            return Ok((IntPatchSpecPntType::SPntSeamU, i + 1));
        };
        add_point_into_line(line, input.arr_periods, &mut apex_point, Some(vertex))?;
        return Ok((added, i + 1));
    }
    Ok((IntPatchSpecPntType::SPntNone, 0))
}
