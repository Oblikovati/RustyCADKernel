// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of TestDeflection of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx
// ("IntWalk_IWalking_5.gxx") at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TestDeflection`: checks the step to a new point (confused points, 3d and 2d angles
//! against the previous and the new tangents, tangency, the deflection of the chord)
//! and computes the next step.
use super::{COS_REF_2D, COS_REF_3D, IWalkingTypes, IntWalkIWalking, MAX_DIVISION, TheIWFunction};
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::gp::{Direction2, Vector3};
use rustycad_tkmath::math::Vector;

/// The step between the previous and the new point: in the parameters and in space.
struct Chord {
    du: f64,
    dv: f64,
    duv: f64,
    corde: Vector3,
    norme: f64,
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `TestDeflection(sp, Finished, UV, StatusPrecedent, NbDivision, Step, StepSign)`:
    /// the status of the new point `uv` (the function evaluated there) after the
    /// previous point; `step` is halved when the step is too great and recomputed
    /// from the deflection otherwise (unless the line is `finished`).
    pub(super) fn test_deflection(
        &self,
        sp: &mut T::Function,
        finished: bool,
        uv: &Vector<'_>,
        status_precedent: Status,
        (nb_division, step): (&mut i32, &mut f64),
        step_sign: i32,
    ) -> Result<Status, Failure> {
        let chord: Chord = self.chord(sp, uv);
        let mut status: Status = Status::OK;
        if chord.norme <= 4.0 * precision::square_confusion()
            && (chord.duv <= precision::square_p_confusion() || status_precedent != Status::OK)
        {
            status = Status::PointConfondu;
            if status_precedent == Status::PasTropGrand {
                return Ok(Status::ArretSurPointPrecedent);
            }
        } else if let Some(early) = self.angle_3d_test(&chord, step, step_sign) {
            return Ok(early);
        }
        if let Some(early) = self.confused_2d_test(sp, &chord, (status, *step), step_sign)? {
            return Ok(early);
        }
        if *nb_division < MAX_DIVISION
            && status != Status::PointConfondu
            && status_precedent != Status::PointConfondu
        {
            let cosi: f64 = f64::from(step_sign)
                * (chord.du * self.previousd2d.x() + chord.dv * self.previousd2d.y());
            if let Some(early) = self.angles_test(sp, &chord, cosi, (nb_division, step))? {
                return Ok(early);
            }
        }
        if !finished {
            return self.next_step(sp, &chord, status, step);
        }
        Ok(status)
    }
    /// The step from the previous point to the function point at `uv`.
    fn chord(&self, sp: &T::Function, uv: &Vector<'_>) -> Chord {
        let (paramu, paramv) = self.surface_parameters(&self.previous_point);
        let (du, dv) = (uv[1] - paramu, uv[2] - paramv);
        let corde: Vector3 = Vector3::between(self.previous_point.value(), sp.point());
        Chord {
            du,
            dv,
            duv: du * du + dv * dv,
            corde,
            norme: corde.square_magnitude(),
        }
    }
    /// Halves the step: `near` when its parameter components fall under the
    /// resolutions, `far` otherwise.
    fn halve(&self, step: &mut f64, near: Status, far: Status) -> Status {
        *step /= 2.0;
        let step_u: f64 = (*step * self.previousd2d.x()).abs();
        let step_v: f64 = (*step * self.previousd2d.y()).abs();
        if step_u < self.tolerance[1] && step_v < self.tolerance[2] {
            near
        } else {
            far
        }
    }
    /// The 3d angle between the chord and the previous tangent (the result when too
    /// great; OK at once for a null tangent or chord).
    fn angle_3d_test(&self, chord: &Chord, step: &mut f64, step_sign: i32) -> Option<Status> {
        let cosi: f64 = chord.corde.dot(self.previousd3d);
        let mut cosi2: f64 = 0.0;
        if cosi * f64::from(step_sign) >= 0. {
            let div: f64 = self.previousd3d.square_magnitude() * chord.norme;
            if div == 0.0 {
                return Some(Status::OK);
            }
            cosi2 = cosi * cosi / div;
        }
        (cosi2 < COS_REF_3D)
            .then(|| self.halve(step, Status::ArretSurPointPrecedent, Status::PasTropGrand))
    }
    /// The confused points in the parameters (within the resolutions, or a tenth of
    /// the step when smaller: thin shapes, bug #25820), a step back on a confused
    /// point, and the tangency of the function.
    fn confused_2d_test(
        &self,
        sp: &mut T::Function,
        chord: &Chord,
        (status, step): (Status, f64),
        step_sign: i32,
    ) -> Result<Option<Status>, Failure> {
        let min_tol_u: f64 = 0.1 * (step * self.previousd2d.x()).abs();
        let min_tol_v: f64 = 0.1 * (step * self.previousd2d.y()).abs();
        let tol_u: f64 = if min_tol_u > 0.0 {
            std_min(self.tolerance[1], min_tol_u)
        } else {
            self.tolerance[1]
        };
        let tol_v: f64 = if min_tol_v > 0.0 {
            std_min(self.tolerance[2], min_tol_v)
        } else {
            self.tolerance[2]
        };
        if chord.du.abs() < tol_u && chord.dv.abs() < tol_v {
            return Ok(Some(Status::ArretSurPointPrecedent));
        }
        let cosi: f64 = f64::from(step_sign)
            * (chord.du * self.previousd2d.x() + chord.dv * self.previousd2d.y());
        if cosi < 0.0 && status == Status::PointConfondu {
            return Ok(Some(Status::ArretSurPointPrecedent));
        }
        if sp.is_tangent()? {
            return Ok(Some(Status::ArretSurPoint));
        }
        Ok(None)
    }
    /// The angles tested while the step was divided less than `MaxDivision` times:
    /// the 2d step against the previous tangent, the chord against the new 3d
    /// tangent, the 2d step against the new 2d tangent (and a change of side).
    fn angles_test(
        &self,
        sp: &mut T::Function,
        chord: &Chord,
        cosi: f64,
        (nb_division, step): (&mut i32, &mut f64),
    ) -> Result<Option<Status>, Failure> {
        let cosi2: f64 = cosi * cosi / chord.duv;
        if cosi2 < COS_REF_2D || cosi < 0.0 {
            let status: Status =
                self.halve(step, Status::ArretSurPointPrecedent, Status::PasTropGrand);
            *nb_division += 1;
            return Ok(Some(status));
        }
        let cosi: f64 = chord.corde.dot(sp.direction3d()?);
        let cosi2: f64 = cosi * cosi / sp.direction3d()?.square_magnitude() / chord.norme;
        if cosi2 < COS_REF_3D {
            return Ok(Some(self.halve(
                step,
                Status::ArretSurPoint,
                Status::PasTropGrand,
            )));
        }
        let d2d: Direction2 = sp.direction2d()?;
        let cosi: f64 = chord.du * d2d.x() + chord.dv * d2d.y();
        let cosi2: f64 = cosi * cosi / chord.duv;
        if cosi2 < COS_REF_2D || d2d.dot(self.previousd2d) < 0.0 {
            return Ok(Some(self.halve(
                step,
                Status::ArretSurPointPrecedent,
                Status::PasTropGrand,
            )));
        }
        Ok(None)
    }
    /// The next step of an unfinished line: from the confused step, or from the
    /// deflection of the chord estimated with the two tangents.
    fn next_step(
        &self,
        sp: &mut T::Function,
        chord: &Chord,
        status: Status,
        step: &mut f64,
    ) -> Result<Status, Failure> {
        if status == Status::PointConfondu {
            let (d2dx, d2dy) = (self.previousd2d.x().abs(), self.previousd2d.y().abs());
            *step = self.step_from_tangent(chord, d2dx, d2dy);
            return Ok(status);
        }
        // The chord of the cubic through the two points with the two tangents deviates
        // from the middle of the points by Norme * |d1 - d4|^2 / 64 (squared).
        let previous: Vector3 = self.previousd3d.normalized()?;
        let current: Vector3 = sp.direction3d()?.normalized()?;
        let fleche_courante: f64 =
            (*previous.xyz() - *current.xyz()).square_modulus() * chord.norme / 64.;
        let d2d: Direction2 = sp.direction2d()?;
        let (d2dx, d2dy) = (d2d.x().abs(), d2d.y().abs());
        if fleche_courante <= 0.25 * self.fleche * self.fleche {
            *step = self.step_from_tangent(chord, d2dx, d2dy);
            return Ok(status);
        }
        if fleche_courante > self.fleche * self.fleche {
            return Ok(self.halve(step, Status::ArretSurPointPrecedent, Status::PasTropGrand));
        }
        *step = std_min(*step, self.step_from_tangent(chord, d2dx, d2dy));
        Ok(status)
    }
    /// The step that moves one and a half times the last parameter step (at most the
    /// maximal relative step) along the 2d tangent `(d2dx, d2dy)`.
    fn step_from_tangent(&self, chord: &Chord, d2dx: f64, d2dy: f64) -> f64 {
        let step_u: f64 = std_min((1.5 * chord.du).abs(), self.pas * (self.um_last - self.um));
        let step_v: f64 = std_min((1.5 * chord.dv).abs(), self.pas * (self.vm_last - self.vm));
        if d2dx < self.tolerance[1] {
            return step_v / d2dy;
        }
        if d2dy < self.tolerance[2] {
            return step_u / d2dx;
        }
        std_min(step_u / d2dx, step_v / d2dy)
    }
}
