// Rust translation of IntPatch_ALineToWLine::MakeWLine(theALine, theFPar, theLPar,
// theLines) of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ALineToWLine.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sampling loop of `MakeWLine`: the state that the native loops keep across their
//! iterations lives in [`Walk`], and each phase of the loop body is one method.
use super::super::IntPatchSpecPntType as Spt;
use super::super::aline::IntPatchALine;
use super::super::line::IntPatchLine;
use super::super::point::IntPatchPoint;
use super::super::special_points::IntPatchSpecialPoints;
use super::specials::add_point_into_line;
use super::{IntPatchALineToWLine, aline_of};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, set_period};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The flow of one iteration of the sampling loop.
#[derive(PartialEq, Eq)]
pub(super) enum Flow {
    /// `continue` or the end of the body: the parameter moves by the step.
    Next,
    /// `break`: the line is complete.
    Break,
}

/// The native locals of `MakeWLine` that live across the loops.
pub(super) struct Walk<'a> {
    pub(super) conv: &'a IntPatchALineToWLine,
    pub(super) aline: &'a IntPatchALine,
    pub(super) l_par: f64,
    pub(super) tol: f64,
    pub(super) prm_tol: f64,
    pub(super) vertex_params: Vec<f64>,
    pub(super) checked: Vec<bool>,
    pub(super) seq_vertex: Vec<IntPatchPoint>,
    pub(super) periods: [f64; 4],
    pub(super) pre_point_exist: Spt,
    pub(super) singular_surface_id: i32,
    pub(super) prev_l_point: IntSurfPntOn2S,
    pub(super) parameter: f64,
    // The locals of one line (one run of the inner loop).
    pub(super) step: f64,
    pub(super) is_step_reduced: bool,
    pub(super) new_vert_id: usize,
    pub(super) lin_on_2s: Handle<IntSurfLineOn2S>,
    pub(super) first_degenerated: bool,
    pub(super) last_degenerated: bool,
    pub(super) step_min: f64,
    pub(super) step_max: f64,
    pub(super) is_last: bool,
    pub(super) prev_param: f64,
}

/// A point of the line and its parameters on both quadrics (0 on an undefined one,
/// whose native outputs stay at their initial 0).
pub(super) fn point_on_quadrics(conv: &IntPatchALineToWLine, pnt_3d: Point3) -> IntSurfPntOn2S {
    let (u1, v1) = conv.quad1.parameters(pnt_3d).unwrap_or((0.0, 0.0));
    let (u2, v2) = conv.quad2.parameters(pnt_3d).unwrap_or((0.0, 0.0));
    let mut p = IntSurfPntOn2S::new();
    p.set_value_all(pnt_3d, u1, v1, u2, v2);
    p
}

impl IntPatchALineToWLine {
    /// `MakeWLine(theALine, theFPar, theLPar, theLines)`: samples the line from `f_par`
    /// to `l_par` into walking lines broken at the poles, with the vertices of the line
    /// (whose tolerances are unified first) as their vertices.
    /// `conv.make_wline_range(&aline, 0.0, 6.0, &mut lines)?`.
    pub fn make_wline_range(
        &self,
        aline: &Handle<IntPatchALine>,
        f_par: f64,
        l_par: f64,
        lines: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<(), Failure> {
        let line: &IntPatchALine = aline_of(aline)?;
        let nb_vert: i32 = line.nb_vertex();
        if nb_vert == 0 {
            return Ok(());
        }
        unify_vertex_tolerances(line, nb_vert)?;
        let mut walk = Walk::new(self, line, (f_par, l_par), nb_vert)?;
        while walk.parameter < l_par {
            if walk.run_line()? == Flow::Break {
                break;
            }
            if walk.lin_on_2s.nb_points() < 2 {
                walk.parameter += walk.step;
                continue;
            }
            if let Some(wline) = walk.build_wline()? {
                lines.push(wline);
            }
        }
        Ok(())
    }
}

/// The same points can be marked by different vertices: the vertices marking the same
/// point take the sum of their tolerances. The tolerance of the first vertex of a pair
/// is read once, before its inner loop.
fn unify_vertex_tolerances(line: &IntPatchALine, nb_vert: i32) -> Result<(), Failure> {
    for i in 1..nb_vert {
        let cur_vert: IntPatchPoint = line.vertex(i)?;
        let curr_pt: IntSurfPntOn2S = cur_vert.pnt_on_2s();
        let cur_toler: f64 = cur_vert.tolerance();
        for j in i + 1..=nb_vert {
            let vert: IntPatchPoint = line.vertex(j)?;
            let sum_tol: f64 = cur_toler + vert.tolerance();
            if curr_pt.is_same(&vert.pnt_on_2s(), sum_tol, -1.0) {
                line.change_vertex(i, &mut |v| v.set_tolerance(sum_tol))?;
                line.change_vertex(j, &mut |v| v.set_tolerance(sum_tol))?;
            }
        }
    }
    Ok(())
}

impl<'a> Walk<'a> {
    fn new(
        conv: &'a IntPatchALineToWLine,
        aline: &'a IntPatchALine,
        (f_par, l_par): (f64, f64),
        nb_vert: i32,
    ) -> Result<Self, Failure> {
        let tol: f64 = 2.0 * conv.tol_3d + precision::confusion();
        let prm_tol: f64 = std_max(1.0e-4 * (l_par - f_par), precision::p_confusion());
        let vertex_params: Vec<f64> = (1..=nb_vert)
            .map(|i| aline.vertex(i).map(|v| v.parameter_on_line()))
            .collect::<Result<_, _>>()?;
        let periods: [f64; 4] = set_period(&*conv.s1, &*conv.s2)?;
        Ok(Self {
            conv,
            aline,
            l_par,
            tol,
            prm_tol,
            checked: vec![false; vertex_params.len()],
            seq_vertex: vec![IntPatchPoint::new(); vertex_params.len()],
            vertex_params,
            periods,
            pre_point_exist: Spt::SPntNone,
            singular_surface_id: 0,
            prev_l_point: IntSurfPntOn2S::new(),
            parameter: f_par,
            step: 0.0,
            is_step_reduced: false,
            new_vert_id: 0,
            lin_on_2s: Handle::default(),
            first_degenerated: false,
            last_degenerated: false,
            step_min: 0.0,
            step_max: 0.0,
            is_last: false,
            prev_param: f_par,
        })
    }

    /// The step `(L - P) / (N - 1)`, reduced when a vertex that is not checked yet
    /// lies nearer; `Break` when the step is under the resolution of `L`.
    fn set_step(&mut self) -> Flow {
        self.step = (self.l_par - self.parameter) / f64::from(self.conv.nb_points_in_wline - 1);
        if self.step < epsilon(self.l_par) {
            return Flow::Break;
        }
        let mut l_par: f64 = self.l_par;
        for (i, &param) in self.vertex_params.iter().enumerate() {
            if self.checked[i] {
                continue;
            }
            l_par = param;
            if (l_par - self.parameter).abs() < self.prm_tol {
                continue;
            }
            break;
        }
        self.reduce_step_to(l_par);
        Flow::Next
    }

    /// The step becomes a fifth of the distance to the next vertex when it passes it.
    pub(super) fn reduce_step_to(&mut self, l_par: f64) {
        if (self.step - (l_par - self.parameter) > self.prm_tol)
            && ((l_par - self.parameter).abs() > self.prm_tol)
        {
            self.step = std_max((l_par - self.parameter) / 5.0, 1.0e-5);
            self.is_step_reduced = true;
        }
    }

    /// One line: the step, the fresh point set, then the inner loop
    /// `for (; !isLast; aParameter += aStep)`.
    fn run_line(&mut self) -> Result<Flow, Failure> {
        self.is_step_reduced = false;
        if self.set_step() == Flow::Break {
            return Ok(Flow::Break);
        }
        self.new_vert_id = 0;
        self.lin_on_2s = Handle::new(IntSurfLineOn2S::new(None));
        (self.first_degenerated, self.last_degenerated) = (false, false);
        (self.step_min, self.step_max) = (0.1 * self.step, 10.0 * self.step);
        self.is_last = false;
        self.prev_param = self.parameter;
        while !self.is_last {
            if self.sample()? == Flow::Break {
                break;
            }
            self.parameter += self.step;
        }
        Ok(Flow::Next)
    }

    /// The point of the line at the current parameter and whether its 2d parameters can
    /// be computed (not in a pole: a section radius under 5e-6 removes the last point
    /// when it was marked degenerated), with the magnitude of the tangent.
    fn current_point(&mut self) -> Result<(IntSurfPntOn2S, bool, f64), Failure> {
        let (mut pnt_3d, mut tg) = (Point3::default(), Vector3::default());
        self.aline.d1(self.parameter, &mut pnt_3d, &mut tg)?;
        let is_point_valid: bool = self.conv.get_section_radius(pnt_3d)? >= 5.0e-6;
        if !is_point_valid && self.last_degenerated {
            // The current last point is wrong.
            self.lin_on_2s.remove_point(self.lin_on_2s.nb_points())?;
        }
        Ok((
            point_on_quadrics(self.conv, pnt_3d),
            is_point_valid,
            tg.magnitude(),
        ))
    }

    /// The body of the inner loop.
    fn sample(&mut self) -> Result<Flow, Failure> {
        if self.l_par <= self.parameter {
            self.is_last = true;
            if self.pre_point_exist != Spt::SPntNone {
                return Ok(Flow::Break);
            }
            self.parameter = self.l_par;
        }
        let (mut p_on_2s, is_point_valid, tg_magn) = self.current_point()?;
        if self.pre_point_exist != Spt::SPntNone
            && let Some(flow) = self.after_special_point(&mut p_on_2s)?
        {
            return Ok(flow);
        }
        self.pre_point_exist = Spt::SPntNone;
        let vertex_number: Option<usize> = self.passed_vertex(&p_on_2s)?;
        self.prev_param = self.parameter;
        let Some(vertex_number) = vertex_number else {
            self.add_plain_point(&mut p_on_2s, is_point_valid, tg_magn)?;
            return Ok(Flow::Next);
        };
        self.add_vertex(vertex_number, p_on_2s, is_point_valid)
    }

    /// The first vertex not checked yet that the parameter jumped over or fell on, or
    /// whose point is the current one near its parameter (the line may be
    /// self-interfering, so the parameter and the 3d distance are checked together).
    fn passed_vertex(&self, p_on_2s: &IntSurfPntOn2S) -> Result<Option<usize>, Failure> {
        for (i, &param) in self.vertex_params.iter().enumerate() {
            if self.checked[i] {
                continue;
            }
            let vp: IntPatchPoint = self.aline.vertex(i as i32 + 1)?;
            let (prev, cur) = (self.prev_param, self.parameter);
            if (prev < param && param <= cur)
                || (prev == cur && param == cur)
                || (p_on_2s.is_same(&vp.pnt_on_2s(), vp.tolerance(), -1.0)
                    && (vp.parameter_on_line() - cur).abs() < self.prm_tol)
            {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// A point that is no vertex: a valid one after the step computation, a degenerated
    /// one (to be corrected later) as the first point or after at least two points.
    fn add_plain_point(
        &mut self,
        p_on_2s: &mut IntSurfPntOn2S,
        is_point_valid: bool,
        tg_magn: f64,
    ) -> Result<(), Failure> {
        if is_point_valid {
            if !self.is_step_reduced {
                let steps = [self.step_min, self.step_max];
                let (l, cur, defl) = (self.l_par, self.parameter, self.conv.tol_3d);
                let mut step: f64 = self.step;
                let conv = self.conv;
                conv.step_computing(self.aline, p_on_2s, l, cur, tg_magn, steps, defl, &mut step)?;
                self.step = step;
            }
            add_point_into_line(&self.lin_on_2s, &self.periods, p_on_2s, None)?;
            self.prev_l_point = *p_on_2s;
            return Ok(());
        }
        let nb: i32 = self.lin_on_2s.nb_points();
        let to_add: bool = nb == 0 || nb > 1;
        if nb == 0 {
            self.first_degenerated = true;
        } else if nb > 1 {
            self.last_degenerated = true;
        }
        if to_add {
            add_point_into_line(&self.lin_on_2s, &self.periods, p_on_2s, None)?;
            self.prev_l_point = *p_on_2s;
        }
        Ok(())
    }

    /// The reference point of the continuation after a special point: through a pole,
    /// the point of the line halfway to the next vertex that is not the pole itself (or
    /// to the end); when the current point is the previous one, its V parameter on the
    /// singular surface is taken from the previous step.
    fn reference_after_pole(
        &self,
        p_on_2s: &mut IntSurfPntOn2S,
    ) -> Result<IntSurfPntOn2S, Failure> {
        let mut prt: f64 = 0.5 * (self.prev_param + self.l_par);
        for &param in &self.vertex_params {
            if param <= self.prev_param {
                continue;
            }
            if (param - self.prev_param) < self.prm_tol {
                let pnt_3d: Point3 = self.aline.value(param)?;
                if p_on_2s.value().square_distance(&pnt_3d) < precision::square_confusion() {
                    // The vertex is the same as the pole or apex: ignored.
                    continue;
                }
            }
            prt = 0.5 * (param + self.prev_param);
            break;
        }
        let rpt: IntSurfPntOn2S = point_on_quadrics(self.conv, self.aline.value(prt)?);
        let (u1, _, u2, _) = rpt.parameters();
        let same_tol: f64 = std_max(precision::approximation(), self.tol);
        if p_on_2s.is_same(&self.prev_l_point, same_tol, -1.0) {
            // Set the V parameter as the precise value found on the previous step (the
            // native code reuses the variables of the reference parameters).
            if self.singular_surface_id == 1 {
                let (_, v2) = p_on_2s.parameters_on_s1();
                p_on_2s.set_parameters_on(true, u1, v2);
            } else {
                let (_, v1) = p_on_2s.parameters_on_s2();
                p_on_2s.set_parameters_on(false, u2, v1);
            }
        }
        Ok(rpt)
    }

    /// The 2d tolerance of the continuation: none through a pole, the resolutions of the
    /// seam directions otherwise.
    fn continuation_tol_2d(&self) -> Result<f64, Failure> {
        let (s1, s2, tol) = (&self.conv.s1, &self.conv.s2, self.conv.tol_3d);
        let u_res: f64 = std_max(s1.u_resolution(tol)?, s2.u_resolution(tol)?);
        let v_res: f64 = std_max(s1.v_resolution(tol)?, s2.v_resolution(tol)?);
        Ok(match self.pre_point_exist {
            Spt::SPntPole => -1.0,
            Spt::SPntSeamV => v_res,
            Spt::SPntSeamUV => std_max(u_res, v_res),
            _ => u_res,
        })
    }

    /// The continuation after a special point (`aPrePointExist != IntPatch_SPntNone`):
    /// `Some(flow)` when the native body breaks or continues there.
    fn after_special_point(
        &mut self,
        p_on_2s: &mut IntSurfPntOn2S,
    ) -> Result<Option<Flow>, Failure> {
        let tol_2d: f64 = self.continuation_tol_2d()?;
        let rpt: IntSurfPntOn2S = if self.pre_point_exist == Spt::SPntPole {
            self.reference_after_pole(p_on_2s)?
        } else {
            *p_on_2s
        };
        let (s1, s2) = (&self.conv.s1, &self.conv.s2);
        let spt: Spt = self.pre_point_exist;
        let mut prev: IntSurfPntOn2S = self.prev_l_point;
        let cont = IntPatchSpecialPoints::continue_after_special_point(
            s1, s2, &rpt, spt, tol_2d, &mut prev, false,
        )?;
        self.prev_l_point = prev;
        if cont {
            add_point_into_line(&self.lin_on_2s, &self.periods, &mut self.prev_l_point, None)?;
            return Ok(None);
        }
        if self.parameter == self.l_par {
            // Strictly equal!!!
            return Ok(Some(Flow::Break));
        }
        if self.parameter + self.step < self.l_par && self.jumps_after(&rpt)? {
            self.prev_l_point = rpt;
            self.prev_param = self.parameter;
            return Ok(Some(Flow::Next));
        }
        Ok(None)
    }

    /// Prediction of the next point: whether it is farther than PI from the reference
    /// point in the parameters of either surface.
    fn jumps_after(&self, rpt: &IntSurfPntOn2S) -> Result<bool, Failure> {
        let (mut pnt_3d_next, mut tg) = (Point3::default(), Vector3::default());
        self.aline
            .d1(self.parameter + self.step, &mut pnt_3d_next, &mut tg)?;
        let next: IntSurfPntOn2S = point_on_quadrics(self.conv, pnt_3d_next);
        let far = |on_first: bool| {
            next.value_on_surface(on_first)
                .square_distance(rpt.value_on_surface(on_first))
                > PI * PI
        };
        Ok(far(false) || far(true))
    }
}
