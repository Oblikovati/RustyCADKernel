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
// Rust translation of the IntPatch_PrmPrmIntersection::Perform overloads on two polyhedra
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Perform` overloads on the polyhedra of the two surfaces: the walkings start
//! from the section lines (longest first) and the tangent zones of their interference.
use super::IntPatchPrmPrmIntersection;
use super::helpers::{section_point_to_parameters, widen_bounds};
use super::start_params;
use super::walk_steps::{
    AcceptOptions, EndsCheck, WalkInputs, duplicated_line, is_new_start, keep_rejected,
};
use crate::int_patch::{IntPatchInterferencePolyhedron, IntPatchPolyhedron};
use crate::int_start::TheTopolTool;
use crate::int_walk::IntWalkPWalking;
use crate::intf::{SectionLine, SectionPoint};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// The tolerances of the marching: `TolTangency`, `Epsilon`, `Deflection` and
/// `Increment`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntPatchPrmPrmTolerances {
    /// `TolTangency`.
    pub tol_tangency: f64,
    /// `Epsilon`.
    pub epsilon: f64,
    /// `Deflection`.
    pub deflection: f64,
    /// `Increment`.
    pub increment: f64,
}

/// The walking over the interference of two polyhedra; `uv` keeps the parameters of the
/// last section point read, as the native locals do.
struct PolyWalk<'a> {
    inputs: WalkInputs<'a>,
    polys: (&'a IntPatchPolyhedron, &'a IntPatchPolyhedron),
    pw: IntWalkPWalking,
    uv: [f64; 4],
    /// The native `UminLig1, VminLig1, UminLig2, VminLig2`, which the first point of a
    /// section line is read into.
    first_uv: [f64; 4],
}

impl PolyWalk<'_> {
    /// The parameters of a section point on both polyhedra.
    fn parameters(&mut self, sp: &SectionPoint) -> Result<[f64; 4], Failure> {
        section_point_to_parameters(sp, self.polys.0, self.polys.1, &mut self.uv)?;
        Ok(self.uv)
    }
}

impl IntPatchPrmPrmIntersection {
    /// `Perform(Caro1, Polyhedron1, Domain1, Caro2, Polyhedron2, Domain2, TolTangency,
    /// Epsilon, Deflection, Increment)`: the intersection from the interference of the
    /// given polyhedra of the two surfaces.
    ///
    /// ```ignore
    /// inter.perform_polyhedra((&s1, &p1, &*d1), (&s2, &p2, &*d2), tolerances)?;
    /// ```
    pub fn perform_polyhedra(
        &mut self,
        (surf1, poly1, d1): (
            &Adaptor3dSurfaceHandle,
            &IntPatchPolyhedron,
            &(dyn TheTopolTool + 'static),
        ),
        (surf2, poly2, d2): (
            &Adaptor3dSurfaceHandle,
            &IntPatchPolyhedron,
            &(dyn TheTopolTool + 'static),
        ),
        tolerances: IntPatchPrmPrmTolerances,
    ) -> Result<(), Failure> {
        let interference = IntPatchInterferencePolyhedron::between(poly1, poly2)?;
        self.empt = true;
        self.done = true;
        self.slin.clear();
        let t = tolerances;
        let pw = IntWalkPWalking::new(
            surf1,
            surf2,
            t.tol_tangency,
            t.epsilon,
            t.deflection,
            t.increment,
        )?;
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
        let mut walk = PolyWalk {
            inputs,
            polys: (poly1, poly2),
            pw,
            uv: [0.0; 4],
            first_uv: [0.0; 4],
        };
        let nb_lig_sec: i32 = interference.nb_section_lines();
        if nb_lig_sec >= 1 {
            for ls in sorted_section_lines(&interference)? {
                self.walk_poly_section(&mut walk, interference.line_value(ls)?)?;
            }
        }
        self.walk_poly_zones(&mut walk, &interference)
    }

    /// `Perform(Caro1, Polyhedron1, Domain1, Caro2, Domain2, ...)`: the polyhedron of
    /// the second surface is computed (`IntPatch_Polyhedron(Caro2)`).
    ///
    /// ```ignore
    /// inter.perform_polyhedron1((&s1, &p1, &*d1), (&s2, &*d2), tolerances)?;
    /// ```
    pub fn perform_polyhedron1(
        &mut self,
        (surf1, poly1, d1): (
            &Adaptor3dSurfaceHandle,
            &IntPatchPolyhedron,
            &(dyn TheTopolTool + 'static),
        ),
        (surf2, d2): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        tolerances: IntPatchPrmPrmTolerances,
    ) -> Result<(), Failure> {
        let poly2 = IntPatchPolyhedron::new(surf2)?;
        self.perform_polyhedra((surf1, poly1, d1), (surf2, &poly2, d2), tolerances)
    }

    /// `Perform(Caro1, Domain1, Caro2, Polyhedron2, Domain2, ...)`: the polyhedron of the
    /// first surface is computed (`IntPatch_Polyhedron(Caro1)`).
    ///
    /// ```ignore
    /// inter.perform_polyhedron2((&s1, &*d1), (&s2, &p2, &*d2), tolerances)?;
    /// ```
    pub fn perform_polyhedron2(
        &mut self,
        (surf1, d1): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        (surf2, poly2, d2): (
            &Adaptor3dSurfaceHandle,
            &IntPatchPolyhedron,
            &(dyn TheTopolTool + 'static),
        ),
        tolerances: IntPatchPrmPrmTolerances,
    ) -> Result<(), Failure> {
        let poly1 = IntPatchPolyhedron::new(surf1)?;
        self.perform_polyhedra((surf1, &poly1, d1), (surf2, poly2, d2), tolerances)
    }

    /// The marching from one section line of the interference.
    fn walk_poly_section(
        &mut self,
        walk: &mut PolyWalk<'_>,
        line: &SectionLine,
    ) -> Result<(), Failure> {
        let nbp: i32 = line.number_of_points();
        let mut tab_pt_dep: Vec<bool> = vec![false; (nbp + 1).max(0) as usize];
        section_point_to_parameters(
            line.get_point(1)?,
            walk.polys.0,
            walk.polys.1,
            &mut walk.first_uv,
        )?;
        let first: [f64; 4] = walk.first_uv;
        let (mut min, mut max) = (first, first);
        for ilig in 2..=nbp {
            let uv: [f64; 4] = walk.parameters(line.get_point(ilig)?)?;
            widen_bounds((&mut min, &mut max), uv);
        }
        // The native minima are the variables the next line reads its first point into.
        walk.first_uv = min;
        let mut state = StartChoice::new(nbp);
        let mut lignetrouvee = false;
        loop {
            let nbps2: i32 = state.next(line)?;
            if !tab_pt_dep[nbps2 as usize] {
                tab_pt_dep[nbps2 as usize] = true;
                let uv: [f64; 4] = walk.parameters(line.get_point(nbps2)?)?;
                lignetrouvee |= self.walk_poly_start(walk, uv, (min, max), EndsCheck::Both)?;
            }
            let n: i32 = state.n;
            if !(nbp > 5 && ((n < 3) || !lignetrouvee) && ((n - 3 < nbp) || lignetrouvee)) {
                return Ok(());
            }
        }
    }

    /// The walking from the start parameters `uv`, in the bounds `(min, max)`; returns
    /// whether a line was added.
    fn walk_poly_start(
        &mut self,
        walk: &mut PolyWalk<'_>,
        uv: [f64; 4],
        (min, max): ([f64; 4], [f64; 4]),
        ends: EndsCheck,
    ) -> Result<bool, Failure> {
        let start = start_params(uv)?;
        let Some(start_pon2s) = walk.pw.perform_first_point(&start)? else {
            return Ok(false);
        };
        let nb_lig_calculee: i32 = self.slin.len() as i32;
        if !is_new_start(&walk.inputs, &start_pon2s, &self.slin)? {
            return Ok(false);
        }
        walk.pw.perform_bounded(&start, min, max)?;
        if !walk.pw.is_done() || walk.pw.nb_points()? <= 2 {
            return Ok(false);
        }
        let pw: &IntWalkPWalking = &walk.pw;
        let points = (pw.value(1)?.value(), pw.value(pw.nb_points()?)?.value());
        let duplicated =
            duplicated_line(&walk.inputs, pw, &self.slin, (nb_lig_calculee, true, ends))?;
        if keep_rejected(pw, duplicated, &mut self.slin)? {
            return Ok(false);
        }
        let options = AcceptOptions {
            loose: false,
            purging: None,
            several: None,
        };
        self.accept_walking(&walk.inputs, pw, &options, points)
    }

    /// The marching from the points of the tangent zones, in the bounds of all of them.
    fn walk_poly_zones(
        &mut self,
        walk: &mut PolyWalk<'_>,
        interference: &IntPatchInterferencePolyhedron,
    ) -> Result<(), Failure> {
        let nb_tan_zon: i32 = interference.nb_tangent_zones();
        let (mut min, mut max) = ([f64::MAX; 4], [-f64::MAX; 4]);
        for z in 1..=nb_tan_zon {
            let zone = interference.zone_value(z)?;
            for pz in 1..=zone.number_of_points() {
                let uv: [f64; 4] = walk.parameters(zone.get_point(pz)?)?;
                widen_bounds((&mut min, &mut max), uv);
            }
        }
        for z in 1..=nb_tan_zon {
            let zone = interference.zone_value(z)?;
            for pz in 1..=zone.number_of_points() {
                let uv: [f64; 4] = walk.parameters(zone.get_point(pz)?)?;
                self.walk_poly_start(walk, uv, (min, max), EndsCheck::Either)?;
            }
        }
        Ok(())
    }
}

/// The choice of the start points of a section line (`NombreDePointsDeDepartDuCheminement`
/// and the start indices): the point of least incidence around the middle, then the
/// first (or middle) one, then the last one (or the middle of the first two), then
/// every point.
struct StartChoice {
    nbp: i32,
    n: i32,
    nbps2: i32,
    first: i32,
    second: i32,
}

impl StartChoice {
    fn new(nbp: i32) -> Self {
        Self {
            nbp,
            n: 0,
            nbps2: if nbp > 3 { nbp / 2 } else { 1 },
            first: 0,
            second: 0,
        }
    }

    /// The next start point.
    fn next(&mut self, line: &SectionLine) -> Result<i32, Failure> {
        self.n += 1;
        let nbp: i32 = self.nbp;
        match self.n {
            1 => {
                self.least_incidence(line)?;
                if nbp < 3 {
                    self.n = 3;
                }
                self.first = self.nbps2;
            }
            2 => {
                self.nbps2 = if self.first == 1 { nbp / 2 } else { 1 };
                self.second = self.nbps2;
            }
            3 => {
                self.nbps2 = if self.first == nbp {
                    (self.first + self.second) / 2
                } else {
                    nbp
                };
            }
            _ => {
                self.nbps2 = self.n - 3;
                self.n += 1;
            }
        }
        Ok(self.nbps2)
    }

    /// The point of least incidence between the quarters of the line, from the middle
    /// outward (each better point must be 10 % better).
    fn least_incidence(&mut self, line: &SectionLine) -> Result<(), Failure> {
        let nbp: i32 = self.nbp;
        let mut incidence: f64 = 3.0;
        let (nbp1_4, nbp3_4) = (nbp / 4, nbp - nbp / 4);
        let ranks = (nbp / 2..nbp3_4).chain(((nbp1_4 + 1)..=nbp / 2).rev());
        for nsp in ranks {
            let current: f64 = line.get_point(nsp)?.incidence();
            if current < incidence {
                self.nbps2 = nsp;
                incidence = 0.9 * current;
            }
        }
        Ok(())
    }
}

/// The section lines of the interference sorted by decreasing number of points (the
/// native bubble sort).
fn sorted_section_lines(
    interference: &IntPatchInterferencePolyhedron,
) -> Result<Vec<i32>, Failure> {
    let mut tab_l: Vec<i32> = (1..=interference.nb_section_lines()).collect();
    loop {
        let mut triok = true;
        for b in 1..tab_l.len() {
            let nb_b: i32 = interference.line_value(tab_l[b])?.number_of_points();
            let nb_a: i32 = interference.line_value(tab_l[b - 1])?.number_of_points();
            if nb_b > nb_a {
                tab_l.swap(b, b - 1);
                triok = false;
            }
        }
        if triok {
            return Ok(tab_l);
        }
    }
}
