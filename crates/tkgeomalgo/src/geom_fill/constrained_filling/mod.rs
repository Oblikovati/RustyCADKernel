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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstrainedFilling.hxx/.cxx
// (the fields, the constructor and the consultation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_ConstrainedFilling`: the B-spline surface filled between three or four
//! boundaries, tangent to the constraint surfaces of the boundaries that have one.
//! The work is split by step: the sort and corner checks (`corners`), `Init`
//! (`init`), the approximation of the boundaries (`approx`), the knot matching
//! (`match_knots`), the pole construction (`poles`) and the checks (`checks`).
mod approx;
mod checks;
mod corners;
mod init;
mod match_knots;
mod poles;

use super::GeomFillTgtField;
use super::{GeomFillBoundWithSurf, GeomFillBoundary, GeomFillCoonsAlgPatch, GeomFillCornerState};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkmath::gp::{Point3, Vector3};

/// A pole row of the native `occ::handle<NCollection_HArray1<gp_Pnt>>` fields
/// (`None` is the null handle).
type PoleRow = Option<Array1<'static, Point3>>;

/// The filling of three or four boundaries (`GeomFill_ConstrainedFilling`); the
/// boundaries given as `GeomFillBoundWithSurf` also constrain the tangent planes.
///
/// `let mut f = GeomFillConstrainedFilling::new(8, 2); f.init_four([&b1, &b2, &b3, &b4], false)?; f.surface()`
pub struct GeomFillConstrainedFilling {
    degmax: i32,
    segmax: i32,
    ptch: Handle<GeomFillCoonsAlgPatch>,
    tgalg: [Handle<dyn GeomFillTgtField>; 4],
    mig: [f64; 4],
    stcor: [GeomFillCornerState; 4],
    v: [Vector3; 4],
    appdone: bool,
    degree: [i32; 2],
    curvpol: [PoleRow; 4],
    tgtepol: [PoleRow; 4],
    mults: [Option<Array1<'static, i32>>; 2],
    knots: [Option<Array1<'static, f64>>; 2],
    ab: [Handle<HArray1<f64>>; 4],
    pq: [Handle<HArray1<f64>>; 4],
    dom: [f64; 4],
    ncpol: [PoleRow; 4],
    ntpol: [PoleRow; 4],
    nm: [Option<Array1<'static, i32>>; 2],
    nk: [Option<Array1<'static, f64>>; 2],
    ibound: [i32; 2],
    ctr: [i32; 2],
    nbd3: i32,
    s0: Option<Array2<'static, Point3>>,
    s1: Option<Array2<'static, Point3>>,
    surf: Handle<GeomBSplineSurface>,
}

impl GeomFillConstrainedFilling {
    /// `GeomFill_ConstrainedFilling(MaxDeg, MaxSeg)`: an empty framework whose
    /// surface has degrees at most `max_deg` and at most `max_seg` spans; the domains
    /// start at 1; `GeomFillConstrainedFilling::new(8, 2)`.
    pub fn new(max_deg: i32, max_seg: i32) -> Self {
        Self {
            degmax: max_deg,
            segmax: max_seg,
            ptch: Handle::default(),
            tgalg: Default::default(),
            mig: [0.; 4],
            stcor: [GeomFillCornerState::new(); 4],
            v: [Vector3::default(); 4],
            appdone: false,
            degree: [0; 2],
            curvpol: Default::default(),
            tgtepol: Default::default(),
            mults: Default::default(),
            knots: Default::default(),
            ab: Default::default(),
            pq: Default::default(),
            dom: [1.; 4],
            ncpol: Default::default(),
            ntpol: Default::default(),
            nm: Default::default(),
            nk: Default::default(),
            ibound: [0; 2],
            ctr: [0; 2],
            nbd3: 0,
            s0: None,
            s1: None,
            surf: Handle::default(),
        }
    }
    /// `SetDomain(l, B)`: the relative range `min(1, |l|)` over which the tangent
    /// field of the constrained boundary `b` spreads (`b` is compared by identity
    /// with the sorted boundaries; nothing changes when it is none of them); a
    /// framework without `Init` raises Standard_NullObject;
    /// `f.set_domain(0.5, &bound)?`.
    pub fn set_domain(&mut self, l: f64, b: &Handle<GeomFillBoundWithSurf>) -> Result<(), Failure> {
        let target = b.object_address();
        for k in 0..4 {
            if self.patch()?.bound(k)?.object_address() == target {
                self.dom[k as usize] = l.abs().min(1.);
                return Ok(());
            }
        }
        Ok(())
    }
    /// `ReBuild()`: the poles again with the domains of `set_domain`;
    /// Standard_Failure before a successful approximation; `f.re_build()?`.
    pub fn re_build(&mut self) -> Result<(), Failure> {
        if !self.appdone {
            return Err(Failure::with_message(
                "Standard_Failure",
                "GeomFill_ConstrainedFilling::ReBuild Approx non faite; expected Init first",
            ));
        }
        self.match_knots()?;
        self.perform_s0()?;
        self.perform_s1()?;
        self.perform_surface()
    }
    /// `Boundary(I)`: sorted boundary `i` (0 to 3); Standard_OutOfRange outside;
    /// `f.boundary(0)?`.
    pub fn boundary(&self, i: i32) -> Result<Handle<dyn GeomFillBoundary>, Failure> {
        self.patch()?.bound(i)
    }
    /// `Surface()`: the filled surface, null before `Init`; `f.surface()`.
    pub fn surface(&self) -> Handle<GeomBSplineSurface> {
        self.surf.clone()
    }
    /// The Coons patch, or the Standard_NullObject of the native null dereference.
    fn patch(&self) -> Result<&GeomFillCoonsAlgPatch, Failure> {
        self.ptch
            .get()
            .ok_or_else(|| errors::null_object("GeomFill_ConstrainedFilling: no Init"))
    }
}

/// A field that the native code dereferences: Standard_NullObject when null.
fn required<'a, T>(value: &'a Option<T>, name: &str) -> Result<&'a T, Failure> {
    value.as_ref().ok_or_else(|| {
        errors::null_object(&format!(
            "GeomFill_ConstrainedFilling: {name} is null; expected an approximated boundary"
        ))
    })
}

/// A tangent field that the native code dereferences.
fn field(handle: &Handle<dyn GeomFillTgtField>) -> Result<&dyn GeomFillTgtField, Failure> {
    handle
        .get()
        .ok_or_else(|| errors::null_object("GeomFill_ConstrainedFilling: null tangent field"))
}

/// A boundary that the native code dereferences.
fn live(handle: &Handle<dyn GeomFillBoundary>) -> Result<&dyn GeomFillBoundary, Failure> {
    handle
        .get()
        .ok_or_else(|| errors::null_object("GeomFill_ConstrainedFilling: null boundary"))
}
