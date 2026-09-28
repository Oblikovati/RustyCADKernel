// Created on: 1995-11-02
// Created by: Jacques GOUSSARD
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
// Rust translation of the two static `QuasiFleche` templates of
// ModelingData/TKGeomBase/GCPnts/GCPnts_QuasiUniformDeflection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `QuasiFleche`: the recursive subdivision of `GCPnts_QuasiUniformDeflection`.
use crate::cpnts::PntsCurve;
use crate::gcpnts::point_sequence::PointSequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// `MyMaxQuasiFleshe`: the bound of the call counter.
const MY_MAX_QUASI_FLESHE: i32 = 2000;

/// A point of the subdivision: its parameter, point and (for C1) first derivative.
#[derive(Clone, Copy)]
pub(super) struct FlecheEnd {
    pub u: f64,
    pub point: Point3,
    pub d1: Vector3,
}

/// The fixed inputs and the call counter (`theNbCalls`) of one subdivision.
pub(super) struct QuasiFleche<'c> {
    pub curve: PntsCurve<'c>,
    pub deflection2: f64,
    pub eps: f64,
    pub nb_calls: i32,
}

fn midpoint(first: Point3, second: Point3) -> Point3 {
    let middle: Xyz = (first.xyz() + second.xyz()) * 0.5;
    Point3::new(middle.x(), middle.y(), middle.z())
}

impl QuasiFleche<'_> {
    // The counter entry test. Both early returns leave the counter incremented, as
    // natively, so every later call of the subdivision returns at once too.
    fn enter(&mut self, sequence: &PointSequence) -> Option<i32> {
        self.nb_calls += 1;
        if self.nb_calls >= MY_MAX_QUASI_FLESHE {
            return None;
        }
        let ptslength: i32 = sequence.length();
        if self.nb_calls > 100 && ptslength < 2 {
            return None;
        }
        Some(ptslength)
    }

    /// The C0 `QuasiFleche`: append the points after `deb` up to `fin` so that the
    /// middle of each chord is within the deflection of the curve at the middle
    /// parameter, with at least `nbmin` points; `fleche.fleche_d0(&mut seq, deb, fin, 2)?`.
    pub fn fleche_d0(
        &mut self,
        sequence: &mut PointSequence,
        deb: FlecheEnd,
        fin: FlecheEnd,
        nbmin: i32,
    ) -> Result<(), Failure> {
        let Some(ptslength) = self.enter(sequence) else {
            return Ok(());
        };
        let mut udelta: f64 = fin.u - deb.u;
        let mut delta: FlecheEnd = fin;
        if nbmin > 2 {
            udelta /= f64::from(nbmin - 1);
            delta.point = self.curve.lifted_d0(deb.u + udelta)?;
        }
        delta.u = deb.u + udelta;
        let pverif: Point3 = self.curve.lifted_d0(deb.u + udelta * 0.5)?;
        if midpoint(deb.point, delta.point).square_distance(&pverif) < self.deflection2 {
            sequence.append(delta.u, delta.point);
        } else {
            let middle = FlecheEnd {
                u: deb.u + udelta * 0.5,
                point: pverif,
                d1: deb.d1,
            };
            self.fleche_d0(sequence, deb, middle, 2)?;
            self.fleche_d0(sequence, middle, delta, 2)?;
        }
        if nbmin > 2 {
            let remaining: i32 = nbmin - (sequence.length() - ptslength);
            self.fleche_d0(sequence, delta, fin, remaining)?;
        }
        self.nb_calls -= 1;
        Ok(())
    }

    /// The C1 `QuasiFleche`: as [`Self::fleche_d0`], the deflection first estimated
    /// from the angle of the end tangents (IntWalk_IWalking::TestDeflection) and the
    /// mid-point distance kept when larger than `(chord / 2)^2 + deflection^2`;
    /// `fleche.fleche_d1(&mut seq, deb, fin, 2)?`.
    pub fn fleche_d1(
        &mut self,
        sequence: &mut PointSequence,
        deb: FlecheEnd,
        fin: FlecheEnd,
        nbmin: i32,
    ) -> Result<(), Failure> {
        let Some(ptslength) = self.enter(sequence) else {
            return Ok(());
        };
        let mut udelta: f64 = fin.u - deb.u;
        let mut delta: FlecheEnd = fin;
        if nbmin > 2 {
            udelta /= f64::from(nbmin - 1);
            (delta.point, delta.d1) = self.curve.lifted_d1(deb.u + udelta)?;
        }
        delta.u = deb.u + udelta;
        let fleche: f64 = self.fleche_estimate(deb, delta, udelta)?;
        if fleche < self.deflection2 {
            sequence.append(delta.u, delta.point);
        } else {
            self.fleche_d1(sequence, deb, delta, 3)?;
        }
        if nbmin > 2 {
            let remaining: i32 = nbmin - (sequence.length() - ptslength);
            self.fleche_d1(sequence, delta, fin, remaining)?;
        }
        self.nb_calls -= 1;
        Ok(())
    }

    // The squared deflection of the chord [deb, delta]: the tangent estimate when it
    // applies, replaced by the mid-point distance when that one is too large for the
    // estimate to be trusted (false positives), the mid-point distance otherwise.
    // The negated test keeps the native NaN behavior.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn fleche_estimate(
        &self,
        deb: FlecheEnd,
        delta: FlecheEnd,
        udelta: f64,
    ) -> Result<f64, Failure> {
        // square length of chord
        let norme: f64 = Vector3::between(deb.point, delta.point).square_magnitude();
        let tangent_fleche: Option<f64> = self.tangent_fleche(deb.d1, delta.d1, norme)?;
        let pverif: Point3 = self.curve.lifted_d0(deb.u + udelta * 0.5)?;
        let fleche_mid_mid: f64 = midpoint(deb.point, delta.point).square_distance(&pverif);
        Ok(match tangent_fleche {
            Some(fleche) if !(fleche_mid_mid > norme / 4.0 + self.deflection2) => fleche,
            _ => fleche_mid_mid,
        })
    }

    // Evaluation of the deflection by interpolation (see IntWalk_IWalking::TestDeflection):
    // |t1 - t2|^2 |chord|^2 / 64 of the unit tangents, so at most |chord|^2 / 16; a
    // chord shorter than 4 deflections is excluded, since it would always pass.
    fn tangent_fleche(
        &self,
        vdeb: Vector3,
        vdelta: Vector3,
        norme: f64,
    ) -> Result<Option<f64>, Failure> {
        if !(norme > self.eps && norme > 16.0 * self.deflection2) {
            return Ok(None);
        }
        let (n1, n2) = (vdeb.square_magnitude(), vdelta.square_magnitude());
        if !(n1 > self.eps && n2 > self.eps) {
            return Ok(None);
        }
        // square distance between ends of two normalized vectors [0; 4]
        let normediff: f64 =
            (*vdeb.normalized()?.xyz() - *vdelta.normalized()?.xyz()).square_modulus();
        if normediff > self.eps {
            return Ok(Some(normediff * norme / 64.0));
        }
        Ok(None)
    }
}
