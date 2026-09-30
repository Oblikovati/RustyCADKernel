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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_AppSurf.gxx
// (Perform(Lin, F, NbMaxP)) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppBlend_AppSurf::Perform(Lin, F, NbMaxP)`: the sections scaled into the unit
//! box, cut into pieces ("troncons") of at most about `NbMaxP` sections, each
//! approximated by Bezier curves, all joined into one B-spline.
use super::internal_perform::std_max;
use super::sections::{BoxScale, SectionArrays};
use super::{AppBlendAppSurf, TheLine, TheSectionGenerator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_def::{Compute, MultiLine};
use rustycad_tkgeombase::app_par_curves::{Constraint, MultiCurveEval};
use rustycad_tkgeombase::approx::{MCurvesToBSpCurve, ParametrizationType};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::math::Vector;

/// The state carried from piece to piece.
struct TroncState {
    concat: MCurvesToBSpCurve,
    constraints: [Constraint; 2],
    tolerances: [f64; 2],
    param_seq: Vec<f64>,
}

/// The native piece layout: `(first section, section count)` of each piece, the
/// pieces sharing their end sections.
fn tronc_layout(nb_point_tot: i32, nb_max_p: i32) -> Result<Vec<(i32, i32)>, Failure> {
    if nb_max_p == 0 {
        return Err(Failure::with_message(
            "Standard_DivideByZero",
            "AppBlend_AppSurf::Perform: NbMaxP = 0; expected a positive section count",
        ));
    }
    let mut nbtronc: i32 = nb_point_tot / nb_max_p;
    let mut reste: i32 = nb_point_tot - nbtronc * nb_max_p;
    let mut nmax: i32 = nb_max_p;
    if nbtronc > 0 && reste > 0 {
        nmax = nb_point_tot / (nbtronc + 1);
        if nmax > (2 * nb_max_p) / 3 {
            nbtronc += 1;
            reste = nb_point_tot - nbtronc * nmax;
        } else {
            nmax = nb_max_p;
        }
    } else if nbtronc == 0 {
        (nbtronc, nmax, reste) = (1, reste, 0);
    }
    let rab: i32 = reste / nbtronc + 1;
    let (mut start, mut layout) = (1, Vec::new());
    for _ in 0..nbtronc {
        let rabrab: i32 = rab.min(reste);
        if reste > 0 {
            reste -= rabrab;
        }
        layout.push((start, nmax + rabrab + 1));
        start += nmax + rabrab;
    }
    if let Some(last) = layout.last_mut() {
        last.1 -= 1;
    }
    Ok(layout)
}

/// `(DX - X) < seuil ? (1, 0) : (1 / (DX - X), -X / (DX - X))`: the scale `(X, DX)`
/// of a coordinate range `[min, max]`.
fn axis_scale(min: f64, max: f64, seuil: f64) -> (f64, f64) {
    if (max - min) < seuil {
        return (0.0, 1.0);
    }
    let d: f64 = 1.0 / (max - min);
    (min * -d, d)
}

impl AppBlendAppSurf {
    /// `Perform(Lin, F, NbMaxP)`: approximates the sections of `lin` by pieces of
    /// about `nb_max_p` sections (0 raises `Standard_DivideByZero`, a division by
    /// zero natively); `approx.perform_by_pieces(Some(&line), &mut generator, 30)?`.
    pub fn perform_by_pieces<L, G>(
        &mut self,
        lin: Option<&L>,
        f: &mut G,
        nb_max_p: i32,
    ) -> Result<(), Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        self.done = false;
        let Some(lin) = lin else {
            return Ok(());
        };
        self.seq_poles2d.clear();
        let mut arrays: SectionArrays = self.section_shape(f)?;
        let scale: BoxScale = self.box_scale(lin, f, &mut arrays)?;
        let mut state = TroncState {
            concat: MCurvesToBSpCurve::new(),
            constraints: [Constraint::NoConstraint; 2],
            tolerances: [0.0; 2],
            param_seq: Vec::new(),
        };
        if self.knownp {
            state.param_seq.push(f.parameter(&lin.point(1))?);
        }
        for (start, size) in tronc_layout(lin.nb_points(), nb_max_p)? {
            self.approximate_tronc(lin, f, (&mut arrays, &scale), (start, size), &mut state)?;
        }
        (self.tol3dreached, self.tol2dreached) = (state.tolerances[0], state.tolerances[1]);
        state.concat.perform()?;
        let multc = state.concat.value().clone();
        let (knownp, param_seq) = (self.knownp, state.param_seq);
        self.store_solution(&multc, &arrays, |knots| {
            if !knownp {
                return Ok(());
            }
            if param_seq.len() as i32 != knots.length() {
                let u1: f64 = f.parameter(&lin.point(1))?;
                let u2: f64 = f.parameter(&lin.point(lin.nb_points()))?;
                return bsplclib::reparametrize(u1, u2, knots);
            }
            let lower: i32 = knots.lower();
            for (k, &u) in param_seq.iter().enumerate() {
                knots[lower + k as i32] = u;
            }
            Ok(())
        })
    }
    /// The box of the poles of every section, per 3d coordinate and per 2d curve,
    /// as the affine maps into the unit box (below 1000 times the tolerance, the
    /// identity).
    fn box_scale<L, G>(
        &self,
        lin: &L,
        f: &mut G,
        arrays: &mut SectionArrays,
    ) -> Result<BoxScale, Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        let (nb_u, nb_2d) = (arrays.nb_u_poles, arrays.nb_poles2d);
        let mut boxes = [(f64::MAX, -f64::MAX); 3];
        let mut boxes2d = vec![[(f64::MAX, -f64::MAX); 2]; nb_2d.max(0) as usize];
        let widen = |range: &mut (f64, f64), v: f64| {
            if v < range.0 {
                range.0 = v;
            }
            if v > range.1 {
                range.1 = v;
            }
        };
        for i in 1..=lin.nb_points() {
            f.section_d1(&lin.point(i), &mut arrays.section)?;
            for j in 1..=nb_u {
                let p = arrays.section.poles[j];
                [p.x(), p.y(), p.z()]
                    .into_iter()
                    .zip(boxes.iter_mut())
                    .for_each(|(v, r)| widen(r, v));
            }
            for (j, range) in boxes2d.iter_mut().enumerate() {
                let p = arrays.section.poles2d[j as i32 + 1];
                widen(&mut range[0], p.x());
                widen(&mut range[1], p.y());
            }
        }
        let (seuil, seuil2d) = (1000.0 * self.tol3d, 1000.0 * self.tol2d);
        Ok(BoxScale {
            axes: boxes.map(|(min, max)| axis_scale(min, max, seuil)),
            curves2d: boxes2d
                .into_iter()
                .map(|ranges| ranges.map(|(min, max)| axis_scale(min, max, seuil2d)))
                .collect(),
        })
    }
    /// The multi-line of one piece and its end constraints.
    fn tronc_line<L, G>(
        &self,
        lin: &L,
        f: &mut G,
        (arrays, scale): (&mut SectionArrays, &BoxScale),
        (start, size): (i32, i32),
        constraints: &mut [Constraint; 2],
    ) -> Result<MultiLine, Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        let line = MultiLine::with_nb_multi_points(size)?;
        for i in 1..=size {
            let withderiv: bool = arrays.load_d1(f, &lin.point(start + i - 1))?;
            arrays.weighted(withderiv, Some(scale));
            let constraint = if withderiv {
                Constraint::TangencyPoint
            } else {
                Constraint::PassPoint
            };
            if i == 1 {
                constraints[0] = constraint;
            } else if i == size {
                constraints[1] = constraint;
            }
            line.set_value(i, &arrays.constraint(withderiv)?)?;
        }
        Ok(line)
    }
    /// Approximates one piece and appends its Bezier curves, mapped back from the
    /// unit box, to the concatenation.
    fn approximate_tronc<L, G>(
        &mut self,
        lin: &L,
        f: &mut G,
        (arrays, scale): (&mut SectionArrays, &BoxScale),
        (start, size): (i32, i32),
        state: &mut TroncState,
    ) -> Result<(), Failure>
    where
        L: TheLine,
        G: TheSectionGenerator<Point = L::Point>,
    {
        let line = self.tronc_line(
            lin,
            f,
            (arrays, scale),
            (start, size),
            &mut state.constraints,
        )?;
        if size == 2 {
            self.narrow_two_point_degrees(state.constraints[0], state.constraints[1]);
        }
        let (mut ufloc, mut ulloc) = (0.0, 0.0);
        let mut approx = Compute::new(self.options(ParametrizationType::ChordLength, false));
        if self.knownp {
            ufloc = f.parameter(&lin.point(start))?;
            ulloc = f.parameter(&lin.point(start + size - 1))?;
            let mut params: Vector<'static> = Vector::filled(1, size, 0.0)?;
            for i in 1..=size {
                params[i] = (f.parameter(&lin.point(start + i - 1))? - ufloc) / (ulloc - ufloc);
            }
            approx = Compute::from_parameters(
                &params,
                self.options(ParametrizationType::ChordLength, true),
            )?;
        }
        approx.set_constraints(state.constraints[0], state.constraints[1]);
        approx.perform(&line)?;
        let pole_dists: Vec<f64> = self.append_pieces(&mut approx, (arrays, scale), state)?;
        if self.knownp {
            let whole: f64 = pole_dists.iter().fold(0.0, |sum, d| sum + d);
            let mut iu: f64 = ufloc;
            for d in pole_dists.iter().take(pole_dists.len().saturating_sub(1)) {
                iu += d / whole * (ulloc - ufloc);
                state.param_seq.push(iu);
            }
            state.param_seq.push(ulloc);
        }
        Ok(())
    }
    /// Maps each Bezier piece back from the unit box, keeps the reached tolerances,
    /// appends it; returns the distances between the end poles of each piece (of its
    /// curve of the piece's own index, a native quirk) when there are known
    /// parameters and several pieces.
    fn append_pieces(
        &self,
        approx: &mut Compute,
        (arrays, scale): (&SectionArrays, &BoxScale),
        state: &mut TroncState,
    ) -> Result<Vec<f64>, Failure> {
        let [(x, dx), (y, dy), (z, dz)] = scale.axes;
        let nb_curves: i32 = approx.nb_multi_curves();
        let mut pole_dists: Vec<f64> = Vec::new();
        for index in 1..=nb_curves {
            let (tol3d, tol2d) = approx.error(index)?;
            let t = &mut state.tolerances;
            t[0] = std_max(tol3d / dx, t[0]);
            t[0] = std_max(tol3d / dy, t[0]);
            t[0] = std_max(tol3d / dz, t[0]);
            let mucu = approx.change_value(index)?;
            for j in 1..=arrays.nb_u_poles {
                mucu.transform(j, [-x / dx, 1.0 / dx, -y / dy, 1.0 / dy, -z / dz, 1.0 / dz])?;
            }
            for (j, [(x2d, dx2d), (y2d, dy2d)]) in scale.curves2d.iter().copied().enumerate() {
                let cu: i32 = j as i32 + 1 + arrays.nb_u_poles;
                mucu.transform2d(cu, [-x2d / dx2d, 1.0 / dx2d, -y2d / dy2d, 1.0 / dy2d])?;
                t[1] = std_max(tol2d / dx2d, t[1]);
                t[1] = std_max(tol2d / dy2d, t[1]);
            }
            state.concat.append(mucu);
            if self.knownp && nb_curves > 1 {
                let first = mucu.pole(index, 1)?;
                let last = mucu.pole(index, MultiCurveEval::nb_poles(&*mucu))?;
                pole_dists.push(first.distance(&last));
            }
        }
        Ok(pole_dists)
    }
}
