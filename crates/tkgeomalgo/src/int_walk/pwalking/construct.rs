// Rust translation of the constructors and ComputePasInit of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors: the walking bounds (widened on periodic surfaces, and by
//! `KELARG` steps on the others for the second constructor), the resolutions and the
//! initial steps; and `ComputePasInit`, the steps estimated from a parameter box.
use super::IntWalkPWalking;
use crate::int_imp::IntImpConstIsoparametric;
use crate::int_surf::IntSurfPntOn2S;
use crate::int_walk::IntWalkTheInt2S;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as tool, Adaptor3dSurfaceHandle};
use rustycad_tkmath::gp::{Direction2, Direction3};

/// `KELARG`: the number of steps a walking bound is widened by.
const KELARG: f64 = 20.;

/// The bounds `(Um, Vm)`, `(UM, VM)` of a surface.
fn surface_bounds(caro: &Adaptor3dSurfaceHandle) -> Result<([f64; 2], [f64; 2]), Failure> {
    let lower: [f64; 2] = [
        tool::first_u_parameter(caro)?,
        tool::first_v_parameter(caro)?,
    ];
    Ok((
        lower,
        [tool::last_u_parameter(caro)?, tool::last_v_parameter(caro)?],
    ))
}

/// The resolution of a parameter, multiplied by its largest bound magnitude when that
/// raises it (and, for the first constructor, keeps it under 10).
fn scaled_resolution(reso: f64, (first, last): (f64, f64), capped: bool) -> f64 {
    let mut maxval: f64 = first.abs();
    let maxval2: f64 = last.abs();
    if maxval2 > maxval {
        maxval = maxval2;
    }
    let newreso: f64 = reso * maxval;
    if newreso > reso && (!capped || newreso < 10.) {
        newreso
    } else {
        reso
    }
}

impl IntWalkPWalking {
    /// `IntWalk_PWalking(Caro1, Caro2, TolTangency, Epsilon, Deflection, Increment)`:
    /// prepares the marching of the intersection of `caro1` and `caro2`; `Perform`
    /// computes the line. `IntWalkPWalking::new(&s1, &s2, 1e-6, 1e-8, 0.01, 0.1)?`.
    pub fn new(
        caro1: &Adaptor3dSurfaceHandle,
        caro2: &Adaptor3dSurfaceHandle,
        tol_tangency: f64,
        epsilon: f64,
        deflection: f64,
        increment: f64,
    ) -> Result<Self, Failure> {
        let mut walk: Self =
            Self::base(caro1, caro2, [tol_tangency, epsilon, deflection, increment])?;
        walk.load_resolutions(caro1, caro2, true)?;
        walk.load_steps();
        walk.limit_resolutions();
        walk.widen_periodic(caro1, caro2, false)?;
        walk.my_step_min = walk.reso.map(|reso| 100.0 * reso);
        for i in 0..4 {
            if walk.pasuv[i] > 10. {
                walk.pasuv[i] = 10.;
            }
            walk.pas_init[i] = walk.pasuv[i];
            walk.pas_sav[i] = walk.pasuv[i];
        }
        Ok(walk)
    }
    /// `IntWalk_PWalking(Caro1, Caro2, TolTangency, Epsilon, Deflection, Increment,
    /// U1, V1, U2, V2)`: widens the bounds of the non-periodic surfaces by `KELARG`
    /// steps, then computes the line from `(u1, v1, u2, v2)`.
    /// `IntWalkPWalking::with_start(&s1, &s2, [1e-6, 1e-8, 0.01, 0.1], [0.5, 0.5, 0.5, 0.5])?`.
    pub fn with_start(
        caro1: &Adaptor3dSurfaceHandle,
        caro2: &Adaptor3dSurfaceHandle,
        [tol_tangency, epsilon, deflection, increment]: [f64; 4],
        [u1, v1, u2, v2]: [f64; 4],
    ) -> Result<Self, Failure> {
        let mut walk: Self =
            Self::base(caro1, caro2, [tol_tangency, epsilon, deflection, increment])?;
        walk.load_resolutions(caro1, caro2, false)?;
        walk.load_steps();
        walk.widen_periodic(caro1, caro2, true)?;
        walk.pas_init = walk.pasuv;
        walk.pas_sav = walk.pasuv;
        walk.limit_resolutions();
        walk.my_step_min = walk.reso.map(|reso| 100.0 * reso);
        let mut par: Array1<'static, f64> = one_based(4)?;
        (par[1], par[2], par[3], par[4]) = (u1, v1, u2, v2);
        walk.perform(&par)?;
        Ok(walk)
    }
    /// The fields set by the initializer lists, and the bounds of both surfaces. The
    /// second constructor leaves the tangency flags and the boundary index
    /// uninitialized natively (Perform sets them); they start at false and 0 here.
    fn base(
        caro1: &Adaptor3dSurfaceHandle,
        caro2: &Adaptor3dSurfaceHandle,
        [tol_tangency, epsilon, deflection, increment]: [f64; 4],
    ) -> Result<Self, Failure> {
        let my_intersection_on2s = IntWalkTheInt2S::new(caro1, caro2, tol_tangency)?;
        let ([um1, vm1], [uu1, vv1]) = surface_bounds(caro1)?;
        let ([um2, vm2], [uu2, vv2]) = surface_bounds(caro2)?;
        Ok(Self {
            done: true,
            line: Handle::default(),
            close: false,
            tgfirst: false,
            tglast: false,
            my_tangent_idx: 0,
            tgdir: Direction3::default(),
            fleche: deflection,
            pas_max: increment * 0.2, // June 25 99, after problems with precision
            tolconf: epsilon,
            my_tol_tang: tol_tangency,
            pasuv: [0.0; 4],
            my_step_min: [0.0; 4],
            pas_sav: [0.0; 4],
            pas_init: [0.0; 4],
            lower: [um1, vm1, um2, vm2],
            upper: [uu1, vv1, uu2, vv2],
            reso: [0.0; 4],
            sens_cheminement: 1,
            choix_iso_sav: IntImpConstIsoparametric::default(),
            previous_point: IntSurfPntOn2S::default(),
            previoustg: false,
            previousd: Direction3::default(),
            previousd1: Direction2::default(),
            previousd2: Direction2::default(),
            firstd1: Direction2::default(),
            firstd2: Direction2::default(),
            my_intersection_on2s,
            static_blocage_sur_pas_trop_grand: 0,
            static_precedent_inflexion: 0,
        })
    }
    /// The resolutions of `Precision::Confusion()` in the order U1, U2, V1, V2 of the
    /// native scaling (the first constructor caps the scaled ones under 10).
    fn load_resolutions(
        &mut self,
        caro1: &Adaptor3dSurfaceHandle,
        caro2: &Adaptor3dSurfaceHandle,
        capped: bool,
    ) -> Result<(), Failure> {
        let conf: f64 = precision::confusion();
        self.reso = [
            tool::u_resolution(caro1, conf)?,
            tool::v_resolution(caro1, conf)?,
            tool::u_resolution(caro2, conf)?,
            tool::v_resolution(caro2, conf)?,
        ];
        for i in [0, 2, 1, 3] {
            self.reso[i] = scaled_resolution(self.reso[i], (self.lower[i], self.upper[i]), capped);
        }
        Ok(())
    }
    /// The maximal steps `pasMax * |UM - Um|`.
    fn load_steps(&mut self) {
        for i in 0..4 {
            self.pasuv[i] = self.pas_max * (self.upper[i] - self.lower[i]).abs();
        }
    }
    /// A resolution above a ten-thousandth of its step becomes a hundred-thousandth.
    fn limit_resolutions(&mut self) {
        for i in 0..4 {
            if self.reso[i] > 0.0001 * self.pasuv[i] {
                self.reso[i] = 0.00001 * self.pasuv[i];
            }
        }
    }
    /// Widens the bounds of a periodic parameter towards its period (at most `KELARG`
    /// steps on each side), and, when `widen_others`, the other bounds by `KELARG`
    /// steps.
    fn widen_periodic(
        &mut self,
        caro1: &Adaptor3dSurfaceHandle,
        caro2: &Adaptor3dSurfaceHandle,
        widen_others: bool,
    ) -> Result<(), Failure> {
        let periods: [Option<f64>; 4] = [
            period_of(tool::is_u_periodic(caro1)?, || tool::u_period(caro1))?,
            period_of(tool::is_v_periodic(caro1)?, || tool::v_period(caro1))?,
            period_of(tool::is_u_periodic(caro2)?, || tool::u_period(caro2))?,
            period_of(tool::is_v_periodic(caro2)?, || tool::v_period(caro2))?,
        ];
        for (i, period) in periods.into_iter().enumerate() {
            let Some(period) = period else {
                if widen_others {
                    self.upper[i] += KELARG * self.pasuv[i];
                    self.lower[i] -= KELARG * self.pasuv[i];
                }
                continue;
            };
            let mut t: f64 = self.upper[i] - self.lower[i];
            if t < period {
                t = 0.5 * (period - t);
                t = if t > KELARG * self.pasuv[i] {
                    KELARG * self.pasuv[i]
                } else {
                    t
                };
                self.upper[i] += t;
                self.lower[i] -= t;
            }
        }
        Ok(())
    }
    /// `ComputePasInit(theDeltaU1, theDeltaV1, theDeltaU2, theDeltaV2)`: the steps grow
    /// to twice the maximal relative step of the given parameter box (at least a
    /// hundredth of a finite walking range), the minimal steps to twice the
    /// resolutions of the tolerance, and no step goes under its minimum.
    pub fn compute_pas_init(&mut self, deltas: [f64; 4]) -> Result<(), Failure> {
        let range_part: f64 = 0.01;
        let increment: f64 = 2.0 * self.pas_max;
        let (caro1, caro2) = self.surfaces();
        for (i, the_delta) in deltas.into_iter().enumerate() {
            let delta: f64 = (self.upper[i] - self.lower[i]).abs();
            self.pasuv[i] = if !precision::is_infinite(delta) {
                std_max(
                    increment * std_max(the_delta, range_part * delta),
                    self.pasuv[i],
                )
            } else {
                std_max(increment * the_delta, self.pasuv[i])
            };
        }
        let resolutions: [f64; 4] = [
            tool::u_resolution(&caro1, self.tolconf)?,
            tool::v_resolution(&caro1, self.tolconf)?,
            tool::u_resolution(&caro2, self.tolconf)?,
            tool::v_resolution(&caro2, self.tolconf)?,
        ];
        for (step_min, resolution) in self.my_step_min.iter_mut().zip(resolutions) {
            *step_min = std_max(*step_min, 2.0 * resolution);
        }
        for i in 0..4 {
            self.pasuv[i] = std_max(self.my_step_min[i], self.pasuv[i]);
        }
        Ok(())
    }
}

/// The period of a parameter when it is periodic.
fn period_of(
    periodic: bool,
    period: impl FnOnce() -> Result<f64, Failure>,
) -> Result<Option<f64>, Failure> {
    if periodic {
        period().map(Some)
    } else {
        Ok(None)
    }
}
