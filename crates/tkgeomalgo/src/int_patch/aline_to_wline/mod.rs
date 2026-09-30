// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ALineToWLine.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_ALineToWLine`: the conversion of an analytic line (`IntPatch_ALine`) of two
//! quadrics into walking lines (docs/geomalgo-design.md section 17).
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
mod build;
mod specials;
mod walk;

use super::aline::IntPatchALine;
use super::line::IntPatchLine;
use super::point_line::curvature_radius_of_inters_line;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfQuadric};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Vector2, Xyz};
use std::f64::consts::PI;

/// `IntPatch_ALineToWLine`: samples an analytic line into walking lines, with the poles
/// and seams of the two quadrics as vertices.
///
/// ```ignore
/// let conv = IntPatchALineToWLine::new(&s1, &s2, 200)?;
/// let mut lines = Vec::new();
/// conv.make_wline(&aline, &mut lines)?;
/// ```
pub struct IntPatchALineToWLine {
    s1: Adaptor3dSurfaceHandle,
    s2: Adaptor3dSurfaceHandle,
    quad1: IntSurfQuadric,
    quad2: IntSurfQuadric,
    nb_points_in_wline: i32,
    tol_open_domain: f64,
    tol_transition: f64,
    tol_3d: f64,
}

/// `IntSurf_Quadric` of an elementary surface; the default (undefined) quadric for the
/// other ones.
fn quadric_of(s: &Adaptor3dSurfaceHandle) -> Result<IntSurfQuadric, Failure> {
    let mut quad = IntSurfQuadric::new();
    match s.get_type()? {
        SurfaceType::Plane => quad.set_plane(&s.plane()?),
        SurfaceType::Cylinder => quad.set_cylinder(&s.cylinder()?),
        SurfaceType::Sphere => quad.set_sphere(&s.sphere()?),
        SurfaceType::Cone => quad.set_cone(&s.cone()?),
        SurfaceType::Torus => quad.set_torus(&s.torus()?),
        _ => {}
    }
    Ok(quad)
}

impl IntPatchALineToWLine {
    /// `IntPatch_ALineToWLine(theS1, theS2, theNbPoints)` (the native default is 200
    /// points): the quadrics of the elementary surfaces, the open-domain tolerance
    /// 1e-9, the transition tolerance 1e-8 and the 3d tolerance Precision::Confusion.
    /// `IntPatchALineToWLine::new(&s1, &s2, 200)?`.
    pub fn new(
        s1: &Adaptor3dSurfaceHandle,
        s2: &Adaptor3dSurfaceHandle,
        nb_points: i32,
    ) -> Result<Self, Failure> {
        Ok(Self {
            quad1: quadric_of(s1)?,
            quad2: quadric_of(s2)?,
            s1: s1.clone(),
            s2: s2.clone(),
            nb_points_in_wline: nb_points,
            tol_open_domain: 1.0e-9,
            tol_transition: 1.0e-8,
            tol_3d: precision::confusion(),
        })
    }

    /// `SetTolOpenDomain(aT)`. `conv.set_tol_open_domain(1e-9)`.
    pub fn set_tol_open_domain(&mut self, t: f64) {
        self.tol_open_domain = t;
    }

    /// `TolOpenDomain()`. `conv.tol_open_domain()`.
    pub fn tol_open_domain(&self) -> f64 {
        self.tol_open_domain
    }

    /// `SetTolTransition(aT)`. `conv.set_tol_transition(1e-8)`.
    pub fn set_tol_transition(&mut self, t: f64) {
        self.tol_transition = t;
    }

    /// `TolTransition()`. `conv.tol_transition()`.
    pub fn tol_transition(&self) -> f64 {
        self.tol_transition
    }

    /// `SetTol3D(aT)`. `conv.set_tol_3d(1e-7)`.
    pub fn set_tol_3d(&mut self, t: f64) {
        self.tol_3d = t;
    }

    /// `Tol3D()`. `conv.tol_3d()`.
    pub fn tol_3d(&self) -> f64 {
        self.tol_3d
    }

    /// `MakeWLine(aline, theLines)`: converts the whole domain of the line, pulled in by
    /// the open-domain tolerance at an open end. `conv.make_wline(&aline, &mut lines)?`.
    pub fn make_wline(
        &self,
        aline: &Handle<IntPatchALine>,
        lines: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<(), Failure> {
        let line: &IntPatchALine = aline_of(aline)?;
        let (mut f, included) = line.first_parameter()?;
        if !included {
            f += self.tol_open_domain;
        }
        let (mut l, included) = line.last_parameter()?;
        if !included {
            l -= self.tol_open_domain;
        }
        self.make_wline_range(aline, f, l, lines)
    }

    /// `CheckDeflection(theMidPt, theMaxDeflection)`: 1 when the point is farther than
    /// the deflection from a quadric (the step is too big), -1 when both distances are
    /// under half of it (too small), 0 otherwise. `conv.check_deflection(mid, 1e-7)`.
    pub fn check_deflection(&self, mid_pt: Xyz, max_deflection: f64) -> i32 {
        let p = Point3::from(mid_pt);
        let mut dist: f64 = self.quad1.distance(p).abs();
        if dist > max_deflection {
            return 1;
        }
        dist = std_max(self.quad2.distance(p).abs(), dist);
        if dist > max_deflection {
            return 1;
        }
        if (dist + dist) < max_deflection {
            return -1;
        }
        0
    }

    /// `GetSectionRadius(thePnt3d)`: the smallest radius of the sections through the
    /// point of the cones and spheres of the pair (RealLast when there is none, 0 outside
    /// a sphere). `conv.get_section_radius(p)?`.
    pub fn get_section_radius(&self, pnt_3d: Point3) -> Result<f64, Failure> {
        let mut ret_val: f64 = f64::MAX;
        for quad in [&self.quad1, &self.quad2] {
            match quad.type_quadric() {
                SurfaceType::Cone => {
                    let cone = quad.cone()?;
                    let r_vec: Xyz = pnt_3d.xyz() - cone.apex().xyz();
                    let dir: Xyz = cone.axis().direction().xyz();
                    ret_val = std_min(ret_val, (r_vec.dot(dir) * cone.semi_angle().tan()).abs());
                }
                SurfaceType::Sphere => {
                    let sphere = quad.sphere()?;
                    let r_vec: Xyz = pnt_3d.xyz() - sphere.location().xyz();
                    let dir: Xyz = sphere.position().direction().xyz();
                    let r: f64 = sphere.radius();
                    let d: f64 = r_vec.dot(dir);
                    let delta: f64 = r * r - d * d;
                    if delta <= 0.0 {
                        return Ok(0.0);
                    }
                    ret_val = std_min(ret_val, delta.sqrt());
                }
                _ => {}
            }
        }
        Ok(ret_val)
    }

    /// `CorrectEndPoint(theLine, theIndex)`: on a quadric whose pole (the apex of a
    /// cone, a pole of a sphere) is the point `index` (1 or the last), replaces its U
    /// parameter by the linear extrapolation of the two neighbouring points along V.
    /// `conv.correct_end_point(&line, 1)?`.
    pub fn correct_end_point(
        &self,
        line: &Handle<IntSurfLineOn2S>,
        index: i32,
    ) -> Result<(), Failure> {
        let points: &IntSurfLineOn2S = line_of_points(line)?;
        // Perform linear extrapolation from two previous points
        let (ind_first, ind_second) = if index == 1 {
            (3, 2)
        } else {
            (index - 2, index - 1)
        };
        let pnt_on_2s: IntSurfPntOn2S = points.value(index)?;
        for is_on_first in [true, false] {
            let quad: &IntSurfQuadric = if is_on_first {
                &self.quad1
            } else {
                &self.quad2
            };
            if !self.is_pole_of(quad, &pnt_on_2s, is_on_first)? {
                continue;
            }
            let prev_prev: Point2 = points.value(ind_first)?.value_on_surface(is_on_first);
            let prev: Point2 = points.value(ind_second)?.value_on_surface(is_on_first);
            let dir = Direction2::from_vector(Vector2::between(prev_prev, prev))?;
            let (x0, y0) = (prev_prev.x(), prev_prev.y());
            let (_, y_end) = pnt_on_2s.parameters_on_surface(is_on_first);
            if dir.y().abs() < rustycad_tkmath::gp::resolution() {
                continue;
            }
            let new_x_end: f64 = dir.x() / dir.y() * (y_end - y0) + x0;
            points.set_uv(index, is_on_first, new_x_end, y_end)?;
        }
        Ok(())
    }

    /// Whether the point is the apex of the cone (1e-10 squared) or on a pole of the
    /// sphere (V within 1e-5 of +-PI/2) of `quad`.
    fn is_pole_of(
        &self,
        quad: &IntSurfQuadric,
        pnt_on_2s: &IntSurfPntOn2S,
        is_on_first: bool,
    ) -> Result<bool, Failure> {
        let (tol, sq_tol) = (1.0e-5, 1.0e-10);
        match quad.type_quadric() {
            SurfaceType::Cone => {
                let apex: Point3 = quad.cone()?.apex();
                Ok(!(apex.square_distance(&pnt_on_2s.value()) > sq_tol))
            }
            SurfaceType::Sphere => {
                let (_, v) = pnt_on_2s.parameters_on_surface(is_on_first);
                Ok(!((v - PI / 2.0).abs() > tol && (v + PI / 2.0).abs() > tol))
            }
            _ => Ok(false),
        }
    }

    /// `StepComputing(theALine, thePOn2S, theLastParOfAline, theCurParam,
    /// theTgMagnitude, theStepMin, theStepMax, theMaxDeflection, theStep)`: the step of
    /// the line parameter from the osculating circle (the tangent whose end is within the
    /// 3d tolerance of it), refined by bisection on the deflection of the chord middle
    /// (50 iterations). Writes the step into `step` and answers whether it converged;
    /// false (step kept) on a null tangent or an undefined curvature.
    /// `conv.step_computing(&aline, &p, l, u, tg, [smin, smax], 1e-7, &mut step)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn step_computing(
        &self,
        aline: &IntPatchALine,
        p_on_2s: &IntSurfPntOn2S,
        last_par_of_aline: f64,
        cur_param: f64,
        tg_magnitude: f64,
        [step_min, step_max]: [f64; 2],
        max_deflection: f64,
        step: &mut f64,
    ) -> Result<bool, Failure> {
        if tg_magnitude < precision::confusion() {
            return Ok(false);
        }
        let eps: f64 = self.tol_3d;
        let not_filled_range: f64 = last_par_of_aline - cur_param;
        let (min_step, max_step) = (step_min, std_min(step_max, not_filled_range));
        if min_step > max_step {
            *step = max_step;
            return Ok(true);
        }
        let r: f64 = curvature_radius_of_inters_line(&self.s1, &self.s2, p_on_2s)?;
        if r < 0.0 {
            return Ok(false);
        }
        // The 3D step is the length of the tangent to the osculating circle whose end
        // is within eps of the circle, converted to the parameter of the curve.
        *step = std_min((eps * (2.0 * r + eps)).sqrt() / tg_magnitude, max_step);
        *step = std_max(*step, min_step);
        let bounds = [min_step, max_step];
        self.refine_step(
            aline,
            p_on_2s.value(),
            cur_param,
            bounds,
            max_deflection,
            step,
        )
    }

    /// The bisection loop of `StepComputing`: a range of 1e6 gives an error near 1e-9
    /// after 50 halvings (1e15 < 2^50 < 1e16).
    fn refine_step(
        &self,
        aline: &IntPatchALine,
        p1: Point3,
        cur_param: f64,
        [mut min_step, mut max_step]: [f64; 2],
        max_deflection: f64,
        step: &mut f64,
    ) -> Result<bool, Failure> {
        let nb_iter_max: i32 = 50;
        let mut nb_iter: i32 = 0;
        loop {
            nb_iter += 1;
            let p2: Point3 = aline.value(cur_param + *step)?;
            let status: i32 = self.check_deflection(0.5 * (p1.xyz() + p2.xyz()), max_deflection);
            if status == 0 {
                break;
            }
            if status < 0 {
                min_step = *step;
            } else {
                max_step = *step;
            }
            *step = 0.5 * (min_step + max_step);
            if !((max_step - min_step) > precision::p_confusion() && nb_iter <= nb_iter_max) {
                break;
            }
        }
        Ok(nb_iter <= nb_iter_max)
    }
}

/// The analytic line of the handle, or the Standard_NullObject of a null handle.
fn aline_of(aline: &Handle<IntPatchALine>) -> Result<&IntPatchALine, Failure> {
    aline.get().ok_or_else(|| {
        super::failures::null_object("IntPatch_ALineToWLine::MakeWLine", "IntPatch_ALine")
    })
}

/// The point set of the handle, or the Standard_NullObject of a null handle.
fn line_of_points(line: &Handle<IntSurfLineOn2S>) -> Result<&IntSurfLineOn2S, Failure> {
    line.get().ok_or_else(|| {
        super::failures::null_object("IntPatch_ALineToWLine::CorrectEndPoint", "IntSurf_LineOn2S")
    })
}
