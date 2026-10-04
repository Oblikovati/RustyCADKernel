//! Sampling of the function on the square: `mma2ds1_` and the file-static
//! `mma2ds2_`.
use super::discretise::{SquareTables, SquareViews};
use crate::adv_app2var::evaluator::{AdvApp2VarEvaluationMut, AdvApp2VarEvaluatorFunc2Var};
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::math_base::mmfmtb1_;

/// The square of `mma2ds1_`: the parameter intervals, the Legendre roots on
/// [-1,1] and the favourite iso direction.
pub struct SampledSquare<'a> {
    /// `ndimen`: dimension of the function.
    pub ndimen: i32,
    /// `uintfn`: the U interval.
    pub uintfn: [f64; 2],
    /// `vintfn`: the V interval.
    pub vintfn: [f64; 2],
    /// `nbpntu`: number of U roots.
    pub nbpntu: i32,
    /// `nbpntv`: number of V roots.
    pub nbpntv: i32,
    /// `urootb`: the U roots.
    pub urootb: &'a [f64],
    /// `vrootb`: the V roots.
    pub vrootb: &'a [f64],
    /// `isofav`: 1 to evaluate U-isos, 2 (or anything else) for V-isos.
    pub isofav: i32,
}

/// Adds the samples of `foncnp` at the roots of the square to the four symmetric
/// tables (`mma2ds1_`), evaluating along the favourite iso direction; `fpntab`
/// (`ndimen * max(nbpntu, nbpntv)`) and `ttable` are scratch. `iercod` is the
/// evaluator code plus 100 when it stops the sampling; returns 0.
///
/// Native quirk: for `isofav == 1` the tables are transposed back with
/// `mmfmtb1_` after the sampling whatever its outcome, and that call clears
/// `iercod`, so an evaluator failure is lost; the port keeps this.
/// `mma2ds1_(&square, &f, &mut tables, &mut fpntab, &mut ttable, &mut ier)`.
pub fn mma2ds1_(
    square: &SampledSquare<'_>,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    tables: &mut SquareTables<'_>,
    fpntab: &mut [f64],
    ttable: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    let iuouv: i32 = if square.isofav < 1 || square.isofav > 2 {
        2
    } else {
        square.isofav
    };
    if iuouv == 2 {
        mma2ds2_(square, iuouv, foncnp, tables, fpntab, ttable, iercod);
    } else if transpose_tables(
        square.ndimen,
        [square.nbpntu, square.nbpntv],
        tables,
        iercod,
    ) {
        let swapped = SampledSquare {
            uintfn: square.vintfn,
            vintfn: square.uintfn,
            nbpntu: square.nbpntv,
            nbpntv: square.nbpntu,
            urootb: square.vrootb,
            vrootb: square.urootb,
            ..*square
        };
        // The transposed soditb plays disotb's role and conversely.
        let mut transposed = SquareTables {
            sosotb: &mut *tables.sosotb,
            disotb: &mut *tables.soditb,
            soditb: &mut *tables.disotb,
            diditb: &mut *tables.diditb,
        };
        mma2ds2_(
            &swapped,
            iuouv,
            foncnp,
            &mut transposed,
            fpntab,
            ttable,
            iercod,
        );
        transpose_tables(
            square.ndimen,
            [square.nbpntv, square.nbpntu],
            tables,
            iercod,
        );
    }
    if *iercod > 0 {
        *iercod += 100;
    }
    0
}

/// The `mmfmtb1_` transpositions of `mma2ds1_`, per dimension, of the four tables
/// sampled on `nbpnt = [first, second]` roots; false when one fails.
fn transpose_tables(
    ndimen: i32,
    nbpnt: [i32; 2],
    tables: &mut SquareTables<'_>,
    iercod: &mut i32,
) -> bool {
    let full: [i32; 2] = [nbpnt[0] / 2 + 1, nbpnt[1] / 2 + 1];
    let half: [i32; 2] = [nbpnt[0] / 2, nbpnt[1] / 2];
    for nd in 1..=ndimen {
        let blocks: [(&mut [f64], [i32; 2]); 4] = [
            (&mut *tables.sosotb, full),
            (&mut *tables.diditb, full),
            (&mut *tables.soditb, half),
            (&mut *tables.disotb, half),
        ];
        for (table, [isz1, isz2]) in blocks {
            let start: usize = ((nd - 1) * isz1 * isz2) as usize;
            // In place natively: mmfmtb1_ transposes through its own scratch table.
            let block: Vec<f64> = table[start..start + (isz1 * isz2) as usize].to_vec();
            let (mut ibid1, mut ibid2) = (0, 0);
            mmfmtb1_(
                isz1,
                &block,
                isz1,
                isz2,
                isz2,
                &mut table[start..],
                &mut ibid1,
                &mut ibid2,
                iercod,
            );
            if *iercod > 0 {
                return false;
            }
        }
    }
    true
}

/// `mma2ds2_`: the sampling itself; like `mma2ds1_` it adds 100 to a positive
/// evaluator code, so a failure seen through `mma2ds1_` carries 200.
fn mma2ds2_(
    square: &SampledSquare<'_>,
    iiuouv: i32,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    tables: &mut SquareTables<'_>,
    fpntab: &mut [f64],
    ttable: &mut [f64],
    iercod: &mut i32,
) {
    *iercod = 0;
    sample_rows(square, iiuouv, foncnp, tables, fpntab, ttable, iercod);
    if *iercod > 0 {
        *iercod += 100;
    }
}

/// The rows of `mma2ds2_`, with the constant parameter along V: the V roots at
/// `-t`, then the middle, then `+t`, each folded into the tables; stops at the
/// first positive evaluator code.
fn sample_rows(
    square: &SampledSquare<'_>,
    iiuouv: i32,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    tables: &mut SquareTables<'_>,
    fpntab: &mut [f64],
    ttable: &mut [f64],
    iercod: &mut i32,
) {
    let mut rows = SquareRows::new(square, iiuouv, foncnp, tables, fpntab, ttable, iercod);
    rows.run();
}

struct SquareRows<'a, 'b> {
    square: &'a SampledSquare<'b>,
    sampler: RowSampler<'a>,
    state: MutableSquareState,
    tables: SquareViews<'a>,
    fpntab: &'a mut [f64],
    ttable: &'a mut [f64],
    iercod: &'a mut i32,
    alinv: f64,
    blinv: f64,
}

impl<'a, 'b> SquareRows<'a, 'b> {
    fn new(
        square: &'a SampledSquare<'b>,
        iiuouv: i32,
        foncnp: &'a dyn AdvApp2VarEvaluatorFunc2Var,
        tables: &'a mut SquareTables<'_>,
        fpntab: &'a mut [f64],
        ttable: &'a mut [f64],
        iercod: &'a mut i32,
    ) -> Self {
        initialize_parameter_roots(square, ttable);
        let state = mutable_square_state(square, iiuouv);
        let views = SquareViews::new(tables, square.nbpntu, square.nbpntv);
        Self::from_parts(square, foncnp, state, views, fpntab, ttable, iercod)
    }

    fn from_parts(
        square: &'a SampledSquare<'b>,
        foncnp: &'a dyn AdvApp2VarEvaluatorFunc2Var,
        state: MutableSquareState,
        tables: SquareViews<'a>,
        fpntab: &'a mut [f64],
        ttable: &'a mut [f64],
        iercod: &'a mut i32,
    ) -> Self {
        let v = square.vintfn;
        Self {
            square,
            sampler: RowSampler { foncnp },
            state,
            tables,
            fpntab,
            ttable,
            iercod,
            alinv: (v[1] - v[0]) / 2.,
            blinv: (v[1] + v[0]) / 2.,
        }
    }

    fn run(&mut self) {
        self.sample_negative_rows();
        if *self.iercod > 0 || !self.sample_middle_row() {
            return;
        }
        self.sample_positive_rows();
    }

    fn sample_negative_rows(&mut self) {
        for iv in 1..=self.square.nbpntv / 2 {
            let root = F2cArray::new(self.square.vrootb, 1)[iv];
            let row = self.nvroo() - iv + 1;
            if !self.sample_and_fold(self.blinv + self.alinv * root, row, -1.) {
                return;
            }
        }
    }

    fn sample_middle_row(&mut self) -> bool {
        if self.square.nbpntv % 2 == 0 {
            return true;
        }
        if !self.sampler.row(
            &mut self.state,
            self.blinv,
            self.ttable,
            self.fpntab,
            self.iercod,
        ) {
            return false;
        }
        fold_middle_row(&mut self.tables, self.fpntab, self.square, &self.state);
        true
    }

    fn sample_positive_rows(&mut self) {
        let nvroo = self.nvroo();
        for iv in 1..=nvroo {
            let root_index = (self.square.nbpntv + 1) / 2 + iv;
            let root = F2cArray::new(self.square.vrootb, 1)[root_index];
            let constant = self.alinv * root + self.blinv;
            if !self.sample_and_fold(constant, iv, 1.) {
                return;
            }
        }
    }

    fn sample_and_fold(&mut self, constant: f64, row: i32, sign: f64) -> bool {
        if !self.sampler.row(
            &mut self.state,
            constant,
            self.ttable,
            self.fpntab,
            self.iercod,
        ) {
            return false;
        }
        fold_row(
            &mut self.tables,
            self.fpntab,
            self.square,
            &self.state,
            row,
            sign,
        );
        true
    }

    fn nvroo(&self) -> i32 {
        self.square.nbpntv / 2
    }
}

fn initialize_parameter_roots(square: &SampledSquare<'_>, ttable: &mut [f64]) {
    let u = square.uintfn;
    let (half_span, center) = ((u[1] - u[0]) / 2., (u[1] + u[0]) / 2.);
    let roots = F2cArray::new(square.urootb, 1);
    let mut params = F2cArrayMut::new(ttable, 1);
    for index in 1..=square.nbpntu {
        params[index] = center + half_span * roots[index];
    }
}

fn mutable_square_state(square: &SampledSquare<'_>, iso: i32) -> MutableSquareState {
    let (first, second) = if iso == 1 {
        (square.vintfn, square.uintfn)
    } else {
        (square.uintfn, square.vintfn)
    };
    MutableSquareState::new(square, iso, first, second)
}

/// One `Evaluate` call of `mma2ds2_` along the U roots at the constant `tcons`.
struct MutableSquareState {
    dimension: i32,
    parameter_count: i32,
    favor_iso: i32,
    intervals: [[f64; 2]; 2],
    derivative_order: i32,
}

impl MutableSquareState {
    fn new(
        square: &SampledSquare<'_>,
        favor_iso: i32,
        first_interval: [f64; 2],
        second_interval: [f64; 2],
    ) -> Self {
        Self {
            dimension: square.ndimen,
            parameter_count: square.nbpntu,
            favor_iso,
            intervals: [first_interval, second_interval],
            derivative_order: 0,
        }
    }
}

struct RowSampler<'a> {
    foncnp: &'a dyn AdvApp2VarEvaluatorFunc2Var,
}

impl RowSampler<'_> {
    /// Evaluates the row; false when the evaluator returns a positive code.
    fn row(
        &self,
        state: &mut MutableSquareState,
        constant: f64,
        parameters: &mut [f64],
        fpntab: &mut [f64],
        iercod: &mut i32,
    ) -> bool {
        evaluate_row(self.foncnp, state, constant, parameters, fpntab, iercod);
        *iercod <= 0
    }
}

fn evaluate_row(
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    state: &mut MutableSquareState,
    constant: f64,
    parameters: &mut [f64],
    fpntab: &mut [f64],
    iercod: &mut i32,
) {
    let mut u_order = state.derivative_order;
    let mut v_order = state.derivative_order;
    let mut const_param = constant;
    let [u_start_end, v_start_end] = &mut state.intervals;
    let mut call = AdvApp2VarEvaluationMut {
        dimension: &mut state.dimension,
        u_start_end,
        v_start_end,
        favor_iso: &mut state.favor_iso,
        const_param: &mut const_param,
        nb_params: &mut state.parameter_count,
        parameters,
        u_order: &mut u_order,
        v_order: &mut v_order,
    };
    foncnp.evaluate_native(&mut call, fpntab, iercod);
    state.derivative_order = changed_order(state.derivative_order, u_order, v_order);
}

fn changed_order(previous: i32, u_order: i32, v_order: i32) -> i32 {
    if u_order != previous {
        u_order
    } else {
        v_order
    }
}

/// Folds one row of samples at V root row `jv` into the tables: `sign` is -1 for
/// the rows at `-t` (subtracted in the V differences) and +1 for those at `+t`.
fn fold_row(
    t: &mut SquareViews<'_>,
    fpntab: &[f64],
    square: &SampledSquare<'_>,
    state: &MutableSquareState,
    jv: i32,
    sign: f64,
) {
    let ndimen: i32 = state.dimension;
    let fpnt = F2cArray::new(fpntab, square.ndimen + 1);
    let nuroo: i32 = square.nbpntu / 2;
    let jdec: i32 = (square.nbpntu + 1) / 2;
    for id in 1..=ndimen {
        for iu in 1..=nuroo {
            let up: f64 = fpnt[id + (iu + jdec) * square.ndimen];
            let um: f64 = fpnt[id + (nuroo - iu + 1) * square.ndimen];
            let (f, h) = (t.f(iu, jv, id), t.h(iu, jv, id));
            t.soso[f] = t.soso[f] + up + um;
            t.diso[h] = t.diso[h] + up - um;
            if sign < 0. {
                t.sodi[h] = t.sodi[h] - up - um;
                t.didi[f] = t.didi[f] - up + um;
            } else {
                t.sodi[h] = t.sodi[h] + up + um;
                t.didi[f] = t.didi[f] + up - um;
            }
        }
        if state.parameter_count % 2 != 0 {
            let up: f64 = fpnt[id + jdec * square.ndimen];
            let f: i32 = t.f(0, jv, id);
            t.soso[f] += up;
            if sign < 0. {
                t.didi[f] -= up;
            } else {
                t.didi[f] += up;
            }
        }
    }
}

/// Folds the row at the middle V root of an odd count into the `jv = 0` entries.
fn fold_middle_row(
    t: &mut SquareViews<'_>,
    fpntab: &[f64],
    square: &SampledSquare<'_>,
    state: &MutableSquareState,
) {
    let ndimen: i32 = state.dimension;
    let fpnt = F2cArray::new(fpntab, square.ndimen + 1);
    let nuroo: i32 = square.nbpntu / 2;
    let jdec: i32 = (square.nbpntu + 1) / 2;
    for id in 1..=ndimen {
        for iu in 1..=nuroo {
            let up: f64 = fpnt[id + (jdec + iu) * square.ndimen];
            let um: f64 = fpnt[id + (nuroo - iu + 1) * square.ndimen];
            let f: i32 = t.f(iu, 0, id);
            t.soso[f] = t.soso[f] + up + um;
            t.didi[f] = t.didi[f] + up - um;
        }
        if state.parameter_count % 2 != 0 {
            let up: f64 = fpnt[id + jdec * square.ndimen];
            let f: i32 = t.f(0, 0, id);
            t.soso[f] += up;
        }
    }
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod tests;
