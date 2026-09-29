// Created on: 1996-07-12
// Created by: Stagiaire Mary FABIEN
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of BRepTools_NurbsConvertModification::NewCurve2d
// (ModelingData/TKBRep/BRepTools/BRepTools_NurbsConvertModification.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_NurbsConvertModification::NewCurve2d`: the p-curve on the converted
//! surface, projected or converted.
mod seam;

use super::NurbsConvertModification;
use super::helpers::is_convert;
use crate::brep::BRepTool;
use crate::brep_tools::{eval_and_update_tol, is_really_closed};
use crate::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::geom2d_convert;
use rustycad_tkgeombase::proj_lib::{ComputeApprox, ComputeApproxOnPolarSurface};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::Vector2;

/// The edge and face being converted and the ones being built.
pub(super) type EdgeOnFace<'a> = (&'a Edge, &'a Face);

/// The 3D curve of an edge with its range.
pub(super) struct RangedCurve {
    pub(super) curve: Handle<dyn GeomCurve>,
    pub(super) range: (f64, f64),
}

/// Whether a 2D curve is exactly a Geom2d B-spline or Bezier curve.
pub(super) fn is_spline_2d(curve: &Handle<dyn Geom2dCurve>) -> bool {
    curve.get().is_some_and(|c| {
        matches!(
            c.dynamic_type().name(),
            "Geom2d_BSplineCurve" | "Geom2d_BezierCurve"
        )
    })
}

/// `Geom2dConvert::CurveToBSplineCurve(C)` (TgtThetaOver2).
pub(super) fn to_bspline_2d(
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(geom2d_convert::curve_to_bspline_curve(curve, ParameterisationType::TgtThetaOver2)?.cast())
}

/// `new Geom2d_TrimmedCurve(C, f, l)`.
pub(super) fn trimmed_2d(
    curve: &Handle<dyn Geom2dCurve>,
    (f, l): (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(Handle::new(Geom2dTrimmedCurve::new(curve, f, l, true, true)?).cast())
}

/// `new Geom2dAdaptor_Curve(C, f, l)` as an adaptor handle.
pub(super) fn adaptor_2d(
    curve: &Handle<dyn Geom2dCurve>,
    (f, l): (f64, f64),
) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    Ok(Handle::new(Geom2dAdaptorCurve::with_range(curve, f, l)?).cast())
}

/// `new GeomAdaptor_Curve(C, f, l)` as an adaptor handle.
pub(super) fn adaptor_3d(curve: &RangedCurve) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let (f, l) = curve.range;
    Ok(Handle::new(GeomAdaptorCurve::with_range(&curve.curve, f, l)?).cast())
}

/// `S->Bounds(...)` widened by 10% on each side (`u`, `v`), the widening reduced for a
/// periodic direction so that the domain spans at most one period: `(Uinf - u, Usup + u,
/// Vinf - v, Vsup + v)`.
pub(super) fn widened_domain(surface: &dyn GeomSurface) -> Result<(f64, f64, f64, f64), Failure> {
    let (u_inf, u_sup, v_inf, v_sup) = surface.bounds();
    let mut u = (u_sup - u_inf) * 0.1;
    let mut v = (v_sup - v_inf) * 0.1;
    if surface.is_u_periodic() {
        u = periodic_margin(surface.u_period()?, (u_inf, u_sup), u);
    }
    if surface.is_v_periodic() {
        v = periodic_margin(surface.v_period()?, (v_inf, v_sup), v);
    }
    Ok((u_inf - u, u_sup + u, v_inf - v, v_sup + v))
}

fn periodic_margin(period: f64, (inf, sup): (f64, f64), margin: f64) -> f64 {
    if period < sup + 2.0 * margin - inf {
        if period <= sup - inf {
            return 0.0;
        }
        return (period - (sup - inf)) * 0.5;
    }
    margin
}

/// `new GeomAdaptor_Surface(S, U1, U2, V1, V2)` on the widened domain.
pub(super) fn widened_adaptor(
    surface: &Handle<dyn GeomSurface>,
) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    let Some(stored) = surface.get() else {
        return Err(crate::brep::errors::null_object(
            "BRepTools_NurbsConvertModification::NewCurve2d: null surface",
        ));
    };
    let (u1, u2, v1, v2) = widened_domain(stored)?;
    Ok(Handle::new(GeomAdaptorSurface::with_bounds(surface, (u1, u2, v1, v2))?).cast())
}

impl NurbsConvertModification {
    /// `NewCurve2d(E, F, newE, newF, Curve2d, Tol)`: a degenerated edge gets its p-curve
    /// converted; a seam of the face is projected with its two p-curves (see the `seam`
    /// module); any other p-curve is projected on the new (or original) surface, as
    /// described in `project_pcurve`. Each result is recorded with its original and may
    /// raise the edge tolerance through `BRepTools::EvalAndUpdateTol`.
    pub(super) fn converted_curve2d(
        &self,
        (edge, face): EdgeOnFace<'_>,
        (new_edge, new_face): EdgeOnFace<'_>,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *tolerance = BRepTool::edge_tolerance(edge)?;
        let found = BRepTool::curve_on_face(edge, face)?;
        let base = found.pcurve.clone();
        let c3d = BRepTool::curve_with_location(edge)?;
        let spline_3d = c3d.curve.is_kind_name("Geom_BSplineCurve")
            || c3d.curve.is_kind_name("Geom_BezierCurve");
        let is_convert_2d = (!c3d.curve.is_null() && !spline_3d) || is_convert(edge)?;
        let raw_3d = RangedCurve {
            curve: c3d.curve,
            range: (c3d.first, c3d.last),
        };
        let range = (found.first, found.last);
        if BRepTool::degenerated(edge)? {
            let mut c2d = base.clone();
            if !c2d.is_kind_name("Geom2d_TrimmedCurve") {
                c2d = trimmed_2d(&c2d, range)?;
            }
            *curve = to_bspline_2d(&c2d)?;
            self.record(base.cast(), curve.cast());
            return Ok(true);
        }
        let context = Curve2dContext {
            edge,
            face,
            new_edge,
            new_face,
            base,
            range,
            is_convert_2d,
            raw_3d,
        };
        if !is_really_closed(edge, face)? {
            return self.project_pcurve(&context, curve, tolerance);
        }
        self.project_seam(&context, curve, tolerance)
    }

    /// The EvalAndUpdateTol of a result and its record.
    pub(super) fn accept(
        &self,
        context: &Curve2dContext<'_>,
        (c3d, surface): (&RangedCurve, &Handle<dyn GeomSurface>),
        curve: &Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let (f3d, l3d) = c3d.range;
        let new_tolerance =
            eval_and_update_tol(context.new_edge, &c3d.curve, curve, surface, f3d, l3d)?;
        self.raise_tolerance(context.new_edge, new_tolerance, tolerance);
        self.record(context.base.cast(), curve.cast());
        Ok(true)
    }

    /// A p-curve off a seam: its basis trimmed to the range (narrowed to a non-periodic
    /// basis) is projected on the new face's surface (on the original plane, whose
    /// parametrisation does not change); a B-spline or Bezier p-curve on an unchanged
    /// face is copied when a conversion is needed.
    fn project_pcurve(
        &self,
        context: &Curve2dContext<'_>,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let (c2d, range) = trimmed_basis(&context.base, context.range)?;
        let c3d = self.new_or_old_3d_curve(context)?;
        let Some(c3d) = c3d else {
            return Ok(false);
        };
        let surface = BRepTool::surface(context.face)?;
        if surface.is_instance_name("Geom_Plane") {
            // No change of parametrisation on a plane.
            return self.project_on_plane(context, &c3d, &surface, curve, tolerance);
        }
        let mut target = surface;
        if context.new_face.is_null() {
            if is_spline_2d(&c2d) {
                if !context.is_convert_2d {
                    return Ok(false);
                }
                *curve = copy_2d(&c2d)?;
                return self.accept(context, (&c3d, &target), curve, tolerance);
            }
        } else {
            let new_surface = BRepTool::surface(context.new_face)?;
            if !new_surface.is_null() {
                target = new_surface;
            }
        }
        self.project_on_surface(context, (&c2d, range), (&c3d, &target), curve, tolerance)
    }

    /// The 3D curve to project: the curve of the new edge when it has one (in the
    /// global frame), else the untransformed curve of the edge read for `isConvert2d`
    /// when there is one, else the curve of the edge in the global frame; none without
    /// any, as natively.
    fn new_or_old_3d_curve(
        &self,
        context: &Curve2dContext<'_>,
    ) -> Result<Option<RangedCurve>, Failure> {
        let mut found = RangedCurve {
            curve: context.raw_3d.curve.clone(),
            range: context.raw_3d.range,
        };
        if !context.new_edge.is_null() {
            let (curve, f, l) = BRepTool::curve(context.new_edge)?;
            found = RangedCurve {
                curve,
                range: (f, l),
            };
        }
        if found.curve.is_null() {
            let (curve, f, l) = BRepTool::curve(context.edge)?;
            found = RangedCurve {
                curve,
                range: (f, l),
            };
        }
        if found.curve.is_null() {
            return Ok(None);
        }
        Ok(Some(found))
    }

    /// On a plane: `ProjLib_ComputeApprox` of the 3D curve, its B-spline, or its Bezier
    /// converted.
    fn project_on_plane(
        &self,
        context: &Curve2dContext<'_>,
        c3d: &RangedCurve,
        surface: &Handle<dyn GeomSurface>,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let adapted: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(surface)?).cast();
        let projection = ComputeApprox::with_curve(&adaptor_3d(c3d)?, &adapted, *tolerance)?;
        *curve = if projection.bspline().is_null() {
            to_bspline_2d(&projection.bezier().cast())?
        } else {
            projection.bspline().cast()
        };
        self.accept(context, (c3d, surface), curve, tolerance)
    }

    /// Off a plane: `ProjLib_ComputeApproxOnPolarSurface` on the widened domain; on a
    /// periodic surface the result is translated back to the old p-curve's domain when
    /// it starts more than half a period away. When the projection fails, the p-curve
    /// is converted instead.
    fn project_on_surface(
        &self,
        context: &Curve2dContext<'_>,
        (c2d, range): (&Handle<dyn Geom2dCurve>, (f64, f64)),
        (c3d, surface): (&RangedCurve, &Handle<dyn GeomSurface>),
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let adapted = widened_adaptor(surface)?;
        let projection = ComputeApproxOnPolarSurface::with_initial_curve(
            &adaptor_2d(c2d, range)?,
            &adaptor_3d(c3d)?,
            &adapted,
            *tolerance,
        )?;
        if !projection.is_done() {
            *curve = to_bspline_2d(c2d)?;
            return self.accept(context, (c3d, surface), curve, tolerance);
        }
        *curve = projection.bspline().cast();
        *curve = restore_domain(surface, c2d, curve, range.0)?;
        self.accept(context, (c3d, surface), curve, tolerance)
    }
}

/// The p-curve translated back by the vector from its new start to the old one when
/// that vector is longer than half a period of the periodic surface.
fn restore_domain(
    surface: &Handle<dyn GeomSurface>,
    old: &Handle<dyn Geom2dCurve>,
    new: &Handle<dyn Geom2dCurve>,
    first: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (Some(stored), Some(old_curve), Some(new_curve)) = (surface.get(), old.get(), new.get())
    else {
        return Ok(new.clone());
    };
    if !stored.is_u_periodic() && !stored.is_v_periodic() {
        return Ok(new.clone());
    }
    let mut min_dist = precision::infinite();
    if stored.is_u_periodic() {
        min_dist = (0.5 * stored.u_period()?).min(min_dist);
    }
    if stored.is_v_periodic() {
        min_dist = (0.5 * stored.v_period()?).min(min_dist);
    }
    min_dist *= min_dist;
    let old_start = old_curve.value(first)?;
    let new_start = new_curve.value(first)?;
    let shift = Vector2::between(new_start, old_start);
    if shift.square_magnitude() > min_dist {
        return Ok(new_curve.translated(shift)?.cast());
    }
    Ok(new.clone())
}

/// `C2d->Copy()` as a Geom2d curve.
pub(super) fn copy_2d(curve: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let Some(stored) = curve.get() else {
        return Ok(Handle::default());
    };
    Ok(stored.copy()?.cast())
}

/// The basis of a trimmed p-curve (exact type), the range narrowed to a non-periodic
/// basis, and the basis trimmed to it.
fn trimmed_basis(
    base: &Handle<dyn Geom2dCurve>,
    (mut f, mut l): (f64, f64),
) -> Result<(Handle<dyn Geom2dCurve>, (f64, f64)), Failure> {
    let mut c2d = base.clone();
    if let Some(trimmed) = c2d.cast::<Geom2dTrimmedCurve>().get() {
        c2d = trimmed.basis_curve();
    }
    let Some(basis) = c2d.get() else {
        return Err(crate::brep::errors::null_object(
            "BRepTools_NurbsConvertModification::NewCurve2d: null p-curve",
        ));
    };
    if !basis.is_periodic() {
        let (fc, lc) = (basis.first_parameter(), basis.last_parameter());
        if fc - f > precision::p_confusion() {
            f = fc;
        }
        if l - lc > precision::p_confusion() {
            l = lc;
        }
    }
    Ok((trimmed_2d(&c2d, (f, l))?, (f, l)))
}

/// The arguments of one `NewCurve2d` call.
pub(super) struct Curve2dContext<'a> {
    pub(super) edge: &'a Edge,
    pub(super) face: &'a Face,
    pub(super) new_edge: &'a Edge,
    pub(super) new_face: &'a Face,
    /// `aBaseC2d`: the p-curve of the edge on the face, the key of the record.
    pub(super) base: Handle<dyn Geom2dCurve>,
    /// Its range.
    pub(super) range: (f64, f64),
    /// `isConvert2d`: the 3D curve is not a spline, or a p-curve needs a conversion.
    pub(super) is_convert_2d: bool,
    /// The untransformed 3D curve of the edge and its range (`C3d`, `f3d`, `l3d` after
    /// the first `BRep_Tool::Curve(E, Loc, f3d, l3d)`).
    pub(super) raw_3d: RangedCurve,
}
