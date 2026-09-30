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
// Rust translation of the IntPatch_PrmPrmIntersection::Perform overloads from given start
// points (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Perform` overloads from given start points: a list of points (with their
//! images across the closed directions of the surfaces), or one point.
use super::IntPatchPrmPrmIntersection;
use super::helpers::{surface_periods, widen_bounds};
use super::perform_poly::IntPatchPrmPrmTolerances;
use super::start_params;
use super::walk_steps::{
    AcceptOptions, EndsCheck, WalkInputs, add_end_vertices, duplicated_line, is_new_start,
    keep_rejected, new_wline, put_vertices, transitions,
};
use crate::int_start::TheTopolTool;
use crate::int_surf::IntSurfPntOn2S;
use crate::int_walk::IntWalkPWalking;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// The bounds of a surface in one direction and whether it is closed there.
struct ClosedSide {
    closed: bool,
    first: f64,
    last: f64,
}

/// The four sides `U1, V1, U2, V2` of the surfaces.
fn closed_sides(
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
) -> Result<[ClosedSide; 4], Failure> {
    let side = |closed: bool, first: f64, last: f64| ClosedSide {
        closed,
        first,
        last,
    };
    Ok([
        side(
            s1.is_u_closed()?,
            s1.first_u_parameter()?,
            s1.last_u_parameter()?,
        ),
        side(
            s1.is_v_closed()?,
            s1.first_v_parameter()?,
            s1.last_v_parameter()?,
        ),
        side(
            s2.is_u_closed()?,
            s2.first_u_parameter()?,
            s2.last_u_parameter()?,
        ),
        side(
            s2.is_v_closed()?,
            s2.first_v_parameter()?,
            s2.last_v_parameter()?,
        ),
    ])
}

/// The images of the points on a bound of a closed direction on the opposite bound
/// (the 3d point of an image is the origin, as natively), without the images that
/// match a given point.
fn additional_points(points: &[IntSurfPntOn2S], sides: &[ClosedSide; 4]) -> Vec<IntSurfPntOn2S> {
    let tol_par: f64 = precision::p_confusion();
    let mut additional: Vec<IntSurfPntOn2S> = Vec::new();
    for pnt in points {
        let (a, b, c, d) = pnt.parameters();
        let uv: [f64; 4] = [a, b, c, d];
        // One point is edited for all the images of a given point, as natively.
        let mut new_pnt = IntSurfPntOn2S::new();
        for (k, side) in sides.iter().enumerate() {
            if !side.closed {
                continue;
            }
            let image: f64 = if (uv[k] - side.first).abs() <= tol_par {
                side.last
            } else if (uv[k] - side.last).abs() <= tol_par {
                side.first
            } else {
                continue;
            };
            let mut params: [f64; 4] = uv;
            params[k] = image;
            new_pnt.set_parameters(params[0], params[1], params[2], params[3]);
            additional.push(new_pnt);
        }
    }
    // Cut repeated points.
    for pnt in points {
        let (u1, v1, u2, v2) = pnt.parameters();
        additional.retain(|p| {
            let (nu1, nv1, nu2, nv2) = p.parameters();
            !((u1 - nu1).abs() <= tol_par
                && (v1 - nv1).abs() <= tol_par
                && (u2 - nu2).abs() <= tol_par
                && (v2 - nv2).abs() <= tol_par)
        });
    }
    additional
}

impl IntPatchPrmPrmIntersection {
    /// `Perform(Caro1, Domain1, Caro2, Domain2, TolTangency, Epsilon, Deflection,
    /// Increment, ListOfPnts)`: the intersection lines through the given points, each
    /// walking extended to the boundaries and joined to the vertices of the lines found.
    /// On closed surfaces the images of the points on a seam or a bound are appended to
    /// `list_of_pnts`. An empty list only sets the intersection done.
    ///
    /// ```ignore
    /// inter.perform_with_points((&s1, &*d1), (&s2, &*d2), tolerances, &mut points)?;
    /// ```
    pub fn perform_with_points(
        &mut self,
        (surf1, d1): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        (surf2, d2): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        tolerances: IntPatchPrmPrmTolerances,
        list_of_pnts: &mut Vec<IntSurfPntOn2S>,
    ) -> Result<(), Failure> {
        if list_of_pnts.is_empty() {
            self.done = true;
            return Ok(());
        }
        self.empt = true;
        self.slin.clear();
        let sides: [ClosedSide; 4] = closed_sides(surf1, surf2)?;
        let mut min: [f64; 4] = [0, 1, 2, 3].map(|k| sides[k].first);
        let mut max: [f64; 4] = [0, 1, 2, 3].map(|k| sides[k].last);
        let periods: [f64; 4] = surface_periods(surf1, surf2)?;
        if sides.iter().any(|side| side.closed) {
            let additional = additional_points(list_of_pnts, &sides);
            list_of_pnts.extend(additional);
        }
        for pnt in list_of_pnts.iter() {
            let (a, b, c, d) = pnt.parameters();
            widen_bounds((&mut min, &mut max), [a, b, c, d]);
        }
        let t = tolerances;
        let inputs = WalkInputs {
            surf1,
            d1,
            surf2,
            d2: Some(d2),
            tol_tangency: t.tol_tangency,
            epsilon: t.epsilon,
            deflection: t.deflection,
            increment: t.increment,
        };
        let mut pw = IntWalkPWalking::new(
            surf1,
            surf2,
            t.tol_tangency,
            t.epsilon,
            t.deflection,
            t.increment,
        )?;
        for pnt in list_of_pnts.iter() {
            self.walk_list_point(&inputs, &mut pw, pnt, ((min, max), periods))?;
        }
        self.done = true;
        Ok(())
    }

    /// The walking from one point of the list.
    fn walk_list_point(
        &mut self,
        inputs: &WalkInputs<'_>,
        pw: &mut IntWalkPWalking,
        pnt: &IntSurfPntOn2S,
        ((min, max), periods): (([f64; 4], [f64; 4]), [f64; 4]),
    ) -> Result<(), Failure> {
        let (u1, v1, u2, v2) = pnt.parameters();
        let start = start_params([u1, v1, u2, v2])?;
        let Some(start_pon2s) = pw.perform_first_point(&start)? else {
            return Ok(());
        };
        let nb_lig_calculee: i32 = self.slin.len() as i32;
        if !is_new_start(inputs, &start_pon2s, &self.slin)? {
            return Ok(());
        }
        pw.perform_bounded(&start, min, max)?;
        if !pw.is_done() || pw.nb_points()? <= 2 {
            return Ok(());
        }
        // Try to extend the intersection line to boundary, if it is possible.
        pw.put_to_boundary(inputs.surf1, inputs.surf2)?;
        let ends = (pw.value(1)?.value(), pw.value(pw.nb_points()?)?.value());
        let duplicated = duplicated_line(
            inputs,
            pw,
            &self.slin,
            (nb_lig_calculee, true, EndsCheck::Both),
        )?;
        if keep_rejected(pw, duplicated, &mut self.slin)? {
            return Ok(());
        }
        let options = AcceptOptions {
            loose: true,
            purging: None,
            several: Some(periods),
        };
        self.accept_walking(inputs, pw, &options, ends)?;
        Ok(())
    }

    /// `Perform(Caro1, Domain1, Caro2, Domain2, U1, V1, U2, V2, TolTangency, Epsilon,
    /// Deflection, Increment)`: the intersection line through the point of parameters
    /// `(u1, v1, u2, v2)`, when the walking finds one.
    ///
    /// ```ignore
    /// inter.perform_from_start((&s1, &*d1), (&s2, &*d2), [0.5, 0.5, 0.5, 0.5], tolerances)?;
    /// ```
    pub fn perform_from_start(
        &mut self,
        (surf1, d1): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        (surf2, d2): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        depart: [f64; 4],
        tolerances: IntPatchPrmPrmTolerances,
    ) -> Result<(), Failure> {
        // Traitement des Lignes de sections.
        self.empt = true;
        self.done = true;
        self.slin.clear();
        let t = tolerances;
        let mut pw = IntWalkPWalking::new(
            surf1,
            surf2,
            t.tol_tangency,
            t.epsilon,
            t.deflection,
            t.increment,
        )?;
        let start = start_params(depart)?;
        // The first point of the marching, from the approximate point.
        if pw.perform_first_point(&start)?.is_none() {
            return Ok(());
        }
        pw.perform(&start)?;
        if !pw.is_done() || pw.nb_points()? <= 2 {
            return Ok(());
        }
        let inputs = WalkInputs {
            surf1,
            d1,
            surf2,
            d2: Some(d2),
            tol_tangency: t.tol_tangency,
            epsilon: t.epsilon,
            deflection: t.deflection,
            increment: t.increment,
        };
        let ends = (pw.value(1)?.value(), pw.value(pw.nb_points()?)?.value());
        let wline = new_wline(&pw, transitions(&inputs, &pw, false)?)?;
        // The method PutVertexOnLine can reduce the number of points in <wline>.
        if !put_vertices(&inputs, &wline, t.tol_tangency)? {
            return Ok(());
        }
        add_end_vertices(&pw, &wline, ends, t.tol_tangency)?;
        self.slin.push(wline.cast());
        self.empt = false;
        Ok(())
    }
}
