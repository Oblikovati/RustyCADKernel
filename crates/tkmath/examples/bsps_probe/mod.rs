//! BSplSLib counterparts of tkmath_bsps_probe.cxx and tkmath_bsps_cache_probe.cxx.
//!
//! Grids arrive as "lowerRow rows lowerCol cols values..." in row-major order.
use crate::bspl_probe::{In, o, op};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::bsplslib::{SurfaceD2, SurfaceSpan};
use rustycad_tkmath::gp::{Point3, Vector3};

mod cache;
mod eval;
mod misc;
pub use cache::{bezier_case, cache_case};
pub use eval::{eval_case, iso_case, ops_case, poles_case, rational_case};
pub use misc::{function_case, interpolate_case, move_case, resolution_case};

pub fn grid3(f: &mut In<'_>) -> Array2<'static, Point3> {
    let (rl, nr, cl, nc) = (f.i(), f.i(), f.i(), f.i());
    let mut grid: Array2<'static, Point3> =
        Array2::with_bounds(rl, rl + nr - 1, cl, cl + nc - 1).unwrap();
    for point in grid.as_mut_slice() {
        *point = Point3::new(f.r(), f.r(), f.r());
    }
    grid
}

pub fn grid1(f: &mut In<'_>) -> Array2<'static, f64> {
    let (rl, nr, cl, nc) = (f.i(), f.i(), f.i(), f.i());
    let mut grid: Array2<'static, f64> =
        Array2::with_bounds(rl, rl + nr - 1, cl, cl + nc - 1).unwrap();
    for value in grid.as_mut_slice() {
        *value = f.r();
    }
    grid
}

pub fn og3(grid: &Array2<'_, Point3>) {
    grid.as_slice().iter().for_each(op);
}

pub fn og1(grid: &Array2<'_, f64>) {
    grid.as_slice().iter().for_each(|value| o(*value));
}

pub fn ov(vector: &Vector3) {
    o(vector.x());
    o(vector.y());
    o(vector.z());
}

/// The D2 layout of BSplSLib::D2 and BSplSLib_Cache::D2: P Vu Vv Vuu Vvv Vuv.
pub fn od2(d: &SurfaceD2) {
    op(&d.p);
    [d.vu, d.vv, d.vuu, d.vvv, d.vuv].iter().for_each(ov);
}

pub fn pgrid(rows: i32, cols: i32) -> Array2<'static, Point3> {
    let mut grid: Array2<'static, Point3> = Array2::with_bounds(1, rows, 1, cols).unwrap();
    grid.init(Point3::new(91.0, 91.0, 91.0));
    grid
}

pub fn wgrid(rows: i32, cols: i32) -> Array2<'static, f64> {
    let mut grid: Array2<'static, f64> = Array2::with_bounds(1, rows, 1, cols).unwrap();
    grid.init(91.0);
    grid
}

/// A surface request: degrees, flags, knots, multiplicities, poles and weights.
pub struct Surf {
    pub ud: i32,
    pub vd: i32,
    pub up: bool,
    pub vp: bool,
    pub ur: bool,
    pub vr: bool,
    pub um: bool,
    pub uk: Array1<'static, f64>,
    pub um_: Array1<'static, i32>,
    pub vk: Array1<'static, f64>,
    pub vm: Array1<'static, i32>,
    pub p: Array2<'static, Point3>,
    pub hw: bool,
    pub w: Array2<'static, f64>,
}

impl Surf {
    pub fn read(f: &mut In<'_>) -> Self {
        let (ud, vd, up, vp, ur, vr, um) = (f.i(), f.i(), f.b(), f.b(), f.b(), f.b(), f.b());
        let (uk, um_, vk, vm) = (f.reals(), f.ints(), f.reals(), f.ints());
        let p = grid3(f);
        let hw: bool = f.b();
        let w = grid1(f);
        Self {
            ud,
            vd,
            up,
            vp,
            ur,
            vr,
            um,
            uk,
            um_,
            vk,
            vm,
            p,
            hw,
            w,
        }
    }
    pub fn w(&self) -> Option<&Array2<'static, f64>> {
        if self.hw { Some(&self.w) } else { None }
    }
    pub fn umults(&self) -> Option<&Array1<'static, i32>> {
        if self.um { Some(&self.um_) } else { None }
    }
    pub fn vmults(&self) -> Option<&Array1<'static, i32>> {
        if self.um { Some(&self.vm) } else { None }
    }
    /// The evaluation arguments of BSplSLib::D0..DN at (u, v) with hints.
    pub fn span(&self, u: f64, v: f64, u_index: i32, v_index: i32) -> SurfaceSpan<'_, 'static> {
        SurfaceSpan {
            u,
            v,
            u_index,
            v_index,
            u_knots: &self.uk,
            v_knots: &self.vk,
            u_mults: self.umults(),
            v_mults: self.vmults(),
            u_degree: self.ud,
            v_degree: self.vd,
            u_rat: self.ur,
            v_rat: self.vr,
            u_per: self.up,
            v_per: self.vp,
        }
    }
}
