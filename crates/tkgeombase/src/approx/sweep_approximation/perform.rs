// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SweepApproximation.cxx (Perform, Approximation)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_SweepApproximation::Perform`: the subspaces of the section (weights,
//! 2D curves, 3D poles) with their tolerances, then AdvApprox.
use super::cache::SweepEvaluator;
use super::{SweepApproximation, SweepOptions};
use crate::approx::sweep_function::{SweepD1, SweepD2, SweepFunction};
use crate::failure_capture::FailureCapture;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adv_approx::{
    AdvApproxApproxAFunction, AdvApproxCutting, AdvApproxDichoCutting, AdvApproxPrefAndRec,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{GeneralTransform2, Point2, Point3, Vector3};

/// The tolerances of the three kinds of subspace.
struct SweepTolerances {
    one_d: Array1<'static, f64>,
    two_d: Option<Array1<'static, f64>>,
    three_d: Array1<'static, f64>,
    three_d_min: f64,
    bound_2d: f64,
}

fn one_based<T: Default + Clone>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(|e| array_failure(e, "Approx_SweepApproximation"))
}

fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

impl SweepApproximation {
    /// `Perform(First, Last, Tol3d, BoundTol, Tol2d, TolAngular, Continuity, Degmax,
    /// Segmax)`: approximates the sweep on `[first, last]`; a continuity from C2 is
    /// reduced when D2 (then D1) returns false, and a native D1/D2 that is not
    /// implemented raises; `approx.perform(0.0, 1.0, options)?`.
    pub fn perform(&mut self, first: f64, last: f64, options: SweepOptions) -> Result<(), Failure> {
        let func = self.cache.func.clone();
        let func: &dyn SweepFunction = super::func_ref(&func)?;
        let (nb_pol_sect, nb_knot_sect, udeg) = func.section_shape()?;
        self.udeg = udeg;
        self.cache.num_2d = func.nb_2d_curves()?;
        let mut u_knots: Array1<'static, f64> = one_based(nb_knot_sect)?;
        let mut u_mults: Array1<'static, i32> = one_based(nb_knot_sect)?;
        func.knots(&mut u_knots)?;
        func.mults(&mut u_mults)?;
        (self.tab_u_knots, self.tab_u_mults) = (Some(u_knots), Some(u_mults));
        self.cache.num_3d = nb_pol_sect;
        let tolerances: SweepTolerances = self.tolerances(func, &options)?;
        self.allocate()?;
        let continuity: Shape = self.checked_continuity(func, [first, last], options.continuity)?;
        func.set_tolerance(tolerances.three_d_min / 20., options.tol_2d / 20.)?;
        let cut_tool: Box<dyn AdvApproxCutting> = cutting(func)?;
        self.approximation(
            &tolerances,
            [first, last],
            continuity,
            &options,
            cut_tool.as_ref(),
        )
    }

    // (2.1) and (2.2): the 3D tolerances of the function, the 1D ones of a rational
    // section, and the 2D ones through the affinities of the resolutions.
    fn tolerances(
        &mut self,
        func: &dyn SweepFunction,
        options: &SweepOptions,
    ) -> Result<SweepTolerances, Failure> {
        let num_3d: i32 = self.cache.num_3d;
        let mut one_d: Array1<'static, f64> = one_based(num_3d)?;
        let mut three_d: Array1<'static, f64> = one_based(num_3d)?;
        func.get_tolerance(
            options.bound_tol,
            options.tol_3d,
            options.tol_angular,
            &mut three_d,
        )?;
        let mut three_d_min: f64 = options.tol_3d;
        for ii in 1..=num_3d {
            if three_d[ii] < three_d_min {
                three_d_min = three_d[ii];
            }
        }
        self.cache.num_1d = 0;
        if func.is_rational()? {
            self.cache.num_1d = num_3d;
            let mut wmin: Array1<'static, f64> = one_based(num_3d)?;
            func.get_minimal_weight(&mut wmin)?;
            let size: f64 = func.maximal_section()?;
            let barycentre: Point3 = func.barycentre_of_surf()?;
            self.cache.translation = Vector3::new(barycentre.x(), barycentre.y(), barycentre.z());
            for ii in 1..=num_3d {
                let mut tol: f64 = three_d[ii] / 2.;
                one_d[ii] = tol * wmin[ii] / size;
                tol *= wmin[ii];
                three_d[ii] = std_max(tol, 1.0e-20);
            }
        }
        let (two_d, bound_2d) = self.two_d_tolerances(func, options)?;
        Ok(SweepTolerances {
            one_d,
            two_d,
            three_d,
            three_d_min,
            bound_2d,
        })
    }

    fn two_d_tolerances(
        &mut self,
        func: &dyn SweepFunction,
        options: &SweepOptions,
    ) -> Result<(Option<Array1<'static, f64>>, f64), Failure> {
        let num_2d: i32 = self.cache.num_2d;
        if num_2d == 0 {
            return Ok((None, 0.0));
        }
        let mut two_d: Array1<'static, f64> = one_based(num_2d)?;
        let mut affin: Array1<'static, GeneralTransform2> = one_based(num_2d)?;
        let the_3d_2d_tol: f64 = 0.9 * options.bound_tol;
        for ii in 1..=num_2d {
            let (tolu, tolv) = func.resolution(ii, the_3d_2d_tol)?;
            let res: f64 = if tolu > tolv {
                affin[ii].set_value(1, 1, tolv / tolu)?;
                tolv
            } else {
                affin[ii].set_value(2, 2, tolu / tolv)?;
                tolu
            };
            two_d[ii] = std_min(options.tol_2d, res);
        }
        self.cache.a_affin = affin;
        Ok((Some(two_d), the_3d_2d_tol))
    }

    // (3) Init: the section arrays of the evaluation (empty 2D arrays without 2D
    // curves).
    fn allocate(&mut self) -> Result<(), Failure> {
        let (num_2d, num_3d) = (self.cache.num_2d, self.cache.num_3d);
        let c = &mut self.cache;
        (c.poles, c.d_poles, c.d2_poles) =
            (one_based(num_3d)?, one_based(num_3d)?, one_based(num_3d)?);
        (c.weights, c.d_weights, c.d2_weights) =
            (one_based(num_3d)?, one_based(num_3d)?, one_based(num_3d)?);
        let count_2d: i32 = num_2d.max(0);
        (c.poles2d, c.d_poles2d, c.d2_poles2d) = (
            one_based(count_2d)?,
            one_based(count_2d)?,
            one_based(count_2d)?,
        );
        self.con_surf_err = Some(one_based(count_2d)?);
        Ok(())
    }

    // Whether D2 (from C2) and D1 (at C1) are available: a false answer lowers the
    // continuity by one step.
    fn checked_continuity(
        &mut self,
        func: &dyn SweepFunction,
        [first, last]: [f64; 2],
        mut continuity: Shape,
    ) -> Result<Shape, Failure> {
        let c = &mut self.cache;
        if continuity >= Shape::C2 {
            let values = SweepD2 {
                d1: SweepD1 {
                    poles: &mut c.poles,
                    d_poles: &mut c.d_poles,
                    poles2d: &mut c.poles2d,
                    d_poles2d: &mut c.d_poles2d,
                    weights: &mut c.weights,
                    d_weights: &mut c.d_weights,
                },
                d2_poles: &mut c.d2_poles,
                d2_poles2d: &mut c.d2_poles2d,
                d2_weights: &mut c.d2_weights,
            };
            if !func.d2(first, first, last, values)? {
                continuity = Shape::C1;
            }
        }
        if continuity == Shape::C1 {
            let values = SweepD1 {
                poles: &mut c.poles,
                d_poles: &mut c.d_poles,
                poles2d: &mut c.poles2d,
                d_poles2d: &mut c.d_poles2d,
                weights: &mut c.weights,
                d_weights: &mut c.d_weights,
            };
            if !func.d1(first, first, last, values)? {
                continuity = Shape::C0;
            }
        }
        Ok(continuity)
    }

    // `Approximation(...)`: AdvApprox, then the surface poles and weights, the 2D
    // curves and the errors.
    fn approximation(
        &mut self,
        tolerances: &SweepTolerances,
        [first, last]: [f64; 2],
        continuity: Shape,
        options: &SweepOptions,
        cut_tool: &dyn AdvApproxCutting,
    ) -> Result<(), Failure> {
        let (num_1d, num_2d, num_3d) = (self.cache.num_1d, self.cache.num_2d, self.cache.num_3d);
        let mut evaluator = SweepEvaluator {
            cache: &mut self.cache,
            capture: FailureCapture::default(),
        };
        let approx = AdvApproxApproxAFunction::with_cutting(
            num_1d,
            num_2d,
            num_3d,
            Some(&tolerances.one_d),
            tolerances.two_d.as_ref(),
            Some(&tolerances.three_d),
            first,
            last,
            continuity,
            options.degmax,
            options.segmax,
            &mut evaluator,
            cut_tool,
        );
        evaluator.capture.take()?;
        let approx: AdvApproxApproxAFunction = approx?;
        self.done = approx.has_result();
        if !self.done {
            return Ok(());
        }
        self.store_surface(&approx)?;
        self.store_curves_2d(&approx)?;
        self.store_errors(&approx, tolerances)
    }

    fn store_surface(&mut self, approx: &AdvApproxApproxAFunction) -> Result<(), Failure> {
        self.vdeg = approx.degree();
        let nb_poles: i32 = approx.nb_poles()?;
        let (num_1d, num_3d) = (self.cache.num_1d, self.cache.num_3d);
        let failure = |e| array_failure(e, "Approx_SweepApproximation::Approximation");
        let mut poles: Array2<'static, Point3> =
            Array2::with_bounds(1, num_3d, 1, nb_poles).map_err(failure)?;
        let mut weights: Array2<'static, f64> =
            Array2::with_bounds(1, num_3d, 1, nb_poles).map_err(failure)?;
        let approx_poles = approx.poles().ok_or_else(|| super::null_table("Poles"))?;
        if num_1d == num_3d {
            let poles_1d = approx
                .poles_1d()
                .ok_or_else(|| super::null_table("Poles1d"))?;
            let t: Vector3 = self.cache.translation;
            for ii in 1..=num_3d {
                for jj in 1..=nb_poles {
                    let p: Point3 = *approx_poles.value(jj, ii).map_err(failure)?;
                    let w: f64 = *poles_1d.value(jj, ii).map_err(failure)?;
                    let p = Point3::new(p.x() / w + t.x(), p.y() / w + t.y(), p.z() / w + t.z());
                    poles.set_value(ii, jj, p).map_err(failure)?;
                    weights.set_value(ii, jj, w).map_err(failure)?;
                }
            }
        } else {
            weights.init(1.0);
            for ii in 1..=num_3d {
                for jj in 1..=nb_poles {
                    poles
                        .set_value(ii, jj, *approx_poles.value(jj, ii).map_err(failure)?)
                        .map_err(failure)?;
                }
            }
        }
        (self.tab_poles, self.tab_weights) = (Some(poles), Some(weights));
        self.tab_v_knots = approx.knots().cloned();
        self.tab_v_mults = approx.multiplicities().cloned();
        Ok(())
    }

    // The 2D curves, back through the inverse affinities; appended to the curves of
    // a previous Perform (the sequence is never cleared natively).
    fn store_curves_2d(&mut self, approx: &AdvApproxApproxAFunction) -> Result<(), Failure> {
        if self.cache.num_2d <= 0 {
            return Ok(());
        }
        self.deg2d = self.vdeg;
        self.tab_2d_knots = approx.knots().cloned();
        self.tab_2d_mults = approx.multiplicities().cloned();
        let nb_poles: i32 = approx.nb_poles()?;
        for ii in 1..=self.cache.num_2d {
            let inverse: GeneralTransform2 = self.cache.a_affin[ii].inverted()?;
            let mut p2d: Array1<'static, Point2> = one_based(nb_poles)?;
            approx.poles_2d_of(ii, &mut p2d)?;
            for jj in 1..=nb_poles {
                let mut coord = p2d[jj].xy();
                inverse.transforms(&mut coord);
                p2d[jj] = Point2::new(coord.x(), coord.y());
            }
            self.seq_poles_2d.push(p2d);
        }
        Ok(())
    }

    fn store_errors(
        &mut self,
        approx: &AdvApproxApproxAFunction,
        tolerances: &SweepTolerances,
    ) -> Result<(), Failure> {
        let (num_1d, num_2d, num_3d) = (self.cache.num_1d, self.cache.num_2d, self.cache.num_3d);
        let (mut m3, mut a3) = (one_based(num_3d)?, one_based(num_3d)?);
        for ii in 1..=num_3d {
            (m3[ii], a3[ii]) = (approx.max_error(3, ii)?, approx.average_error(3, ii)?);
        }
        (self.m_error_3d, self.a_error_3d) = (Some(m3), Some(a3));
        if super::func_ref(&self.cache.func)?.is_rational()? {
            let (mut m1, mut a1) = (one_based(num_3d)?, one_based(num_3d)?);
            for ii in 1..=num_1d {
                (m1[ii], a1[ii]) = (approx.max_error(1, ii)?, approx.average_error(1, ii)?);
            }
            (self.m_error_1d, self.a_error_1d) = (Some(m1), Some(a1));
        }
        if num_2d <= 0 {
            return Ok(());
        }
        let two_d = tolerances
            .two_d
            .as_ref()
            .ok_or_else(|| super::null_table("TwoDTol"))?;
        let (mut m2, mut a2) = (one_based(num_2d)?, one_based(num_2d)?);
        let con_surf = self
            .con_surf_err
            .as_mut()
            .ok_or_else(|| super::null_table("COnSurfErr"))?;
        for ii in 1..=num_2d {
            (m2[ii], a2[ii]) = (approx.max_error(2, ii)?, approx.average_error(2, ii)?);
            con_surf[ii] = (m2[ii] / two_d[ii]) * tolerances.bound_2d;
        }
        (self.tab_2d_error, self.ave_2d_error) = (Some(m2), Some(a2));
        Ok(())
    }
}

// Preferred cuts at the C2 and C3 breaks when the path has several C3 spans, else
// dichotomic cutting.
fn cutting(func: &dyn SweepFunction) -> Result<Box<dyn AdvApproxCutting>, Failure> {
    let nb_c2: i32 = func.nb_intervals(Shape::C2)?;
    let nb_c3: i32 = func.nb_intervals(Shape::C3)?;
    if nb_c3 <= 1 {
        return Ok(Box::new(AdvApproxDichoCutting::new()));
    }
    let mut c2: Array1<'static, f64> = one_based(nb_c2 + 1)?;
    func.intervals(&mut c2, Shape::C2)?;
    let mut c3: Array1<'static, f64> = one_based(nb_c3 + 1)?;
    func.intervals(&mut c3, Shape::C3)?;
    Ok(Box::new(AdvApproxPrefAndRec::new(&c2, &c3, 5.0)?))
}
