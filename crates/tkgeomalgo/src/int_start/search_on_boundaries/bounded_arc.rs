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
// Rust translation of BoundedArc of
// ModelingAlgorithms/TKGeomAlgo/IntStart/IntStart_SearchOnBoundaries.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BoundedArc`: the solution points and the solution parts of one arc with a closed
//! parameter range, from the exact curve-quadric intersection when the arc is a conic,
//! else from `math_FunctionAllRoots`.
use super::treat_lc::{is_degenerated_curve, is_degenerated_quadric, is_regularity};
use super::{points, value_or_keep};
use crate::int_curve_surface::HInter;
use crate::int_start::tools::{SearchOnBoundariesTypes, TheArc, TheArcFunction, TheSegment};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{FunctionAllRoots, FunctionSample};

/// The arc under search and the lists the search appends to.
pub(super) struct BoundedArc<'a, T: SearchOnBoundariesTypes> {
    pub(super) arc: &'a TheArc,
    pub(super) domain: &'a T::TopolTool,
    /// `[Pdeb, Pfin]`.
    pub(super) range: [f64; 2],
    pub(super) pnt: &'a mut Vec<T::PathPoint>,
    pub(super) seg: &'a mut Vec<T::Segment>,
    pub(super) tol_boundary: f64,
    pub(super) tol_tangency: f64,
}

/// The roots of the function on the arc: from the exact intersection (`IsIntCSdone`) or
/// from `math_FunctionAllRoots` (`pSol`).
pub(super) struct ArcRoots {
    pub(super) eps_x: f64,
    pub(super) maxdist: f64,
    pub(super) int_cs: HInter,
    pub(super) is_int_cs_done: bool,
    pub(super) p_sol: Option<FunctionAllRoots>,
    pub(super) nbp: i32,
    pub(super) nbi: i32,
    pub(super) type_con_s: CurveType,
}

impl<T: SearchOnBoundariesTypes> BoundedArc<'_, T> {
    /// `BoundedArc(...)`: returns `Arcsol`, whether the whole arc is solution.
    pub(super) fn run(
        &mut self,
        func: &mut T::Function,
        recheck_on_regularity: bool,
    ) -> Result<bool, Failure> {
        let [pdeb, pfin] = self.range;
        // The tolerance of the arc (EpsX ~ 1e-5) is not coherent with the walking
        // (ResolutionU and V ~ 1e-9): a vertex found here is not found again as a stop
        // point of a walking line.
        let mut eps_x: f64 = 1.0e-10;
        // lbr, 22 April 96: at least 100 samples.
        let nb_echant: i32 = func.nb_samples()?.max(100);
        // Modification of 24 August 93.
        let mut n_tol_tangency: f64 = self.tol_tangency;
        if (pfin - pdeb) < (self.tol_tangency * 10.0) {
            n_tol_tangency = (pfin - pdeb) * 0.1;
        }
        if eps_x > (n_tol_tangency + n_tol_tangency) {
            eps_x = n_tol_tangency * 0.1;
        }
        if self.rejection(func)? {
            return Ok(false);
        }
        let quadric: IntSurfQuadric = *func.quadric();
        let echant = FunctionSample::new(pdeb, pfin, nb_echant);
        let maxdist: f64 = self.max_distance(func, &echant, nb_echant)?;
        let mut roots = ArcRoots {
            eps_x,
            maxdist,
            int_cs: HInter::new(),
            is_int_cs_done: false,
            p_sol: None,
            nbp: 0,
            nbi: 0,
            type_con_s: CurveType::OtherCurve,
        };
        if quadric.type_quadric() != SurfaceType::OtherSurface {
            self.exact_intersection(func, &quadric, &mut roots)?;
        }
        if !roots.is_int_cs_done {
            let p_sol = all_roots(func, &echant, eps_x, maxdist)?;
            roots.nbp = p_sol.number_of_points()?;
            roots.p_sol = Some(p_sol);
        }
        // jgv: build the solution on the whole boundary.
        if recheck_on_regularity
            && roots.nbp > 0
            && is_regularity(self.domain)?
            && self.recheck(func, &echant, eps_x)?
        {
            return Ok(true);
        }
        if roots.nbp != 0 {
            self.solution_points(func, &roots, &quadric)?;
        }
        self.segments(func, &roots, recheck_on_regularity)
    }

    /// The rejection of 15 October 98: true when the bounds of the function from six
    /// first-order samples keep one sign.
    fn rejection(&self, func: &mut T::Function) -> Result<bool, Failure> {
        let [pdeb, pfin] = self.range;
        let mut minr: f64 = f64::MAX;
        let mut maxr: f64 = -minr;
        let mut maxdr: f64 = -minr;
        let mut dur: f64 = (pfin - pdeb) * 0.2;
        let mut ur: f64 = pdeb;
        for _ in 1..=6 {
            if let Some((f, mut d)) = func.checked_values(ur)? {
                if d < 0.0 {
                    d = -d;
                }
                d *= dur + dur;
                if d > maxdr {
                    maxdr = d;
                }
                let (lminr, lmaxr) = (f - d, f + d);
                if lminr < minr {
                    minr = lminr;
                }
                if lmaxr > maxr {
                    maxr = lmaxr;
                }
                if minr < 0.0 && maxr > 0.0 {
                    return Ok(false);
                }
            }
            ur += dur;
        }
        dur = 0.001 + maxdr + (maxr - minr) * 0.1;
        minr -= dur;
        maxr += dur;
        Ok(!(minr < 0.0 && maxr > 0.0))
    }

    /// `maxdist`: TolBoundary + TolTangency when the function stays within it on every
    /// sample and it is below 0.01 (NIZNHY-PKV, 12 April 2001), else TolBoundary.
    fn max_distance(
        &self,
        func: &mut T::Function,
        echant: &FunctionSample,
        nb_echant: i32,
    ) -> Result<f64, Failure> {
        let maxdist: f64 = self.tol_boundary + self.tol_tangency;
        let mut aelargir: bool = true;
        let mut i: i32 = 1;
        while i <= nb_echant && aelargir {
            let u: f64 = echant.parameter(i);
            if let Some(dist) = func.checked_value(u)?
                && (dist > maxdist || -dist > maxdist)
            {
                aelargir = false;
            }
            i += 1;
        }
        if !(aelargir && maxdist < 0.01) {
            return Ok(self.tol_boundary);
        }
        Ok(maxdist)
    }

    /// The exact intersection of the arc on the surface with the quadric, for the
    /// canonic curves and the real quadrics other than the torus.
    fn exact_intersection(
        &self,
        func: &mut T::Function,
        quadric: &IntSurfQuadric,
        roots: &mut ArcRoots,
    ) -> Result<(), Failure> {
        let a_surf: Handle<dyn Adaptor3dSurface> = func.surface();
        let con_s = Adaptor3dCurveOnSurface::with_curve(self.arc, &a_surf)?;
        roots.type_con_s = con_s.get_type()?;
        let h_con_s: Handle<Adaptor3dCurveOnSurface> = Handle::new(con_s);
        let ga_hsurf: Handle<dyn Adaptor3dSurface> = quadric_surface(quadric)?;
        let canonic: bool = matches!(
            roots.type_con_s,
            CurveType::Line
                | CurveType::Circle
                | CurveType::Ellipse
                | CurveType::Parabola
                | CurveType::Hyperbola
        );
        if canonic
            && quadric.type_quadric() != SurfaceType::Torus
            && !is_degenerated_curve(&h_con_s)?
            && !is_degenerated_quadric(quadric)?
        {
            let curve: Handle<dyn Adaptor3dCurve> = h_con_s.cast();
            roots.int_cs.perform(&curve, &ga_hsurf)?;
        }
        roots.is_int_cs_done = roots.int_cs.is_done();
        if roots.is_int_cs_done {
            roots.nbp = roots.int_cs.nb_points()?;
            roots.nbi = roots.int_cs.nb_segments()?;
        }
        // Without an intersection the arc may touch the quadric within a tolerance: the
        // sampled search checks it.
        if roots.nbp == 0 && roots.nbi == 0 {
            roots.is_int_cs_done = false;
        }
        Ok(())
    }

    /// The recheck of a regular edge: when the function stays within 5e-3 of zero along
    /// the whole arc, the null intervals at 5e-4 are the solution segments. Returns true
    /// when the arc is solution (the native early return).
    fn recheck(
        &mut self,
        func: &mut T::Function,
        echant: &FunctionSample,
        eps_x: f64,
    ) -> Result<bool, Failure> {
        let the_tol: f64 = 5.0e-4;
        let sol_again: FunctionAllRoots = all_roots(func, echant, eps_x, the_tol)?;
        let nbi_again: i32 = sol_again.number_of_intervals()?;
        if nbi_again <= 0 || !self.on_boundary(func, the_tol * 10.)? {
            return Ok(false);
        }
        let [pdeb, pfin] = self.range;
        for i in 1..=nbi_again {
            let [mut pardeb, mut parfin] = sol_again.interval(i)?;
            if (pardeb - pdeb).abs() <= precision::p_confusion() {
                pardeb = pdeb;
            }
            if (parfin - pfin).abs() <= precision::p_confusion() {
                parfin = pfin;
            }
            let [ideb, ifin] = sol_again.interval_state(i)?;
            let ptdeb: Point3 = func.valpoint(ideb)?;
            let ptfin: Point3 = func.valpoint(ifin)?;
            self.add_segment([(ptdeb, pardeb), (ptfin, parfin)], the_tol)?;
        }
        Ok(true)
    }

    /// Whether the function is within `global_tol` of zero at eleven regular samples.
    fn on_boundary(&self, func: &mut T::Function, global_tol: f64) -> Result<bool, Failure> {
        let [pdeb, pfin] = self.range;
        let nb_samples: i32 = 10;
        let delta: f64 = (pfin - pdeb) / f64::from(nb_samples);
        // The native `aValue` is uninitialized; see `value_or_keep`.
        let mut a_value: f64 = 0.0;
        for i in 0..=nb_samples {
            let a_param: f64 = pdeb + f64::from(i) * delta;
            a_value = value_or_keep(func, a_param, a_value)?;
            if a_value.abs() > global_tol {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// A segment of the arc between two solution points, processed as points with
    /// `tol`.
    pub(super) fn add_segment(
        &mut self,
        [(ptdeb, pardeb), (ptfin, parfin)]: [(Point3, f64); 2],
        tol: f64,
    ) -> Result<(), Failure> {
        let mut newseg: T::Segment = T::Segment::default();
        newseg.set_value(self.arc);
        let ranged: i32 = self.point_process(ptdeb, pardeb, tol)?;
        newseg.set_limit_point(self.path_point(ranged)?, true);
        let rangef: i32 = self.point_process(ptfin, parfin, tol)?;
        newseg.set_limit_point(self.path_point(rangef)?, false);
        self.seg.push(newseg);
        Ok(())
    }

    pub(super) fn point_process(
        &mut self,
        pt: Point3,
        para: f64,
        tol: f64,
    ) -> Result<i32, Failure> {
        points::point_process::<T>(pt, para, (self.arc, self.domain), self.pnt, tol)
    }

    fn path_point(&self, range: i32) -> Result<&T::PathPoint, Failure> {
        crate::int_start::failures::sequence_value(self.pnt, range, "NCollection_Sequence::Value")
    }

    /// The solution segments (the null intervals), then `Arcsol`: one segment that
    /// covers the whole range.
    fn segments(
        &mut self,
        func: &mut T::Function,
        roots: &ArcRoots,
        recheck_on_regularity: bool,
    ) -> Result<bool, Failure> {
        let mut nbi: i32 = roots.nbi;
        if let Some(p_sol) = roots.p_sol.as_ref().filter(|_| !roots.is_int_cs_done) {
            nbi = p_sol.number_of_intervals()?;
        }
        if !recheck_on_regularity && roots.nbp != 0 {
            nbi = 0;
        }
        let (mut pardeb, mut parfin): (f64, f64) = (0., 0.);
        for i in 1..=nbi {
            let ends: [(Point3, f64); 2] = segment_ends(func, roots, i)?;
            (pardeb, parfin) = (ends[0].1, ends[1].1);
            self.add_segment(ends, self.tol_boundary)?;
        }
        let [pdeb, pfin] = self.range;
        Ok(nbi == 1
            && (pardeb - pdeb).abs() < precision::p_confusion()
            && (parfin - pfin).abs() < precision::p_confusion())
    }
}

/// The ends `(point, parameter)` of the solution interval `i`.
fn segment_ends<F: TheArcFunction>(
    func: &mut F,
    roots: &ArcRoots,
    i: i32,
) -> Result<[(Point3, f64); 2], Failure> {
    let Some(p_sol) = roots.p_sol.as_ref().filter(|_| !roots.is_int_cs_done) else {
        let int_seg = roots.int_cs.segment(i)?;
        let (end1, end2) = (int_seg.first_point(), int_seg.second_point());
        return Ok([(end1.pnt(), end1.w()), (end2.pnt(), end2.w())]);
    };
    let [pardeb, parfin] = p_sol.interval(i)?;
    let [ideb, ifin] = p_sol.interval_state(i)?;
    let ptdeb: Point3 = func.valpoint(ideb)?;
    let ptfin: Point3 = func.valpoint(ifin)?;
    Ok([(ptdeb, pardeb), (ptfin, parfin)])
}

/// `math_FunctionAllRoots(Func, Echant, EpsX, tol, tol)`; Standard_Failure when it is
/// not done, as natively.
fn all_roots<F: TheArcFunction>(
    func: &mut F,
    echant: &FunctionSample,
    eps_x: f64,
    tol: f64,
) -> Result<FunctionAllRoots, Failure> {
    let roots = FunctionAllRoots::new(func, echant, [eps_x, tol, tol]);
    func.take_failure()?;
    let roots: FunctionAllRoots = roots?;
    if !roots.is_done() {
        return Err(Failure::new(
            "IntStart_SearchOnBoundaries: math_FunctionAllRoots is not done; expected IsDone()",
        ));
    }
    Ok(roots)
}

/// The Geom surface of the quadric, adapted.
fn quadric_surface(quadric: &IntSurfQuadric) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    let surface: Handle<dyn GeomSurface> = match quadric.type_quadric() {
        SurfaceType::Plane => Handle::new(GeomPlane::from_pln(quadric.plane())).cast(),
        SurfaceType::Cylinder => {
            Handle::new(GeomCylindricalSurface::from_cylinder(quadric.cylinder()?)).cast()
        }
        SurfaceType::Cone => Handle::new(GeomConicalSurface::from_cone(quadric.cone()?)).cast(),
        SurfaceType::Sphere => {
            Handle::new(GeomSphericalSurface::from_sphere(quadric.sphere()?)).cast()
        }
        SurfaceType::Torus => Handle::new(GeomToroidalSurface::from_torus(quadric.torus()?)).cast(),
        _ => Handle::default(),
    };
    Ok(Handle::new(GeomAdaptorSurface::with_surface(&surface)?).cast())
}
