// Created on: 1996-03-12
// Created by: Bruno DUMORTIER
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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_BSplineSurfaceToBezierSurface.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_BSplineSurfaceToBezierSurface`: the Bezier patches of a B-spline
//! surface.
use super::{copy_bspline_surface, live_bspline_surface};
use crate::array_access::{grid, put};
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurface};
use rustycad_tkmath::gp::Point3;

/// A B-spline surface copy whose knots all have multiplicity UDegree (VDegree), so
/// that each knot rectangle is one Bezier patch.
///
/// `let split = BSplineSurfaceToBezierSurface::new(&surface)?; let patch = split.patch(1, 1)?;`
#[derive(Clone)]
pub struct BSplineSurfaceToBezierSurface {
    my_surface: Handle<GeomBSplineSurface>,
}

impl BSplineSurfaceToBezierSurface {
    /// The patches of the whole surface
    /// (`GeomConvert_BSplineSurfaceToBezierSurface(BasisSurface)`);
    /// `BSplineSurfaceToBezierSurface::new(&surface)?`.
    pub fn new(basis_surface: &Handle<GeomBSplineSurface>) -> Result<Self, Failure> {
        let my_surface = copy_bspline_surface(live_bspline_surface(basis_surface)?)?;
        let surface: &GeomBSplineSurface = live_bspline_surface(&my_surface)?;
        let (u1, u2, v1, v2) = surface.bounds();
        split_at_knots(surface, [u1, u2, v1, v2])?;
        Ok(Self { my_surface })
    }

    /// The patches of `[u1, u2] x [v1, v2]`; a bound within `ParametricTolerance / 2`
    /// of a knot inside the domain snaps to it. Standard_DomainError when
    /// `U2 - U1` or `V2 - V1` is below `ParametricTolerance`
    /// (`GeomConvert_BSplineSurfaceToBezierSurface(BasisSurface, U1, U2, V1, V2,
    /// ParametricTolerance)`);
    /// `BSplineSurfaceToBezierSurface::with_range(&surface, 0.0, 1.0, 0.2, 0.8, 1e-9)?`.
    pub fn with_range(
        basis_surface: &Handle<GeomBSplineSurface>,
        (u1, u2): (f64, f64),
        (v1, v2): (f64, f64),
        parametric_tolerance: f64,
    ) -> Result<Self, Failure> {
        if (u2 - u1 < parametric_tolerance) || (v2 - v1 < parametric_tolerance) {
            return Err(failure(
                "Standard_DomainError",
                &format!(
                    "GeomConvert_BSplineSurfaceToBezierSurface: ranges {} x {}; expected >= {parametric_tolerance}",
                    u2 - u1,
                    v2 - v1
                ),
            ));
        }
        let ptol: f64 = parametric_tolerance / 2.0;
        let my_surface = copy_bspline_surface(live_bspline_surface(basis_surface)?)?;
        let surface: &GeomBSplineSurface = live_bspline_surface(&my_surface)?;
        let uf: f64 = snapped(surface, true, u1, ptol, |knot| knot > u1)?;
        let ul: f64 = snapped(surface, true, u2, ptol, |knot| knot < u2)?;
        let vf: f64 = snapped(surface, false, v1, ptol, |knot| knot > v1)?;
        let vl: f64 = snapped(surface, false, v2, ptol, |knot| knot < v2)?;
        split_at_knots(surface, [uf, ul, vf, vl])?;
        Ok(Self { my_surface })
    }

    /// The Bezier patch of knot rectangle (`u_index`, `v_index`), rational when the
    /// B-spline is in either direction; Standard_OutOfRange outside
    /// `1..=NbUPatches x 1..=NbVPatches` (`Patch(UIndex, VIndex)`);
    /// `split.patch(1, 2)?`.
    pub fn patch(&self, u_index: i32, v_index: i32) -> Result<Handle<GeomBezierSurface>, Failure> {
        let surface: &GeomBSplineSurface = live_bspline_surface(&self.my_surface)?;
        if u_index < 1
            || u_index > surface.nb_u_knots() - 1
            || v_index < 1
            || v_index > surface.nb_v_knots() - 1
        {
            return Err(failure(
                "Standard_OutOfRange",
                &format!(
                    "GeomConvert_BSplineSurfaceToBezierSurface: patch ({u_index}, {v_index}); expected within {} x {}",
                    surface.nb_u_knots() - 1,
                    surface.nb_v_knots() - 1
                ),
            ));
        }
        let (udeg, vdeg): (i32, i32) = (surface.u_degree(), surface.v_degree());
        let offsets: [i32; 2] = [udeg * (u_index - 1), vdeg * (v_index - 1)];
        let poles: Array2<'static, Point3> =
            patch_grid(udeg, vdeg, offsets, |i, j| surface.pole(i, j))?;
        if !(surface.is_u_rational() || surface.is_v_rational()) {
            return Ok(Handle::new(GeomBezierSurface::new(&poles)?));
        }
        let weights: Array2<'static, f64> =
            patch_grid(udeg, vdeg, offsets, |i, j| surface.weight(i, j))?;
        Ok(Handle::new(GeomBezierSurface::with_weights(
            &poles, &weights,
        )?))
    }

    /// Stores every patch into `surfaces` at `(1..=NbUPatches, 1..=NbVPatches)`;
    /// Standard_OutOfRange for an index outside `surfaces` (`Patches(Surfaces)`);
    /// `split.patches(&mut patches)?`.
    pub fn patches(
        &self,
        surfaces: &mut Array2<'_, Handle<GeomBezierSurface>>,
    ) -> Result<(), Failure> {
        let (nb_u, nb_v): (i32, i32) = (self.nb_u_patches()?, self.nb_v_patches()?);
        for i in 1..=nb_u {
            for j in 1..=nb_v {
                put(surfaces, i, j, self.patch(i, j)?, "Patches")?;
            }
        }
        Ok(())
    }

    /// Copies the U knots of the split surface into `t_knots` from its lower bound
    /// (`UKnots(TKnots)`); `split.u_knots(&mut knots)?`.
    pub fn u_knots(&self, t_knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let surface: &GeomBSplineSurface = live_bspline_surface(&self.my_surface)?;
        copy_knots(surface.nb_u_knots(), |ii| surface.u_knot(ii), t_knots)
    }

    /// Copies the V knots of the split surface into `t_knots` from its lower bound
    /// (`VKnots(TKnots)`); `split.v_knots(&mut knots)?`.
    pub fn v_knots(&self, t_knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let surface: &GeomBSplineSurface = live_bspline_surface(&self.my_surface)?;
        copy_knots(surface.nb_v_knots(), |ii| surface.v_knot(ii), t_knots)
    }

    /// The number of patches in U, NbUKnots - 1 (`NbUPatches()`); `split.nb_u_patches()?`.
    pub fn nb_u_patches(&self) -> Result<i32, Failure> {
        Ok(live_bspline_surface(&self.my_surface)?.nb_u_knots() - 1)
    }

    /// The number of patches in V, NbVKnots - 1 (`NbVPatches()`); `split.nb_v_patches()?`.
    pub fn nb_v_patches(&self) -> Result<i32, Failure> {
        Ok(live_bspline_surface(&self.my_surface)?.nb_v_knots() - 1)
    }
}

// The bound snapped to the knot it lies on (LocateU/LocateV within `ptol`) when
// `inside` accepts that knot.
fn snapped(
    surface: &GeomBSplineSurface,
    in_u: bool,
    bound: f64,
    ptol: f64,
    inside: impl Fn(f64) -> bool,
) -> Result<f64, Failure> {
    let (i1, i2): (i32, i32) = if in_u {
        surface.locate_u(bound, ptol, false)?
    } else {
        surface.locate_v(bound, ptol, false)?
    };
    // We are on the knot.
    if i1 == i2 {
        let knot: f64 = if in_u {
            surface.u_knot(i1)?
        } else {
            surface.v_knot(i1)?
        };
        if inside(knot) {
            return Ok(knot);
        }
    }
    Ok(bound)
}

// Segment(Uf, Ul, Vf, Vl), then every knot raised to multiplicity UDegree (VDegree).
fn split_at_knots(surface: &GeomBSplineSurface, [uf, ul, vf, vl]: [f64; 4]) -> Result<(), Failure> {
    let tolerances: (f64, f64) = (precision::p_confusion(), precision::p_confusion());
    surface.segment((uf, ul), (vf, vl), tolerances)?;
    surface.increase_u_multiplicity_range(
        surface.first_u_knot_index()?,
        surface.last_u_knot_index()?,
        surface.u_degree(),
    )?;
    surface.increase_v_multiplicity_range(
        surface.first_v_knot_index()?,
        surface.last_v_knot_index()?,
        surface.v_degree(),
    )
}

// The (UDeg + 1) x (VDeg + 1) grid read at the patch offsets.
fn patch_grid<T: Default>(
    udeg: i32,
    vdeg: i32,
    [u_offset, v_offset]: [i32; 2],
    read: impl Fn(i32, i32) -> Result<T, Failure>,
) -> Result<Array2<'static, T>, Failure> {
    let mut values: Array2<'static, T> = grid(udeg + 1, vdeg + 1)?;
    for i in 1..=udeg + 1 {
        for j in 1..=vdeg + 1 {
            put(
                &mut values,
                i,
                j,
                read(i + u_offset, j + v_offset)?,
                "Patch",
            )?;
        }
    }
    Ok(values)
}

// TKnots(kk) = Knot(ii) for ii in 1..=count, kk from TKnots.Lower().
fn copy_knots(
    count: i32,
    knot: impl Fn(i32) -> Result<f64, Failure>,
    t_knots: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    for (ii, kk) in (1..=count).zip(t_knots.lower()..) {
        t_knots
            .set_value(kk, knot(ii)?)
            .map_err(|e| array_failure(e, "Knots"))?;
    }
    Ok(())
}
