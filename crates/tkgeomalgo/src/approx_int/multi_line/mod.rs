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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_MultiLine.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ApproxInt_MultiLine.gxx`: the points of a walking line, shifted by the offsets of
//! the approximation, as the multi-line of the AppParCurves generics
//! (docs/geomalgo-design.md section 20). The `ApproxInt_MultiLineTool.lxx` tool is its
//! `LineTool` implementation (`line_tool`); the point insertions are in `make_between`
//! and `one_more_point`.
mod dump;
mod line_tool;
mod make_between;
mod one_more_point;

use super::line::ApproxIntLine;
use super::sv_surfaces::SvSurfacesRef;
use crate::int_patch::failures::null_object;
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays;
use rustycad_tkgeombase::approx::Status;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// The offsets added to the points of the line (`Xo, Yo, Zo, U1o, V1o, U2o, V2o`),
/// which bring the approximated points near the origin.
///
/// ```
/// use rustycad_tkgeomalgo::approx_int::ApproxIntOffsets;
/// assert_eq!(ApproxIntOffsets::default().u2o, 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ApproxIntOffsets {
    /// `Xo`.
    pub xo: f64,
    /// `Yo`.
    pub yo: f64,
    /// `Zo`.
    pub zo: f64,
    /// `U1o`.
    pub u1o: f64,
    /// `V1o`.
    pub v1o: f64,
    /// `U2o`.
    pub u2o: f64,
    /// `V2o`.
    pub v2o: f64,
}

/// The constructor arguments of a multi-line after the line and the SvSurfaces:
/// `NbP3d, NbP2d, ApproxU1V1, ApproxU2V2, xo... v2o, P2DOnFirst, IndMin, IndMax`
/// (the native defaults of `IndMin` and `IndMax` are 0).
///
/// ```
/// use rustycad_tkgeomalgo::approx_int::ApproxIntMultiLineLayout;
/// let layout = ApproxIntMultiLineLayout { nb_p3d: 1, ind_max: 5, ..Default::default() };
/// assert_eq!(layout.ind_min, 0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ApproxIntMultiLineLayout {
    /// `NbP3d`: 1 when the 3d points are approximated, else 0.
    pub nb_p3d: i32,
    /// `NbP2d`: the number of parameter curves (0 to 2).
    pub nb_p2d: i32,
    /// `ApproxU1V1`.
    pub approx_u1v1: bool,
    /// `ApproxU2V2`.
    pub approx_u2v2: bool,
    /// The offsets of the points.
    pub offsets: ApproxIntOffsets,
    /// `P2DOnFirst`: with one parameter curve, whether it is on the first surface.
    pub p2d_on_first: bool,
    /// `IndMin`.
    pub ind_min: i32,
    /// `IndMax`.
    pub ind_max: i32,
}

/// `ApproxInt_MultiLine` over the line `L` (GeomInt_TheMultiLineOfWLApprox over
/// IntPatch_WLine): the multipoints `FirstPoint..=LastPoint` of the line, with one 3d
/// point and up to two 2d points, and the SvSurfaces that add points when it is set.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeomalgo::approx_int::{ApproxIntMultiLine, ApproxIntMultiLineLayout};
/// use rustycad_tkgeomalgo::int_patch::IntPatchWLine;
/// let line = ApproxIntMultiLine::<IntPatchWLine>::with_line(Handle::default(), ApproxIntMultiLineLayout {
///     ind_min: 7, ind_max: 3, ..Default::default()
/// });
/// assert_eq!((line.first_point(), line.last_point()), (3, 7));
/// ```
pub struct ApproxIntMultiLine<L: ApproxIntLine> {
    ptr_on_sv_surfaces: Option<SvSurfacesRef>,
    my_line: Handle<L>,
    indicemin: i32,
    indicemax: i32,
    nbp3d: i32,
    nbp2d: i32,
    approx_u1v1: bool,
    approx_u2v2: bool,
    p2d_on_first: bool,
    offsets: ApproxIntOffsets,
}

impl<L: ApproxIntLine> Clone for ApproxIntMultiLine<L> {
    fn clone(&self) -> Self {
        Self {
            ptr_on_sv_surfaces: self.ptr_on_sv_surfaces.clone(),
            my_line: self.my_line.clone(),
            ..*self
        }
    }
}

impl<L: ApproxIntLine> Default for ApproxIntMultiLine<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: ApproxIntLine> ApproxIntMultiLine<L> {
    /// `ApproxInt_MultiLine()`: no line, no SvSurfaces, the indices 0 and `P2DOnFirst`
    /// true; `ApproxIntMultiLine::<IntPatchWLine>::new()`.
    pub fn new() -> Self {
        Self {
            ptr_on_sv_surfaces: None,
            my_line: Handle::default(),
            indicemin: 0,
            indicemax: 0,
            nbp3d: 0,
            nbp2d: 0,
            approx_u1v1: false,
            approx_u2v2: false,
            p2d_on_first: true,
            offsets: ApproxIntOffsets::default(),
        }
    }

    /// `ApproxInt_MultiLine(line, svsurf, NbP3d, ...)`: the line whose SvSurfaces add
    /// points when the approximation needs them; the indices are ordered;
    /// `ApproxIntMultiLine::with_sv_surfaces(line, Some(sv), layout)`.
    pub fn with_sv_surfaces(
        line: Handle<L>,
        sv_surfaces: Option<SvSurfacesRef>,
        layout: ApproxIntMultiLineLayout,
    ) -> Self {
        Self {
            ptr_on_sv_surfaces: sv_surfaces,
            my_line: line,
            indicemin: layout.ind_min.min(layout.ind_max),
            indicemax: layout.ind_min.max(layout.ind_max),
            nbp3d: layout.nb_p3d,
            nbp2d: layout.nb_p2d,
            approx_u1v1: layout.approx_u1v1,
            approx_u2v2: layout.approx_u2v2,
            p2d_on_first: layout.p2d_on_first,
            offsets: layout.offsets,
        }
    }

    /// `ApproxInt_MultiLine(line, NbP3d, ...)`: no extra point will be added;
    /// `ApproxIntMultiLine::with_line(line, layout)`.
    pub fn with_line(line: Handle<L>, layout: ApproxIntMultiLineLayout) -> Self {
        Self::with_sv_surfaces(line, None, layout)
    }

    /// The constructor arguments of this line, for the lines made from it.
    pub(crate) fn layout(&self, ind_min: i32, ind_max: i32) -> ApproxIntMultiLineLayout {
        ApproxIntMultiLineLayout {
            nb_p3d: self.nbp3d,
            nb_p2d: self.nbp2d,
            approx_u1v1: self.approx_u1v1,
            approx_u2v2: self.approx_u2v2,
            offsets: self.offsets,
            p2d_on_first: self.p2d_on_first,
            ind_min,
            ind_max,
        }
    }

    /// The line, or Standard_NullObject where the native code dereferences a null one.
    pub(crate) fn line(&self, member: &str) -> Result<&L, Failure> {
        self.my_line
            .get()
            .ok_or_else(|| null_object(&format!("ApproxInt_MultiLine::{member}"), "line"))
    }

    /// `FirstPoint()`; `line.first_point()`.
    pub fn first_point(&self) -> i32 {
        self.indicemin
    }

    /// `LastPoint()`; `line.last_point()`.
    pub fn last_point(&self) -> i32 {
        self.indicemax
    }

    /// `WhatStatus()`: points can be added when the SvSurfaces are set;
    /// `line.what_status()`.
    pub fn what_status(&self) -> Status {
        if self.ptr_on_sv_surfaces.is_some() {
            Status::PointsAdded
        } else {
            Status::NoPointsAdded
        }
    }

    /// `NbP3d()`; `line.nb_p3d()`.
    pub fn nb_p3d(&self) -> i32 {
        self.nbp3d
    }

    /// `NbP2d()`; `line.nb_p2d()`.
    pub fn nb_p2d(&self) -> i32 {
        self.nbp2d
    }

    /// `Value(Index, TabPnt)`: the shifted 3d point, written at index 1;
    /// `line.value_3d(i, &mut tab)?`.
    pub fn value_3d(&self, index: i32, tab: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        let p: Point3 = self.line("Value")?.point(index)?.value();
        let o = self.offsets;
        arrays::put(
            tab,
            1,
            Point3::new(p.x() + o.xo, p.y() + o.yo, p.z() + o.zo),
        )
    }

    /// `Value(Index, TabPnt2d)`: the shifted parameters on the approximated surfaces
    /// (on one of them when `NbP2d` is 1); `line.value_2d(i, &mut tab2d)?`.
    pub fn value_2d(&self, index: i32, tab: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        let pon2s: IntSurfPntOn2S = self.line("Value")?.point(index)?;
        let (u1, v1, u2, v2) = pon2s.parameters();
        let o = self.offsets;
        let on1 = Point2::new(u1 + o.u1o, v1 + o.v1o);
        let on2 = Point2::new(u2 + o.u2o, v2 + o.v2o);
        if self.nbp2d == 1 {
            return arrays::put(tab, 1, if self.p2d_on_first { on1 } else { on2 });
        }
        arrays::put(tab, 1, on1)?;
        if tab.length() >= 2 {
            arrays::put(tab, 2, on2)?;
        }
        Ok(())
    }

    /// `Value(Index, TabPnt, TabPnt2d)`; `line.value(i, &mut tab, &mut tab2d)?`.
    pub fn value(
        &self,
        index: i32,
        tab: &mut Array1<'_, Point3>,
        tab2d: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        self.value_3d(index, tab)?;
        self.value_2d(index, tab2d)
    }

    /// The parameters of the point `index` and the SvSurfaces, or None without them.
    fn sv_at(&self, index: i32) -> Result<Option<(SvSurfacesRef, [f64; 4])>, Failure> {
        let Some(sv) = self.ptr_on_sv_surfaces.clone() else {
            return Ok(None);
        };
        let (u1, v1, u2, v2) = self.line("Tangency")?.point(index)?.parameters();
        Ok(Some((sv, [u1, v1, u2, v2])))
    }

    /// `Tangency(Index, TabVec)`: the 3d tangent from the SvSurfaces (a null vector
    /// when it cannot be computed); false without SvSurfaces;
    /// `line.tangency_3d(i, &mut tab)?`.
    pub fn tangency_3d(&self, index: i32, tab: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        let Some((sv, uv)) = self.sv_at(index)? else {
            return Ok(false);
        };
        // The native `TabVec(1)` argument is checked before the call.
        arrays::at(tab, 1)?;
        let (ret, tg) = sv.borrow_mut().tangency(uv)?;
        arrays::put(tab, 1, if ret { tg } else { Vector3::new(0.0, 0.0, 0.0) })?;
        Ok(ret)
    }

    /// `Tangency(Index, TabVec2d)`: the tangents in the parameter planes (null vectors
    /// when one cannot be computed); false without SvSurfaces;
    /// `line.tangency_2d(i, &mut tab2d)?`.
    pub fn tangency_2d(&self, index: i32, tab: &mut Array1<'_, Vector2>) -> Result<bool, Failure> {
        let Some((sv, uv)) = self.sv_at(index)? else {
            return Ok(false);
        };
        arrays::at(tab, 1)?;
        let mut sv = sv.borrow_mut();
        let first = self.nbp2d != 1 || self.p2d_on_first;
        let (mut ret, tg1) = if first {
            sv.tangency_on_surf1(uv)?
        } else {
            sv.tangency_on_surf2(uv)?
        };
        arrays::put(tab, 1, tg1)?;
        if self.nbp2d != 1 && ret && tab.length() >= 2 {
            let (ret2, tg2) = sv.tangency_on_surf2(uv)?;
            arrays::put(tab, 2, tg2)?;
            ret = ret && ret2;
        }
        if !ret {
            clear_2d(tab)?;
        }
        Ok(ret)
    }

    /// `Tangency(Index, TabVec, TabVec2d)`: the 2d tangents are computed only when
    /// the 3d one is; `line.tangency(i, &mut tab, &mut tab2d)?`.
    pub fn tangency(
        &self,
        index: i32,
        tab: &mut Array1<'_, Vector3>,
        tab2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        Ok(self.tangency_3d(index, tab)? && self.tangency_2d(index, tab2d)?)
    }
}

// The null tangents of a failed `Tangency(Index, TabVec2d)`.
fn clear_2d(tab: &mut Array1<'_, Vector2>) -> Result<(), Failure> {
    arrays::put(tab, 1, Vector2::new(0.0, 0.0))?;
    if tab.length() >= 2 {
        arrays::put(tab, 2, Vector2::new(0.0, 0.0))?;
    }
    Ok(())
}
