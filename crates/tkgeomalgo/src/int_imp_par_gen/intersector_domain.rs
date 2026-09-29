// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of IntImpParGen_Intersector.gxx (And_Domaine_Objet1_Intersections
// and the domain ends of Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The implicit domain in `IntImpParGen_Intersector`: clipping the null intervals
//! to it and adding the domain ends that meet.
use super::intersector::{Intersector, ParCurveOf, PerformInput, PerformState, is_on_domain};
use super::intersector_segments::{get, set};
use super::traits::{ImpTool, IntersectorTypes, ParTool};
use crate::int_res2d::{Domain, IntersectionPoint, Position};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::gp::Point2;

/// The input tables of And_Domaine_Objet1_Intersections: the interval bounds as
/// parameters of the parametric curve (`Inter2_And_Domain2`) and of the implicit
/// one (`Inter1`).
pub type IntervalBounds<'t, 'a> = (&'t mut Array1<'a, f64>, &'t mut Array1<'a, f64>);
/// The output tables: the result bounds on the implicit (`Resultat1`) and
/// parametric (`Resultat2`) curves.
pub type ResultBounds<'t, 'a> = (&'t mut Array1<'a, f64>, &'t mut Array1<'a, f64>);

impl<T: IntersectorTypes> Intersector<T> {
    /// `And_Domaine_Objet1_Intersections(ImpTool, ParCurve, ImpDomain, ParDomain,
    /// NbResultats, Inter2_And_Domain2, Inter1, Resultat1, Resultat2, EpsNul)`: clips
    /// the `nb_resultats` interval bounds to the implicit domain (a bound beyond the
    /// domain, out of tolerance, is replaced by the domain bound and its parameter on
    /// the parametric curve found by FindV) and sets `nb_resultats` to the number of
    /// result bounds;
    /// `inter.and_domaine_objet1_intersections(&itool, &pconic, &d1, &d2, &mut n, (&mut i2, &mut i1), (&mut r1, &mut r2), 1e-9)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn and_domaine_objet1_intersections(
        &self,
        imp_tool: &T::Imp,
        par_curve: &ParCurveOf<T>,
        imp_domain: &Domain,
        par_domain: &Domain,
        nb_resultats: &mut i32,
        (inter2, inter1): IntervalBounds<'_, '_>,
        (resultat1, resultat2): ResultBounds<'_, '_>,
        eps_nul: f64,
    ) -> Result<(), Failure> {
        let nb_bornes_intersection: i32 = *nb_resultats;
        *nb_resultats = 0;
        let mut out = ClipOutput {
            count: nb_resultats,
            resultat1,
            resultat2,
        };
        for i in (1..=nb_bornes_intersection).step_by(2) {
            let (mut param1, mut param2) = (get(inter1, i)?, get(inter1, i + 1)?);
            let (mut indice_1, mut indice_2) = (i, i + 1);
            if param1 > param2 {
                (param1, param2) = (param2, param1);
                (indice_1, indice_2) = (i + 1, i);
            }
            let on1: bool = is_on_domain(imp_domain, param1, imp_tool.value(param1))?;
            let on2: bool = is_on_domain(imp_domain, param2, imp_tool.value(param2))?;
            let (v0, v1) = (get(inter2, indice_1)?, get(inter2, indice_2)?);
            let clip = ClipContext {
                imp_tool,
                par_curve,
                par_domain,
                v: (v0, v1),
                eps_nul,
            };
            if on1 {
                out.push(get(inter1, indice_1)?, v0)?;
                if on2 {
                    out.push(get(inter1, indice_2)?, v1)?;
                } else {
                    let t: f64 = imp_domain.last_parameter()?;
                    out.push(t, self.clip_v(&clip, t)?)?;
                }
            } else if on2 {
                let t: f64 = imp_domain.first_parameter()?;
                out.push(t, self.clip_v(&clip, t)?)?;
                out.push(get(inter1, indice_2)?, v1)?;
            } else if param1 < imp_domain.first_parameter()?
                && param2 > imp_domain.last_parameter()?
            {
                let t: f64 = imp_domain.first_parameter()?;
                out.push(t, self.clip_v(&clip, t)?)?;
                let t: f64 = imp_domain.last_parameter()?;
                out.push(t, self.clip_v(&clip, t)?)?;
            }
        }
        Ok(())
    }

    // `FindV` at a bound `t` of the implicit domain, between the two parameters of
    // the interval on the parametric curve.
    fn clip_v(&self, clip: &ClipContext<'_, T>, t: f64) -> Result<f64, Failure> {
        let mut pt: Point2 = Point2::default();
        let (v0, v1) = clip.v;
        self.find_v(
            t,
            &mut pt,
            clip.imp_tool,
            clip.par_curve,
            clip.par_domain,
            v0,
            v1,
            clip.eps_nul,
        )
    }
}

// The arguments of the FindV calls of one interval.
struct ClipContext<'c, T: IntersectorTypes> {
    imp_tool: &'c T::Imp,
    par_curve: &'c ParCurveOf<T>,
    par_domain: &'c Domain,
    v: (f64, f64),
    eps_nul: f64,
}

// The result tables and their count.
struct ClipOutput<'o, 'a> {
    count: &'o mut i32,
    resultat1: &'o mut Array1<'a, f64>,
    resultat2: &'o mut Array1<'a, f64>,
}

impl ClipOutput<'_, '_> {
    fn push(&mut self, u: f64, v: f64) -> Result<(), Failure> {
        *self.count += 1;
        set(self.resultat1, *self.count, u)?;
        set(self.resultat2, *self.count, v)
    }
}

impl<T: IntersectorTypes> Intersector<T> {
    // The last step of Perform: each pair of domain ends not already a solution
    // end, closer than the larger of their tolerances, is a point.
    pub(super) fn insert_domain_ends(
        &mut self,
        input: &PerformInput<'_, T>,
        state: &mut PerformState,
    ) -> Result<(), Failure> {
        let (imp, par) = (input.imp_domain, input.par_domain);
        if !state.head_on_imp && imp.has_first_point() {
            let imp_end = (
                Position::Head,
                imp.first_point()?,
                imp.first_parameter()?,
                imp.first_tolerance()?,
            );
            if !state.head_on_par {
                let par_end = (
                    Position::Head,
                    par.first_point()?,
                    par.first_parameter()?,
                    par.first_tolerance()?,
                );
                self.insert_end_pair(input, imp_end, par_end, state)?;
            }
            if !state.end_on_par {
                let par_end = (
                    Position::End,
                    par.last_point()?,
                    par.last_parameter()?,
                    par.last_tolerance()?,
                );
                self.insert_end_pair(input, imp_end, par_end, state)?;
            }
        }
        if !state.end_on_imp && imp.has_last_point() {
            let imp_end = (
                Position::End,
                imp.last_point()?,
                imp.last_parameter()?,
                imp.last_tolerance()?,
            );
            if !state.head_on_par {
                let par_end = (
                    Position::Head,
                    par.first_point()?,
                    par.first_parameter()?,
                    par.first_tolerance()?,
                );
                self.insert_end_pair(input, imp_end, par_end, state)?;
            }
            if !state.end_on_par {
                let par_end = (
                    Position::End,
                    par.last_point()?,
                    par.last_parameter()?,
                    par.last_tolerance()?,
                );
                self.insert_end_pair(input, imp_end, par_end, state)?;
            }
        }
        Ok(())
    }

    // One pair of domain ends `(position, point, parameter, tolerance)`.
    fn insert_end_pair(
        &mut self,
        input: &PerformInput<'_, T>,
        (pos1, point1, param1, tol1): (Position, Point2, f64, f64),
        (pos2, point2, param2, tol2): (Position, Point2, f64, f64),
        state: &mut PerformState,
    ) -> Result<(), Failure> {
        if point1.distance(point2) > std_max(tol1, tol2) {
            return Ok(());
        }
        input
            .imp_tool
            .d2(param1, &mut state.pt1, &mut state.tan1, &mut state.norm1);
        T::Par::d2(
            input.par_curve,
            param2,
            &mut state.pt2,
            &mut state.tan2,
            &mut state.norm2,
        )?;
        state.transition(pos1, pos2, input.tolerance_angulaire);
        let reversed: bool = self.base.reversed_parameters();
        let ip =
            IntersectionPoint::new(point1, param1, param2, state.trans1, state.trans2, reversed);
        self.base.insert(ip)
    }
}
