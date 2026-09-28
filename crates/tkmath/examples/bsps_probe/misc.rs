//! MovePoint, Resolution, Interpolate and FunctionMultiply.
use super::{Surf, grid1, grid3, og1, og3, pgrid, wgrid};
use crate::bspl_probe::{In, o, oi, report};
use rustycad_tkmath::bsplslib::{
    self, EvaluatorFunction, InterpolationGrid, InterpolationKnots, MoveRange, MultiplySource,
    ResolutionDirection,
};
use rustycad_tkmath::gp::{Point3, Vector3};

// move: u v dx dy dz ui1 ui2 vi1 vi2 udeg vdeg rational poles weights uflat vflat
pub fn move_case(v: &[f64]) {
    let mut f = In::new(v);
    let (u, vv) = (f.r(), f.r());
    let displ = Vector3::new(f.r(), f.r(), f.r());
    let range = MoveRange {
        u_index1: f.i(),
        u_index2: f.i(),
        v_index1: f.i(),
        v_index2: f.i(),
    };
    let (ud, vd, rat) = (f.i(), f.i(), f.b());
    let (p, w, uf, vf) = (grid3(&mut f), grid1(&mut f), f.reals(), f.reals());
    let mut np = p.clone();
    np.init(Point3::new(91.0, 91.0, 91.0));
    let result = bsplslib::move_point(u, vv, displ, range, ud, vd, rat, &p, &w, &uf, &vf, &mut np);
    report(result, |(a, b, c, d)| {
        [a, b, c, d].into_iter().for_each(oi);
        og3(&np);
    });
    println!();
}

// resolution: surface tolerance
pub fn resolution_case(v: &[f64]) {
    let mut f = In::new(v);
    let s = Surf::read(&mut f);
    let tol: f64 = f.r();
    let u = ResolutionDirection {
        knots: &s.uk,
        mults: &s.um_,
        degree: s.ud,
        rational: s.ur,
        periodic: s.up,
    };
    let vd = ResolutionDirection {
        knots: &s.vk,
        mults: &s.vm,
        degree: s.vd,
        rational: s.vr,
        periodic: s.vp,
    };
    report(bsplslib::resolution(&s.p, s.w(), u, vd, tol), |(ut, vt)| {
        o(ut);
        o(vt);
    });
    println!();
}

// interpolate: udeg vdeg uflat vflat upar vpar rational values weights
pub fn interpolate_case(v: &[f64]) {
    let mut f = In::new(v);
    let (ud, vd) = (f.i(), f.i());
    let (uf, vf, up, vp) = (f.reals(), f.reals(), f.reals(), f.reals());
    let rat: bool = f.b();
    let (mut p, mut w) = (grid3(&mut f), grid1(&mut f));
    let grid = InterpolationGrid {
        u_degree: ud,
        v_degree: vd,
        u_flat_knots: &uf,
        v_flat_knots: &vf,
        u_parameters: &up,
        v_parameters: &vp,
    };
    let mut problem: i32 = -7;
    let result = if rat {
        bsplslib::interpolate_rational(&grid, &mut p, &mut w, &mut problem)
    } else {
        bsplslib::interpolate(&grid, &mut p, &mut problem)
    };
    report(result, |()| {
        oi(problem);
        og3(&p);
        if rat {
            og1(&w);
        }
    });
    println!();
}

/// c0 + u c1 + v (c2 + u c3), failing above `limit` in u.
struct ProbeSurfaceFunction {
    c: [f64; 4],
    limit: f64,
}

impl EvaluatorFunction for ProbeSurfaceFunction {
    fn evaluate(&self, _request: i32, u: f64, v: f64, result: &mut f64, error_code: &mut i32) {
        if u > self.limit {
            *error_code = 1;
            return;
        }
        *result = self.c[0] + u * self.c[1] + v * (self.c[2] + u * self.c[3]);
        *error_code = 0;
    }
}

// function: surface unewflat vnewflat unewdeg vnewdeg c0 c1 c2 c3 limit
pub fn function_case(v: &[f64]) {
    let mut f = In::new(v);
    let s = Surf::read(&mut f);
    let (uf, vf, und, vnd) = (f.reals(), f.reals(), f.i(), f.i());
    let function = ProbeSurfaceFunction {
        c: [f.r(), f.r(), f.r(), f.r()],
        limit: f.r(),
    };
    let (nu, nv) = (uf.len() as i32 - und - 1, vf.len() as i32 - vnd - 1);
    let (mut nn, mut nd) = (pgrid(nu, nv), wgrid(nu, nv));
    let source = MultiplySource {
        surface: s.span(0.0, 0.0, 0, 0),
        poles: &s.p,
        weights: s.w(),
    };
    let knots = InterpolationKnots {
        u_flat_knots: &uf,
        v_flat_knots: &vf,
        u_new_degree: und,
        v_new_degree: vnd,
    };
    let mut status: i32 = -7;
    let result =
        bsplslib::function_multiply(&function, &source, &knots, &mut nn, &mut nd, &mut status);
    report(result, |()| {
        oi(status);
        og3(&nn);
        og1(&nd);
    });
    println!();
}
