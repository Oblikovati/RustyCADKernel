// Rust translation of the periodic members of TopOpeBRepTool_ShapeTool (AdjustOnPeriodic,
// Closed, PeriodizeParameter and the static PARINBOUNDS, PARONBOUND and ADJUST in
// ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_ShapeTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! UV values brought into the period of a face, closing edges and the periodic
//! parameter of a closing edge.
use super::{TopOpeBRepToolShapeTool, program_error};
use crate::top_ope_brep_tool::two_d::fc2d_curve_on_surface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Direction2;

/// `PARINBOUNDS(par, first, last, tol)`: strictly inside the bounds by `tol`.
fn par_in_bounds(par: f64, first: f64, last: f64, tol: f64) -> bool {
    (first + tol) <= par && par <= (last - tol)
}

/// `PARONBOUND(par, bound, tol)`: within `tol` of the bound.
fn par_on_bound(par: f64, bound: f64, tol: f64) -> bool {
    (bound - tol) <= par && par <= (bound + tol)
}

/// `ADJUST(par, first, last, tol)`: a parameter inside or on the first bound moved by
/// one period, on the last bound moved back.
fn adjust(par: f64, first: f64, last: f64, tol: f64) -> f64 {
    let period = last - first;
    if par_in_bounds(par, first, last, tol) || par_on_bound(par, first, tol) {
        return par + period;
    }
    if par_on_bound(par, last, tol) {
        return par - period;
    }
    par
}

/// `u` brought into `[first, first + period]` unless it is on the period end.
fn in_period(u: f64, first: f64, period: f64, tol: f64) -> f64 {
    if (u - first - period).abs() > tol {
        return elclib::in_period(u, first, first + period);
    }
    u
}

impl TopOpeBRepToolShapeTool {
    /// `AdjustOnPeriodic(F, u, v)`: `u` and `v` brought into the period of the surface
    /// from the first UV bounds of the face, unless they are on the period end;
    /// `TopOpeBRepToolShapeTool::adjust_on_periodic(&face, &mut u, &mut v)?`.
    pub fn adjust_on_periodic(f: &Shape, u: &mut f64, v: &mut f64) -> Result<(), Failure> {
        let ff = topods::face(f)?;
        let (surf, _) = BRepTool::surface_with_location(&ff)?;
        let s = surf.get().ok_or_else(|| {
            program_error("AdjustOnPeriodic: null surface; expected a face with a surface")
        })?;
        let (is_uperio, is_vperio) = (s.is_u_periodic(), s.is_v_periodic());
        if !is_uperio && !is_vperio {
            return Ok(());
        }
        let (uffirst, _uflast, vffirst, _vflast) = brep_tools::uv_bounds(&ff)?;
        let tol: f64 = precision::p_confusion();
        if is_uperio {
            *u = in_period(*u, uffirst, s.u_period()?, tol);
        }
        if is_vperio {
            *v = in_period(*v, vffirst, s.v_period()?, tol);
        }
        Ok(())
    }

    /// `Closed(S1, S2)`: the edge is closed on the face and appears at least twice in
    /// it; `TopOpeBRepToolShapeTool::closed(&edge, &face)?`.
    pub fn closed(s1: &Shape, s2: &Shape) -> Result<bool, Failure> {
        let (e, f) = (topods::edge(s1)?, topods::face(s2)?);
        if !BRepTool::is_closed_on_face(&e, &f)? {
            return Ok(false);
        }
        let n = Explorer::new(s2, ShapeEnum::Edge, ShapeEnum::Shape)?
            .filter(|x| x.is_same(s1))
            .count();
        Ok(n >= 2)
    }

    /// `PeriodizeParameter(par, EE, FF)`: for a closing edge on a periodic surface with
    /// an iso-U (or iso-V) line p-curve, the parameter moved by one period of the
    /// natural U (or V) bounds (ADJUST); Standard_ProgramError without p-curve;
    /// `TopOpeBRepToolShapeTool::periodize_parameter(0.5, &edge, &face)?`.
    pub fn periodize_parameter(par: f64, ee: &Shape, ff: &Shape) -> Result<f64, Failure> {
        if !Self::closed(ee, ff)? {
            return Ok(par);
        }
        let (e, f) = (topods::edge(ee)?, topods::face(ff)?);
        let (surf, _) = BRepTool::surface_with_location(&f)?;
        let s = surf.get().ok_or_else(|| {
            program_error("PeriodizeParameter: null surface; expected a face with a surface")
        })?;
        if !s.is_u_periodic() && !s.is_v_periodic() {
            return Ok(par);
        }
        let (ufirst, ulast, vfirst, vlast) = s.bounds();
        let (pc, _, _, _) = fc2d_curve_on_surface(&e, &f, false)?;
        if pc.is_null() {
            return Err(program_error("ShapeTool::PeriodizeParameter : no 2d curve"));
        }
        let hl: Handle<Geom2dLine> = pc.cast();
        let Some(line) = hl.get() else {
            // NYI : BSpline ...
            return Ok(par);
        };
        let d = line.direction();
        let tol: f64 = precision::angular();
        if d.is_parallel(Direction2::Y, tol) {
            return Ok(adjust(par, ufirst, ulast, tol));
        }
        if d.is_parallel(Direction2::X, tol) {
            return Ok(adjust(par, vfirst, vlast, tol));
        }
        Ok(par)
    }
}
