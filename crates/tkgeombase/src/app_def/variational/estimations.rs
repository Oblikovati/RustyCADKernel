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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (InitCriterionEstimations,
// EstTangent, EstSecnd)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The initial estimations of the tension, flexion and jerk criteria from the
//! tangents and second derivatives the points and the constraints suggest.
use super::Variational;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// A `math_Vector(1, Dimension)` as plain values (index `k` is `V(k + 1)`); the
/// native estimation vectors start uninitialized, the port's at 0.
type EstVector = Vec<f64>;

// math_Vector::Norm2 over the values (its four-accumulator sum).
fn norm2(v: &[f64]) -> Result<f64, Failure> {
    let mut values: Vec<f64> = v.to_vec();
    let upper: i32 = values.len() as i32;
    Vector::from_buffer(&mut values, 1, upper)?.norm2()
}

fn subtracted(a: &[f64], b: &[f64]) -> EstVector {
    a.iter().zip(b).map(|(x, y)| x - y).collect()
}

// V = 0 when |V| <= 1e-9, else V / |V|.
fn unit_or_zero(v: &mut [f64]) -> Result<(), Failure> {
    let vnorm: f64 = norm2(v)?.sqrt();
    if vnorm <= 1.0e-9 {
        v.iter_mut().for_each(|x| *x = 0.);
    } else {
        v.iter_mut().for_each(|x| *x /= vnorm);
    }
    Ok(())
}

// The averaging `(Wpnt V + scale VCnt) * Denom` with Denom = 1 / (Wpnt + Wcnt), or 1
// (EstTangent scales by Wcnt, EstSecnd by Wcnt * Length).
fn average(v: &mut [f64], [wpnt, wcnt]: [f64; 2], (scale, vcnt): (f64, &[f64])) {
    let denom: f64 = if wpnt + wcnt == 0. {
        1.
    } else {
        1. / (wpnt + wcnt)
    };
    v.iter_mut()
        .zip(vcnt)
        .for_each(|(x, c)| *x = (wpnt * *x + scale * c) * denom);
}

// The four indices (current tangent slots and second-derivative slots) that rotate
// through the three estimation vectors: `k` is CurrPoint - 1.
fn rotation(k: usize) -> [usize; 3] {
    [k, (k + 1) % 3, (k + 2) % 3]
}

/// The running estimation state: the three tangents and second derivatives.
struct Estimation {
    tang: [EstVector; 3],
    scnd: [EstVector; 3],
    e2: f64,
    e3: f64,
}

impl Variational {
    /// `InitCriterionEstimations(Length, E1, E2, E3)`: E1 is Length^2; E2 and E3 sum
    /// the squared second derivatives and their differences along the points.
    pub(super) fn init_criterion_estimations(&self, length: f64) -> Result<[f64; 3], Failure> {
        let e1: f64 = length * length;
        let eps1: f64 = precision::confusion() * 0.01;
        let mut est: Estimation = self.first_estimation(length, eps1)?;
        let mut curr: usize = 1;
        for ipnt in self.first_point + 1..self.last_point {
            self.inner_estimation(&mut est, (ipnt, curr), (length, eps1))?;
            curr = (curr + 1) % 3;
        }
        self.last_estimation(&mut est, curr, eps1)?;
        let aux: f64 = length * length;
        Ok([e1, est.e2 * aux, est.e3 * aux])
    }

    // Treatment of first point: the tangents of the first three points and the
    // second derivatives of the first two.
    fn first_estimation(&self, length: f64, eps1: f64) -> Result<Estimation, Failure> {
        let first: i32 = self.first_point;
        let tang: [EstVector; 3] = [
            self.est_tangent(first)?,
            self.est_tangent(first + 1)?,
            self.est_tangent(first + 2)?,
        ];
        let scnd1: EstVector = self.est_secnd(first, (&tang[0], &tang[1]), length)?;
        let scnd2: EstVector = self.est_secnd(first + 1, (&tang[0], &tang[2]), length)?;
        let [e2, e3] = self.first_terms((&scnd1, &scnd2), eps1)?;
        let zero: EstVector = vec![0.; self.dimension as usize];
        Ok(Estimation {
            tang,
            scnd: [scnd1, scnd2, zero],
            e2,
            e3,
        })
    }

    // The E2 and E3 terms of the first point from the first two second derivatives.
    fn first_terms(
        &self,
        (scnd1, scnd2): (&[f64], &[f64]),
        eps1: f64,
    ) -> Result<[f64; 2], Failure> {
        let first: i32 = self.first_point;
        // Modified by skv - Fri Apr  8 14:58:12 2005 OCC8559: Delta between the first two.
        let mut delta: f64 = 0.5 * (self.parameter(first + 1)? - self.parameter(first)?);
        if delta <= eps1 {
            delta = 1.;
        }
        let e2: f64 = norm2(scnd1)? * delta;
        let e3: f64 = if delta > eps1 {
            norm2(&subtracted(scnd2, scnd1))? / (4. * delta)
        } else {
            0.
        };
        Ok([e2, e3])
    }

    // An inner point `ipnt` (slot `curr` of the rotation): the tangent two points on
    // and the second derivative one point on, then its terms of E2 and E3.
    fn inner_estimation(
        &self,
        est: &mut Estimation,
        (ipnt, curr): (i32, usize),
        (length, eps1): (f64, f64),
    ) -> Result<(), Failure> {
        let delta: f64 = 0.5 * (self.parameter(ipnt + 1)? - self.parameter(ipnt - 1)?);
        let [k, next, prev] = rotation(curr);
        if ipnt + 1 != self.last_point {
            est.tang[prev] = self.est_tangent(ipnt + 2)?;
            est.scnd[next] = self.est_secnd(ipnt + 1, (&est.tang[k], &est.tang[prev]), length)?;
        } else {
            est.scnd[next] = self.est_secnd(ipnt + 1, (&est.tang[k], &est.tang[next]), length)?;
        }
        est.e2 += norm2(&est.scnd[k])? * delta;
        est.e3 += if delta > eps1 {
            norm2(&subtracted(&est.scnd[next], &est.scnd[prev]))? / (4. * delta)
        } else {
            0.
        };
        Ok(())
    }

    // Treatment of last point.
    fn last_estimation(&self, est: &mut Estimation, curr: usize, eps1: f64) -> Result<(), Failure> {
        let mut delta: f64 =
            0.5 * (self.parameter(self.last_point)? - self.parameter(self.last_point - 1)?);
        if delta <= eps1 {
            delta = 1.;
        }
        let [k, _, prev] = rotation(curr);
        est.e2 += norm2(&est.scnd[k])? * delta;
        let aux: f64 = norm2(&subtracted(&est.scnd[k], &est.scnd[prev]))?;
        est.e3 += if delta > eps1 {
            aux / (4. * delta)
        } else {
            aux
        };
        Ok(())
    }

    // The point `adr .. adr + Dimension - 1` of myTabPoints (the native vector over it).
    fn tab_vector(&self, adr: i32) -> Result<EstVector, Failure> {
        self.tab_point(adr)?;
        (adr..adr + self.dimension)
            .map(|i| self.tab_point(i))
            .collect()
    }

    // The parabolic estimation at an end: `(c1 P1 + s2 P2 - d P3) * d1` with `d` the
    // chord ratio, or the chord `P2 - P1` (P3 - P2 at the last point).
    fn end_tangent(&self, adr1: i32, last: bool) -> Result<EstVector, Failure> {
        let eps1: f64 = precision::confusion() * 0.01;
        let dim: i32 = self.dimension;
        let (p1, p2, p3) = (
            self.tab_vector(adr1)?,
            self.tab_vector(adr1 + dim)?,
            self.tab_vector(adr1 + 2 * dim)?,
        );
        let v1: f64 = norm2(&subtracted(&p2, &p1))?.sqrt();
        let v2: f64 = if v1 > eps1 {
            norm2(&subtracted(&p3, &p2))?.sqrt()
        } else {
            0.
        };
        if v2 <= eps1 {
            return Ok(if last {
                subtracted(&p3, &p2)
            } else {
                subtracted(&p2, &p1)
            });
        }
        let mut d: f64 = v1 / (v1 + v2);
        let d1: f64 = 1. / (d * (1. - d));
        if last {
            d *= d - 2.;
            return Ok((0..p1.len())
                .map(|i| ((d + 1.) * p1[i] - p2[i] - d * p3[i]) * d1)
                .collect());
        }
        d *= d;
        Ok((0..p1.len())
            .map(|i| ((d - 1.) * p1[i] + p2[i] - d * p3[i]) * d1)
            .collect())
    }

    /// `EstTangent(ipnt, VTang)`: the unit tangent the points suggest, averaged with
    /// the tangency constraint of the point; the ends need three points (weight 0
    /// with fewer), the last one read at `(LastPoint - 3) * Dimension + 1` as natively.
    fn est_tangent(&self, ipnt: i32) -> Result<EstVector, Failure> {
        let dim: i32 = self.dimension;
        let (mut vtang, wpnt): (EstVector, f64) =
            if ipnt == self.first_point || ipnt == self.last_point {
                if self.nb_points < 3 {
                    (vec![0.; dim as usize], 0.)
                } else if ipnt == self.first_point {
                    (self.end_tangent(1, false)?, 1.)
                } else {
                    (self.end_tangent((self.last_point - 3) * dim + 1, true)?, 1.)
                }
            } else {
                let adr1: i32 = (ipnt - self.first_point - 1) * dim + 1;
                (
                    subtracted(&self.tab_vector(adr1 + 2 * dim)?, &self.tab_vector(adr1)?),
                    1.,
                )
            };
        unit_or_zero(&mut vtang)?;
        let (wcnt, vcnt) = self.constraint_vector(ipnt, 1, 0)?;
        average(&mut vtang, [wpnt, wcnt], (wcnt, &vcnt));
        unit_or_zero(&mut vtang)?;
        Ok(vtang)
    }

    /// `EstSecnd(ipnt, VTang1, VTang2, Length, VScnd)`: the tangent difference over
    /// the parameter step, averaged with Length times the curvature constraint.
    fn est_secnd(
        &self,
        ipnt: i32,
        (vtang1, vtang2): (&[f64], &[f64]),
        length: f64,
    ) -> Result<EstVector, Failure> {
        let mut aux: f64 = if ipnt == self.first_point {
            self.parameter(ipnt + 1)? - self.parameter(ipnt)?
        } else if ipnt == self.last_point {
            self.parameter(ipnt)? - self.parameter(ipnt - 1)?
        } else {
            self.parameter(ipnt + 1)? - self.parameter(ipnt - 1)?
        };
        aux = if aux <= 1.0e-9 { 1. } else { 1. / aux };
        let mut vscnd: EstVector = subtracted(vtang2, vtang1)
            .into_iter()
            .map(|x| x * aux)
            .collect();
        let (wcnt, vcnt) = self.constraint_vector(ipnt, 2, 3)?;
        average(&mut vscnd, [1., wcnt], (wcnt * length, &vcnt));
        Ok(vscnd)
    }

    /// The constraint vector of `ipnt` when its constraint is at least `min_kind`
    /// (1 tangent, 2 curvature): the table entries from offset `shift` of its block.
    /// "Warning!! Here it is supposed that all points are in range [myFirstPoint,
    /// myLastPoint]": the search reads one entry past the last constraint (checked).
    fn constraint_vector(
        &self,
        ipnt: i32,
        min_kind: i32,
        shift: i32,
    ) -> Result<(f64, EstVector), Failure> {
        let mut vcnt: EstVector = vec![0.; self.dimension as usize];
        let nb_constr: i32 = self.tables.nb_constr();
        if nb_constr <= 0 {
            return Ok((0., vcnt));
        }
        let mut id_cnt: i32 = 1;
        while self.tables.typ(2 * id_cnt - 1)? < ipnt && id_cnt <= nb_constr {
            id_cnt += 1;
        }
        if self.tables.typ(2 * id_cnt - 1)? != ipnt || self.tables.typ(2 * id_cnt)? < min_kind {
            return Ok((0., vcnt));
        }
        let mut i0: i32 = 2 * self.dimension * (id_cnt - 1) + shift;
        let mut k: usize = 0;
        for _ in 0..self.nb_p3d {
            for _ in 0..3 {
                i0 += 1;
                vcnt[k] = self.tables.value(i0)?;
                k += 1;
            }
            i0 += 3;
        }
        if shift != 0 {
            // EstSecnd steps back one entry before the 2D curvatures.
            i0 -= 1;
        }
        for _ in 0..self.nb_p2d {
            for _ in 0..2 {
                i0 += 1;
                vcnt[k] = self.tables.value(i0)?;
                k += 1;
            }
            i0 += 2;
        }
        Ok((1., vcnt))
    }
}
