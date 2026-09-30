// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Pipe.hxx/.lxx
// (the constructors and the accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Pipe`: the pipe surfaces of a section along a path — a tube, a
//! section moved by a trihedron law, two or more sections, a circular blend between
//! two rails, or a section guided by a curve (docs/geomalgo-design.md, section 14).
use super::location_law::GeomFillLocationLaw;
use super::pipe_error::{GeomFillPipeError, GeomFillTrihedron};
use super::section_law::GeomFillSectionLaw;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::Direction3;

/// `GeomFill_Pipe`: the kind of pipe (`myType`: 0 a law pipe, 1 a tube, 3 several
/// sections, 4 a blend between rails), its laws or rails, and the result.
///
/// `let mut pipe = GeomFillPipe::with_radius(&path, 1.0)?; pipe.perform(1e-4, false, Shape::C1, 11, 30)?;`.
pub struct GeomFillPipe {
    pub(crate) my_status: GeomFillPipeError,
    pub(crate) my_radius: f64,
    pub(crate) my_error: f64,
    pub(crate) my_adp_path: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_adp_first_sect: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_adp_last_sect: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_surface: Handle<dyn GeomSurface>,
    pub(crate) my_loc: Handle<dyn GeomFillLocationLaw>,
    pub(crate) my_sec: Handle<dyn GeomFillSectionLaw>,
    pub(crate) my_type: i32,
    pub(crate) my_exch_uv: bool,
    pub(crate) my_k_part: bool,
    pub(crate) my_polynomial: bool,
}

impl Default for GeomFillPipe {
    fn default() -> Self {
        Self::new()
    }
}

impl GeomFillPipe {
    /// `GeomFill_Pipe()`: an empty pipe (not done, particular cases on);
    /// `GeomFillPipe::new()`.
    pub fn new() -> Self {
        let mut pipe = Self {
            my_status: GeomFillPipeError::PipeNotOk,
            my_radius: 0.0,
            my_error: 0.0,
            my_adp_path: Handle::default(),
            my_adp_first_sect: Handle::default(),
            my_adp_last_sect: Handle::default(),
            my_surface: Handle::default(),
            my_loc: Handle::default(),
            my_sec: Handle::default(),
            my_type: 0,
            my_exch_uv: false,
            my_k_part: false,
            my_polynomial: false,
        };
        pipe.reset();
        pipe
    }

    /// The private `Init()`: no kind, no laws, particular cases on.
    pub(crate) fn reset(&mut self) {
        (self.my_type, self.my_error, self.my_radius) = (0, 0.0, 0.0);
        (self.my_k_part, self.my_polynomial) = (true, false);
        self.my_adp_path = Handle::default();
        self.my_adp_first_sect = Handle::default();
        self.my_adp_last_sect = Handle::default();
        self.my_loc = Handle::default();
        self.my_sec = Handle::default();
    }

    /// `GeomFill_Pipe(Path, Radius)`: a tube; `GeomFillPipe::with_radius(&path, 1.0)?`.
    pub fn with_radius(path: &Handle<dyn GeomCurve>, radius: f64) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init_radius(path, radius)?;
        Ok(pipe)
    }

    /// `GeomFill_Pipe(Path, FirstSect, Option)` (natively defaulted to the corrected
    /// Frenet trihedron); `GeomFillPipe::with_section(&path, &sect, GeomFillTrihedron::IsFrenet)?`.
    pub fn with_section(
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        option: GeomFillTrihedron,
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init_section(path, first_sect, option)?;
        Ok(pipe)
    }

    /// `GeomFill_Pipe(Path, Support, FirstSect)`: a path on a surface, with the
    /// Darboux trihedron; `GeomFillPipe::on_surface(&path2d, &support, &sect)?`.
    pub fn on_surface(
        path: &Handle<dyn Geom2dCurve>,
        support: &Handle<dyn GeomSurface>,
        first_sect: &Handle<dyn GeomCurve>,
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init_on_surface(path, support, first_sect)?;
        Ok(pipe)
    }

    /// `GeomFill_Pipe(Path, FirstSect, LastSect)`;
    /// `GeomFillPipe::with_two_sections(&path, &first, &last)?`.
    pub fn with_two_sections(
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        last_sect: &Handle<dyn GeomCurve>,
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init_two_sections(path, first_sect, last_sect)?;
        Ok(pipe)
    }

    /// `GeomFill_Pipe(Path, NSections)`; `GeomFillPipe::with_sections(&path, &sections)?`.
    pub fn with_sections(
        path: &Handle<dyn GeomCurve>,
        n_sections: &[Handle<dyn GeomCurve>],
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init_sections(path, n_sections)?;
        Ok(pipe)
    }

    /// `GeomFill_Pipe(Path, Curve1, Direction)`: a section kept normal to a
    /// direction; `GeomFillPipe::with_direction(&path, &sect, Direction3::Z)?`.
    pub fn with_direction(
        path: &Handle<dyn GeomCurve>,
        curve1: &Handle<dyn GeomCurve>,
        direction: Direction3,
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init_direction(path, curve1, direction)?;
        Ok(pipe)
    }

    /// `GeomFill_Pipe(Path, Curve1, Curve2, Radius)` of geometric curves;
    /// `GeomFillPipe::with_rails(&path, &c1, &c2, 1.0)?`.
    pub fn with_rails(
        path: &Handle<dyn GeomCurve>,
        curve1: &Handle<dyn GeomCurve>,
        curve2: &Handle<dyn GeomCurve>,
        radius: f64,
    ) -> Result<Self, Failure> {
        let adaptor = |c: &Handle<dyn GeomCurve>| -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
            Ok(Handle::new(GeomAdaptorCurve::with_curve(c)?).cast())
        };
        let (p, c1, c2) = (adaptor(path)?, adaptor(curve1)?, adaptor(curve2)?);
        Ok(Self::with_adaptor_rails(&p, &c1, &c2, radius))
    }

    /// `GeomFill_Pipe(Path, Curve1, Curve2, Radius)` of adaptors;
    /// `GeomFillPipe::with_adaptor_rails(&path, &c1, &c2, 1.0)`.
    pub fn with_adaptor_rails(
        path: &Handle<dyn Adaptor3dCurve>,
        curve1: &Handle<dyn Adaptor3dCurve>,
        curve2: &Handle<dyn Adaptor3dCurve>,
        radius: f64,
    ) -> Self {
        let mut pipe = Self::new();
        pipe.init_rails(path, curve1, curve2, radius);
        pipe
    }

    /// `GeomFill_Pipe(Path, Guide, FirstSect, byACR, rotat)`: a section guided by a
    /// curve (by the curvilinear abscissa or by planes);
    /// `GeomFillPipe::with_guide(&path, &guide, &sect, true, false)?`.
    pub fn with_guide(
        path: &Handle<dyn GeomCurve>,
        guide: &Handle<dyn Adaptor3dCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        by_acr: bool,
        rotat: bool,
    ) -> Result<Self, Failure> {
        let mut pipe = Self::new();
        pipe.init_guide(path, guide, first_sect, by_acr, rotat)?;
        Ok(pipe)
    }

    /// `Surface()`: the result (null before a successful `Perform`); `pipe.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.my_surface.clone()
    }

    /// `ExchangeUV()`: whether the U of the surface is the path; `pipe.exchange_uv()`.
    pub fn exchange_uv(&self) -> bool {
        self.my_exch_uv
    }

    /// `GenerateParticularCase(B)`: whether the law pipes may build elementary
    /// surfaces; `pipe.set_generate_particular_case(false)`.
    pub fn set_generate_particular_case(&mut self, b: bool) {
        self.my_k_part = b;
    }

    /// `GenerateParticularCase()`; `pipe.generate_particular_case()`.
    pub fn generate_particular_case(&self) -> bool {
        self.my_k_part
    }

    /// `ErrorOnSurf()`: the approximation error; `pipe.error_on_surf()`.
    pub fn error_on_surf(&self) -> f64 {
        self.my_error
    }

    /// `IsDone()`: the status is `GeomFill_PipeOk`; `pipe.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_status == GeomFillPipeError::PipeOk
    }

    /// `GetStatus()`; `pipe.get_status()`.
    pub fn get_status(&self) -> GeomFillPipeError {
        self.my_status
    }
}
