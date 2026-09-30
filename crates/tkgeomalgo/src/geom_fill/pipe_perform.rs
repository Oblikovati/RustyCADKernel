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
// Rust translation of Perform, ApproxSurf and KPartT4 of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Pipe.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The computation of a pipe: `GeomFill_Sweep` for the law pipes; for the blend
//! between rails, a cylinder or a torus when the rails allow it, else the
//! approximation of `GeomFill_CircularBlendFunc` (or of the old section
//! generator).
use super::app_sweep::GeomFillAppSweep;
use super::approx_style::GeomFillApproxStyle;
use super::circular_blend_func::GeomFillCircularBlendFunc;
use super::line::GeomFillLine;
use super::pipe::GeomFillPipe;
use super::pipe_error::GeomFillPipeError;
use super::sweep::GeomFillSweep;
use super::sweep_failures::{construction_error, live};
use super::sweep_section_generator::GeomFillSweepSectionGenerator;
use crate::app_blend::AppBlendApprox;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkgeombase::approx::{SweepApproximation, SweepFunction, SweepOptions};
use rustycad_tkmath::geom_abs::Shape;

/// The continuity the approximation can reach for `conti` (C0, C1 or C2).
fn approx_continuity(conti: Shape) -> Shape {
    match conti {
        Shape::C0 => Shape::C0,
        Shape::G1 | Shape::C1 => Shape::C1,
        _ => Shape::C2,
    }
}

/// The non-periodic B-spline surface of a sweep approximation.
fn sweep_surface(app: &SweepApproximation) -> Result<GeomBSplineSurface, Failure> {
    GeomBSplineSurface::with_weights(
        app.surf_poles()?,
        app.surf_weights()?,
        (app.surf_u_knots()?, app.surf_u_mults()?),
        (app.surf_v_knots()?, app.surf_v_mults()?),
        (app.u_degree()?, app.v_degree()?),
        (false, false),
    )
}

impl GeomFillPipe {
    /// `Perform(WithParameters, Polynomial)` (natively defaulted to false): a law
    /// pipe through `Perform(1e-4, Polynomial)`; a blend by its particular case,
    /// else by the section generator; `pipe.perform_with_parameters(false, false)?`.
    pub fn perform_with_parameters(
        &mut self,
        with_parameters: bool,
        polynomial: bool,
    ) -> Result<(), Failure> {
        if !self.my_loc.is_null() && !self.my_sec.is_null() {
            return self.perform(1.0e-4, polynomial, Shape::C1, 11, 30);
        }
        self.my_polynomial = polynomial;
        if self.my_polynomial || !self.k_part_t4()? {
            self.approx_surf(with_parameters)?;
        }
        Ok(())
    }

    /// `Perform(Tol, Polynomial, Conti, DegMax, NbMaxSegment)` (natively defaulted to
    /// C1, 11 and 30); `pipe.perform(1e-4, false, Shape::C1, 11, 30)?`.
    pub fn perform(
        &mut self,
        tol: f64,
        polynomial: bool,
        conti: Shape,
        deg_max: i32,
        nb_max_segment: i32,
    ) -> Result<(), Failure> {
        if self.my_status == GeomFillPipeError::ImpossibleContact {
            return Ok(());
        }
        let the_conti = approx_continuity(conti);
        if self.my_type == 4 {
            if !self.k_part_t4()? {
                self.blend_approximation(tol, polynomial, (the_conti, deg_max, nb_max_segment))?;
            }
            return Ok(());
        }
        if self.my_loc.is_null() || self.my_sec.is_null() {
            return self.perform_with_parameters(true, polynomial);
        }
        self.law_sweep(tol, (the_conti, deg_max, nb_max_segment))
    }

    /// A law pipe: the `GeomFill_Sweep` of the section law along the location law.
    fn law_sweep(
        &mut self,
        tol: f64,
        (conti, deg_max, nb_max_segment): (Shape, i32, i32),
    ) -> Result<(), Failure> {
        let mut sweep = GeomFillSweep::new(&self.my_loc, self.my_k_part)?;
        sweep.set_tolerance(tol, 1.0, 1.0e-5, 1.0);
        sweep.build(
            &self.my_sec,
            GeomFillApproxStyle::Location,
            conti,
            deg_max,
            nb_max_segment,
        )?;
        if sweep.is_done() {
            self.my_surface = sweep.surface();
            self.my_error = sweep.error_on_surface();
            self.my_status = GeomFillPipeError::PipeOk;
        }
        Ok(())
    }

    /// The approximation of the circular blend between the rails.
    fn blend_approximation(
        &mut self,
        tol: f64,
        polynomial: bool,
        shape: (Shape, i32, i32),
    ) -> Result<(), Failure> {
        let func = GeomFillCircularBlendFunc::new(
            &self.my_adp_path,
            &self.my_adp_first_sect,
            &self.my_adp_last_sect,
            self.my_radius,
            polynomial,
        )?;
        let func: Handle<dyn SweepFunction> = Handle::new(func).cast();
        let mut app = SweepApproximation::new(&func);
        let path = live(&self.my_adp_path, "GeomFill_Pipe::Perform", "myAdpPath")?;
        let options = SweepOptions {
            continuity: shape.0,
            degmax: shape.1,
            segmax: shape.2,
            ..SweepOptions::new(tol, tol, 0.0, 0.01)
        };
        app.perform(path.first_parameter()?, path.last_parameter()?, options)?;
        if app.is_done() {
            self.my_surface = Handle::new(sweep_surface(&app)?).cast();
            self.my_error = app.max_error_on_surf()?;
            self.my_status = GeomFillPipeError::PipeOk;
        }
        Ok(())
    }

    /// `ApproxSurf(WithParameters)`: the AppSweep approximation of the sections of
    /// `GeomFill_SweepSectionGenerator` between the rails (Standard_ConstructionError
    /// for another kind of pipe).
    fn approx_surf(&mut self, with_parameters: bool) -> Result<(), Failure> {
        if self.my_type != 4 {
            return Err(construction_error("GeomFill_Pipe"));
        }
        let mut section = GeomFillSweepSectionGenerator::with_rails(
            &self.my_adp_path,
            &self.my_adp_first_sect,
            &self.my_adp_last_sect,
            self.my_radius,
        )?;
        section.perform(self.my_polynomial)?;
        let line = GeomFillLine::with_nb_points(section.nb_sections());
        let t3d = precision::approximation();
        let mut app =
            GeomFillAppSweep::new(4, 8, t3d, precision::p_approximation(), 0, with_parameters);
        app.perform_by_pieces(Some(&line), &mut section, 30)?;
        if !app.is_done() {
            return Ok(());
        }
        let surface = GeomBSplineSurface::with_weights(
            app.surf_poles()?,
            app.surf_weights()?,
            (app.surf_u_knots()?, app.surf_u_mults()?),
            (app.surf_v_knots()?, app.surf_v_mults()?),
            (app.u_degree()?, app.v_degree()?),
            (false, false),
        )?;
        self.my_surface = Handle::new(surface).cast();
        self.my_error = app.tol_reached()?.0;
        self.my_status = GeomFillPipeError::PipeOk;
        Ok(())
    }
}
