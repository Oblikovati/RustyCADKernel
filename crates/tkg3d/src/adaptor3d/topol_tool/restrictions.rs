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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_TopolTool.cxx (Initialize(S), Initialize(C)) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The restrictions of the parametric rectangle and the vertices of one restriction.
use super::super::h_vertex::Adaptor3dHVertex;
use super::super::surface::Adaptor3dSurface;
use super::{Adaptor3dTopolTool, TopolState, samples};
use crate::geom::errors::null_object;
use crate::top_abs::Orientation;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dLine2d};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Point2};

const INFINITE: f64 = precision::infinite();

// The parameter range of one restriction over [first, last] (possibly infinite):
// its length is capped at 2 * Precision::Infinite().
fn clamped(first: f64, last: f64) -> (f64, f64) {
    let delta: f64 = (last - first).min(2. * INFINITE);
    if first >= -INFINITE {
        return (first, first + delta);
    }
    if last <= INFINITE {
        return (last - delta, last);
    }
    (-INFINITE, INFINITE)
}

impl Adaptor3dTopolTool {
    /// Build the restrictions of `surface`: the finite sides of its rectangle, in the
    /// order V = Vinf, U = Usup, V = Vsup, U = Uinf, plus the apex line of a cone
    /// with only two sides (`Initialize(S)`); `tool.initialize_surface(&surface)?`.
    pub fn initialize_surface(
        &self,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        let Some(adapted) = surface.get() else {
            return Err(null_object(
                "Adaptor3d_TopolTool::Initialize: null surface; expected a surface",
            ));
        };
        // The sample count is reset before the bounds are read, as natively.
        self.state.write().nb_samples_u = -1;
        let bounds = (
            adapted.first_u_parameter()?,
            adapted.last_u_parameter()?,
            adapted.first_v_parameter()?,
            adapted.last_v_parameter()?,
        );
        let mut state = self.state.write();
        (state.uinf, state.vinf, state.usup, state.vsup) = (bounds.0, bounds.2, bounds.1, bounds.3);
        state.nb_restr = 0;
        state.id_restr = 0;
        add_sides(&mut state);
        state.surface = surface.clone();
        if state.nb_restr == 2 && adapted.get_type()? == SurfaceType::Cone {
            let (u, v) = samples::cone_apex_param(&adapted.cone()?);
            let (pinf, psup) = clamped(state.uinf, state.usup);
            push(&mut state, Point2::new(u, v), Direction2::X, (pinf, psup));
        }
        Ok(())
    }
    /// Build the vertices of a restriction: FORWARD at a finite first parameter,
    /// REVERSED at a finite last one, resolution 1e-8 (`Initialize(C)`);
    /// `tool.initialize_curve(&restriction)?`.
    pub fn initialize_curve(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<(), Failure> {
        let Some(restriction) = curve.get() else {
            return Err(null_object(
                "Adaptor3d_TopolTool::Initialize: null curve; expected a curve",
            ));
        };
        let mut vertices: Vec<Adaptor3dHVertex> = Vec::new();
        let (first, last) = (
            restriction.first_parameter()?,
            restriction.last_parameter()?,
        );
        if first > -INFINITE {
            vertices.push(Adaptor3dHVertex::new(
                restriction.value(first)?,
                Orientation::Forward,
                1.0e-8,
            ));
        }
        if last < INFINITE {
            vertices.push(Adaptor3dHVertex::new(
                restriction.value(last)?,
                Orientation::Reversed,
                1.0e-8,
            ));
        }
        let mut state = self.state.write();
        state.nb_vtx = 0;
        state.id_vtx = 0;
        for vertex in vertices {
            let index = state.nb_vtx as usize;
            state.vtx[index] = Handle::new(vertex);
            state.nb_vtx += 1;
        }
        Ok(())
    }
}

// The four sides that are finite, each a line of the rectangle's boundary.
fn add_sides(state: &mut TopolState) {
    let (uinf, usup, vinf, vsup) = (state.uinf, state.usup, state.vinf, state.vsup);
    if !precision::is_negative_infinite(vinf) {
        push(
            state,
            Point2::new(0., vinf),
            Direction2::X,
            clamped(uinf, usup),
        );
    }
    if !precision::is_positive_infinite(usup) {
        push(
            state,
            Point2::new(usup, 0.),
            Direction2::Y,
            clamped(vinf, vsup),
        );
    }
    if !precision::is_positive_infinite(vsup) {
        push(
            state,
            Point2::new(0., vsup),
            Direction2::NEG_X,
            clamped(-usup, -uinf),
        );
    }
    if !precision::is_negative_infinite(uinf) {
        push(
            state,
            Point2::new(uinf, 0.),
            Direction2::NEG_Y,
            clamped(-vsup, -vinf),
        );
    }
}

fn push(state: &mut TopolState, point: Point2, direction: Direction2, (pinf, psup): (f64, f64)) {
    let index = state.nb_restr as usize;
    state.restr[index] = Handle::new(Adaptor2dLine2d::new(point, direction, pinf, psup));
    state.nb_restr += 1;
}
