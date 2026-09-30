// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (TheMotor state)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The state of `TheMotor` across its phases and its transitions: the start, the
//! acceptance or the restore of an estimation or an iteration, and the progression.
use super::super::smooth_criterion::SmoothCriterion;
use super::motor_support::copy_of;
use super::project::ProjectErrors;
use crate::fem_tool::Curve;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};

const BIG_VALUE: f64 = 1.0e37;
pub(super) const SMALL_VALUE: f64 = 1.0e-6;
const SMALLEST_VALUE: f64 = 1.0e-9;

/// The local variables of TheMotor that live across its phases. The native
/// `VALCRI`, `ERRQUA`, `ERRMOY` and `NumPnt` start uninitialized; the port's at 0.
#[derive(Default)]
pub(super) struct MotorState {
    pub(super) vocri: [f64; 3],
    pub(super) valcri: [f64; 3],
    pub(super) erold: f64,
    pub(super) errors: ProjectErrors,
    pub(super) cblong: f64,
    pub(super) lnold: f64,
    pub(super) c_current: Handle<Curve>,
    pub(super) c_old: Handle<Curve>,
    pub(super) c_new: Handle<Curve>,
    pub(super) current_ti: Handle<HArray1<f64>>,
    pub(super) new_ti: Handle<HArray1<f64>>,
    pub(super) old_ti: Handle<HArray1<f64>>,
    pub(super) eps_length: f64,
    pub(super) isnear: bool,
    pub(super) lconst: bool,
    pub(super) j_min: [f64; 3],
    pub(super) w_quality: f64,
}

impl MotorState {
    // The state at (0) Init, from the curve, the working copy of the parameters and
    // the estimated length. The fields left to the default: VALCRI at 0, no new
    // curve or parameters yet, not near, and the minima and the quality weight the
    // caller sets.
    pub(super) fn start(
        the_curve: &Handle<Curve>,
        current_ti: Handle<HArray1<f64>>,
        cblong: f64,
    ) -> Result<Self, Failure> {
        Ok(Self {
            vocri: [BIG_VALUE; 3],
            erold: BIG_VALUE,
            errors: ProjectErrors {
                max_err: BIG_VALUE,
                ..ProjectErrors::default()
            },
            cblong,
            lnold: cblong,
            c_current: the_curve.clone(),
            c_old: the_curve.clone(),
            old_ti: copy_of(&current_ti)?,
            current_ti,
            eps_length: SMALL_VALUE,
            lconst: true,
            ..Self::default()
        })
    }
}

// The quality criteria of J above the minima into VALCRI (the (1.2) and (2.4)
// steps); an improved estimation (ICDANA > 0) makes the next assembly rebuild the
// Hessian. Returns ICDANA.
pub(super) fn quality_criteria(
    j: &dyn SmoothCriterion,
    st: &mut MotorState,
) -> Result<i32, Failure> {
    let quality = j.quality_values(st.j_min[0], st.j_min[1], st.j_min[2])?;
    st.valcri = quality.criteria;
    if quality.icdana > 0 {
        st.lconst = true;
    }
    Ok(quality.icdana)
}

// (3.1): the criteria, the error, the curve and the parameters saved before the
// cutting (the length is not).
pub(super) fn save_before_cutting(st: &mut MotorState) {
    st.vocri = st.valcri;
    st.erold = st.errors.max_err;
    st.c_old = st.c_current.clone();
    let values: Array1<'static, f64> = st.current_ti.array1().clone();
    st.old_ti.change_array1().assign(&values);
}

// (2.7) restore the previous iteration (its parameters copied back into the
// current array).
pub(super) fn restore_previous_iteration(st: &mut MotorState) {
    st.valcri = st.vocri;
    st.errors.max_err = st.erold;
    st.cblong = st.lnold;
    st.c_current = st.c_old.clone();
    let values: Array1<'static, f64> = st.old_ti.array1().clone();
    st.current_ti.change_array1().assign(&values);
}

// Iteration is Ok: its curve and parameters become the current ones.
pub(super) fn accept_iteration(st: &mut MotorState) {
    st.c_current = st.c_new.clone();
    st.current_ti = st.new_ti.clone();
}

// (1.5): the new knots are dropped and the state before the estimation restored.
pub(super) fn invalidate_new_knots(st: &mut MotorState) {
    st.valcri = st.vocri;
    st.errors.max_err = st.erold;
    st.cblong = st.lnold;
    st.c_current = st.c_old.clone();
    st.current_ti = st.old_ti.clone();
}

// The estimation is kept: its criteria, length and error become the reference, its
// curve and parameters the current ones.
pub(super) fn accept_estimation(st: &mut MotorState) {
    (st.vocri, st.lnold, st.erold) = (st.valcri, st.cblong, st.errors.max_err);
    st.c_current = st.c_new.clone();
    st.current_ti = st.new_ti.clone();
}

// (1.4) Progression's test.
pub(super) fn progression(st: &MotorState) -> i32 {
    let (erold, errmax, wq) = (st.erold, st.errors.max_err, st.w_quality);
    let (valcri, vocri) = (st.valcri, st.vocri);
    let mut iprog: i32 = 0;
    iprog += i32::from(erold > wq && errmax < 0.95 * erold);
    iprog += i32::from(erold > wq && errmax < 0.8 * erold);
    iprog += i32::from(erold > wq && errmax < wq);
    iprog += i32::from(erold > wq && errmax < 0.99 * erold && errmax < 1.1 * wq);
    iprog += i32::from(valcri[0] < 0.975 * vocri[0]);
    iprog += i32::from(valcri[0] < 0.9 * vocri[0]);
    iprog += i32::from(valcri[1] < 0.95 * vocri[1]);
    iprog += i32::from(valcri[1] < 0.8 * vocri[1]);
    iprog += i32::from(valcri[2] < 0.95 * vocri[2]);
    iprog += i32::from(valcri[2] < 0.8 * vocri[2]);
    if vocri[1] > SMALLEST_VALUE
        && vocri[2] > SMALLEST_VALUE
        && valcri[1] / vocri[1] + 2. * valcri[2] / vocri[2] < 2.8
    {
        iprog += 1;
    }
    iprog
}
