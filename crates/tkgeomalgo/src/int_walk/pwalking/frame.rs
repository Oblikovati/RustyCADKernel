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
// Rust translation of the framing on the border in the marching loop of Perform in
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The framing on the border: the parameters left the domain (`TestArret` reframed
//! them on the best isoparametric), so the point is computed there, in a corner or on
//! a side, and the line may be extended through a tangent zone.
use super::IntWalkPWalking;
use super::perform::{Flow, MarchState};
use crate::int_imp::IntImpConstIsoparametric as Iso;
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool as tool;
use rustycad_tkmath::gp::{self, Direction2, Direction3, Point2, Vector2, Vector3};
use std::f64::consts::PI;

/// The previous point and its tangents, saved to come back to them.
struct PreviousSave {
    point: IntSurfPntOn2S,
    tangent: bool,
    d: Direction3,
    d1: Direction2,
    d2: Direction2,
}

impl IntWalkPWalking {
    /// The native framing branch (`!close` after `TestArret` arrived).
    pub(super) fn frame_on_border(&mut self, st: &mut MarchState) -> Result<Flow, Failure> {
        let b_prev_not_tangent: bool =
            !self.previoustg || !self.my_intersection_on2s.is_tangent()?;
        let sauv_choix_iso: Iso = st.choix_iso;
        st.choix_iso =
            self.my_intersection_on2s
                .perform_iso(&st.param, &mut st.rsnld, st.choix_iso)?;
        if self.my_intersection_on2s.is_empty()? {
            // The framing failed: the step is divided.
            st.arrive = false;
            st.no_test_deflection = true;
            self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
            return Ok(Flow::Next);
        }
        if self.test_arret(true, &mut st.param, &mut st.choix_iso)? {
            return self.frame_in_corner(st, sauv_choix_iso);
        }
        self.frame_on_side(st, sauv_choix_iso, b_prev_not_tangent)
    }
    /// Both surfaces are left at once (or in a corner): the point is recomputed on
    /// the new isoparametric and added, and the walking restarts the other way.
    fn frame_in_corner(
        &mut self,
        st: &mut MarchState,
        sauv_choix_iso: Iso,
    ) -> Result<Flow, Failure> {
        st.nb_pas_ok_conseq = -10;
        st.choix_iso =
            self.my_intersection_on2s
                .perform_iso(&st.param, &mut st.rsnld, st.choix_iso)?;
        if self.my_intersection_on2s.is_empty()? {
            st.arrive = false;
            self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
            st.no_test_deflection = true;
            st.choix_iso = sauv_choix_iso;
            return Ok(Flow::Next);
        }
        self.take_new_point()?;
        let pl = self.previous_point.value();
        if st.b_test_first_point
            && let Some(flow) = self.skip_first_point(st, pl)
        {
            return Ok(flow);
        }
        if self.append_previous_point(st)? {
            return Ok(Flow::Next);
        }
        st.level_of_iter_without_append = 0;
        self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
        Ok(Flow::Next)
    }
    /// One border reached: the point on it ends this direction when it is in the
    /// domain (within 1e-11) and the line does not turn back; otherwise the previous
    /// point comes back. A line that entered a tangent zone is extended through it.
    fn frame_on_side(
        &mut self,
        st: &mut MarchState,
        sauv_choix_iso: Iso,
        b_prev_not_tangent: bool,
    ) -> Result<Flow, Failure> {
        let save = PreviousSave {
            point: self.previous_point,
            tangent: self.previoustg,
            d: self.previousd,
            d1: self.previousd1,
            d2: self.previousd2,
        };
        self.take_new_point()?;
        st.arrive = false;
        if let Some(flow) = self.end_on_side(st, &save)? {
            return Ok(flow);
        }
        let mut was_extended: bool = false;
        if st.arrive && self.my_intersection_on2s.is_tangent()? && b_prev_not_tangent {
            was_extended = self.try_extend(st, sauv_choix_iso)?;
        }
        self.repartir_ou_diviser(&mut st.deja_reparti, &mut st.choix_iso, &mut st.arrive)?;
        let inter = &self.my_intersection_on2s;
        if st.arrive
            && inter.is_done()
            && !inter.is_empty()?
            && inter.is_tangent()?
            && b_prev_not_tangent
            && !was_extended
        {
            self.try_extend(st, sauv_choix_iso)?;
        }
        Ok(Flow::Next)
    }
    /// `ExtendLineInCommonZone` on the saved isoparametric; the line goes on when it
    /// left the tangent zone. Returns whether it was extended.
    fn try_extend(&mut self, st: &mut MarchState, sauv_choix_iso: Iso) -> Result<bool, Failure> {
        if !self.extend_line_in_common_zone(sauv_choix_iso, st.deja_reparti)? {
            return Ok(false);
        }
        st.arrive = false;
        st.choix_iso = sauv_choix_iso;
        Ok(true)
    }
    /// The new point on the side: the parameters become its own; in the domain it is
    /// checked and added (the line then arrives), otherwise the previous point comes
    /// back. Returns the flow when the iteration ends there.
    fn end_on_side(
        &mut self,
        st: &mut MarchState,
        save: &PreviousSave,
    ) -> Result<Option<Flow>, Failure> {
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        let param_on_s1 = Point2::new(st.param[1], st.param[2]);
        let param_on_s2 = Point2::new(st.param[3], st.param[4]);
        (st.param[1], st.param[2], st.param[3], st.param[4]) = (u1, v1, u2, v2);
        let tol2d: f64 = 1.0e-11;
        let (lo, up) = (self.lower, self.upper);
        let flag1: bool = u1 >= lo[0] - tol2d
            && v1 >= lo[1] - tol2d
            && u1 <= up[0] + tol2d
            && v1 <= up[1] + tol2d;
        let flag2: bool = u2 >= lo[2] - tol2d
            && v2 >= lo[3] - tol2d
            && u2 <= up[2] + tol2d
            && v2 <= up[3] + tol2d;
        if !(flag1 && flag2) {
            self.previous_point = save.point;
            (
                self.previoustg,
                self.previousd,
                self.previousd1,
                self.previousd2,
            ) = (save.tangent, save.d, save.d1, save.d2);
            return Ok(None);
        }
        if self.turns_back(save)? {
            st.arrive = true;
            return Ok(Some(Flow::Next));
        }
        let pl = self.previous_point.value();
        if st.b_test_first_point
            && let Some(flow) = self.skip_first_point(st, pl)
        {
            return Ok(Some(flow));
        }
        if self.previoustg && self.stuck_in_tangent_zone(save, (param_on_s1, param_on_s2))? {
            st.arrive = true;
            return Ok(Some(Flow::Next));
        }
        if self.append_previous_point(st)? {
            return Ok(Some(Flow::Next));
        }
        st.level_of_iter_without_append = 0;
        st.arrive = true;
        Ok(None)
    }
    /// Whether the line turns back at the new point on either surface: the last
    /// step and the step to the new point point against each other.
    fn turns_back(&self, save: &PreviousSave) -> Result<bool, Failure> {
        let line = self.line_ref()?;
        if line.nb_points() <= 1 {
            return Ok(false);
        }
        let (pp_u1, pp_v1, pp_u2, pp_v2) = line.value(line.nb_points() - 1)?.parameters();
        let (p_u1, p_v1, p_u2, p_v2) = save.point.parameters();
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        let v1_on_s1 = Vector2::between(Point2::new(pp_u1, pp_v1), Point2::new(p_u1, p_v1));
        let v2_on_s1 = Vector2::between(Point2::new(p_u1, p_v1), Point2::new(u1, v1));
        let v1_on_s2 = Vector2::between(Point2::new(pp_u2, pp_v2), Point2::new(p_u2, p_v2));
        let v2_on_s2 = Vector2::between(Point2::new(p_u2, p_v2), Point2::new(u2, v2));
        Ok(v1_on_s1.dot(v2_on_s1) < 0.0 || v1_on_s2.dot(v2_on_s2) < 0.0)
    }
    /// In a tangent zone near a border, whether the walk turns around the same point
    /// (the steps to the framed parameters and to the new point make more than 3/4 of
    /// a half turn on both surfaces) or goes along a direction in which the
    /// intersection does not move (a null directional derivative on both surfaces).
    fn stuck_in_tangent_zone(
        &mut self,
        save: &PreviousSave,
        (param_on_s1, param_on_s2): (Point2, Point2),
    ) -> Result<bool, Failure> {
        let (prev_u1, prev_v1, prev_u2, prev_v2) = save.point.parameters();
        let (prev_s1, prev_s2) = (Point2::new(prev_u1, prev_v1), Point2::new(prev_u2, prev_v2));
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        let (cur_s1, cur_s2) = (Point2::new(u1, v1), Point2::new(u2, v2));
        let max_angle: f64 = 3. * PI / 4.;
        let angle_s1: f64 = turn_angle(
            Vector2::between(prev_s1, param_on_s1),
            Vector2::between(prev_s1, cur_s1),
        )?;
        let angle_s2: f64 = turn_angle(
            Vector2::between(prev_s2, param_on_s2),
            Vector2::between(prev_s2, cur_s2),
        )?;
        if angle_s1 > max_angle && angle_s2 > max_angle {
            return Ok(true);
        }
        self.is_singular_direction((prev_s1, cur_s1), (prev_s2, cur_s2))
    }
    /// Whether the directional derivatives of both surfaces along the chosen steps
    /// vanish (a sphere walked along U from its pole, for example).
    fn is_singular_direction(
        &mut self,
        (prev_s1, cur_s1): (Point2, Point2),
        (prev_s2, cur_s2): (Point2, Point2),
    ) -> Result<bool, Failure> {
        let (dir_s1, dir_s2) = (
            Vector2::between(prev_s1, cur_s1),
            Vector2::between(prev_s2, cur_s2),
        );
        let (caro1, caro2) = self.surfaces();
        let d1 = tool::d1(&caro1, cur_s1.x(), cur_s1.y())?;
        let d2 = tool::d1(&caro2, cur_s2.x(), cur_s2.y())?;
        let f1: f64 = directional_square(d1.d1u, d1.d1v, dir_s1);
        let f2: f64 = directional_square(d2.d1u, d2.d1v, dir_s2);
        Ok(f1 < gp::resolution() && f2 < gp::resolution())
    }
}

/// The squared derivative of a surface along the parametric direction `dir`.
fn directional_square(du: Vector3, dv: Vector3, dir: Vector2) -> f64 {
    let fx: f64 = du.x() * dir.x() + dv.x() * dir.y();
    let fy: f64 = du.y() * dir.x() + dv.y() * dir.y();
    let fz: f64 = du.z() * dir.x() + dv.z() * dir.y();
    fx * fx + fy * fy + fz * fz
}

/// The angle between the step to the framed parameters and the step to the new
/// point: a half turn when the new point is the previous one, none when the framed
/// parameters are (the walk goes along a tangent zone).
fn turn_angle(to_param: Vector2, to_current: Vector2) -> Result<f64, Failure> {
    if to_current.square_magnitude() < gp::resolution() {
        return Ok(PI);
    }
    if to_param.square_magnitude() < gp::resolution() {
        return Ok(0.0);
    }
    Ok(to_param.angle(to_current)?.abs())
}
