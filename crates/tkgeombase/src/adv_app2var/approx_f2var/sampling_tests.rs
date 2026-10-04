use super::{SampledSquare, mma2ds1_};
use crate::adv_app2var::approx_f2var::SquareTables;
use crate::adv_app2var::evaluator::{
    AdvApp2VarEvaluation, AdvApp2VarEvaluationMut, AdvApp2VarEvaluatorFunc2Var,
};
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq)]
struct CallbackInputs {
    dimension: i32,
    iso: i32,
    count: i32,
    u_bounds: [f64; 2],
    v_bounds: [f64; 2],
    first_parameter: f64,
}

#[derive(Default)]
struct MutatingEvaluator {
    calls: RefCell<Vec<CallbackInputs>>,
}

impl AdvApp2VarEvaluatorFunc2Var for MutatingEvaluator {
    fn evaluate(&self, _: &AdvApp2VarEvaluation<'_>, _: &mut [f64], _: &mut i32) {
        unreachable!("mutable sample requests use evaluate_native")
    }

    fn evaluate_native(
        &self,
        call: &mut AdvApp2VarEvaluationMut<'_>,
        result: &mut [f64],
        error: &mut i32,
    ) {
        self.record_call(call);
        write_values(call, result);
        mutate_first_request(call, self.calls.borrow().len());
        *error = 0;
    }
}

impl MutatingEvaluator {
    fn record_call(&self, call: &AdvApp2VarEvaluationMut<'_>) {
        self.calls.borrow_mut().push(CallbackInputs {
            dimension: *call.dimension,
            iso: *call.favor_iso,
            count: *call.nb_params,
            u_bounds: *call.u_start_end,
            v_bounds: *call.v_start_end,
            first_parameter: call.parameters[0],
        });
    }
}

fn write_values(call: &AdvApp2VarEvaluationMut<'_>, result: &mut [f64]) {
    for point in 0..*call.nb_params as usize {
        write_point(call, result, point);
    }
}

fn write_point(call: &AdvApp2VarEvaluationMut<'_>, result: &mut [f64], point: usize) {
    for component in 0..*call.dimension as usize {
        let index = point * *call.dimension as usize + component;
        result[index] = *call.const_param + call.parameters[point] + component as f64;
    }
}

fn mutate_first_request(call: &mut AdvApp2VarEvaluationMut<'_>, call_count: usize) {
    if call_count != 1 {
        return;
    }
    *call.dimension = 1;
    *call.favor_iso = 1;
    *call.nb_params = 2;
    call.u_start_end[0] = -0.5;
    call.v_start_end[1] = 0.75;
    call.parameters[0] += 0.125;
}

#[test]
fn mma2ds2_reuses_mutable_input_pointers_between_rows() {
    let mut buffers = SamplingBuffers::new();
    let square = SampledSquare {
        ndimen: 2,
        uintfn: [0., 1.],
        vintfn: [0., 1.],
        nbpntu: 3,
        nbpntv: 3,
        urootb: &buffers.u_roots,
        vrootb: &buffers.v_roots,
        isofav: 2,
    };
    let mut tables = SquareTables {
        sosotb: &mut buffers.soso,
        disotb: &mut buffers.diso,
        soditb: &mut buffers.sodi,
        diditb: &mut buffers.didi,
    };
    let (mut fpntab, mut ttable) = (vec![7.5; 6], vec![7.5; 3]);
    let evaluator = MutatingEvaluator::default();
    let mut error = -1;
    mma2ds1_(
        &square,
        &evaluator,
        &mut tables,
        &mut fpntab,
        &mut ttable,
        &mut error,
    );
    assert_eq!(error, 0);
    assert_three_rows_share_the_mutated_state(&evaluator.calls.borrow());
}

struct SamplingBuffers {
    u_roots: Vec<f64>,
    v_roots: Vec<f64>,
    soso: Vec<f64>,
    diso: Vec<f64>,
    sodi: Vec<f64>,
    didi: Vec<f64>,
}

impl SamplingBuffers {
    fn new() -> Self {
        let (mut u_roots, mut v_roots) = (vec![0.; 3], vec![0.; 3]);
        crate::adv_app2var::approx_f2var::mma2roo_(3, 3, &mut u_roots, &mut v_roots);
        Self {
            u_roots,
            v_roots,
            soso: vec![0.25; 8],
            diso: vec![0.25; 2],
            sodi: vec![0.25; 2],
            didi: vec![0.25; 8],
        }
    }
}

fn assert_three_rows_share_the_mutated_state(calls: &[CallbackInputs]) {
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].dimension, 2);
    assert_eq!(calls[0].iso, 2);
    assert_eq!(calls[0].count, 3);
    for later in &calls[1..] {
        assert_eq!(later.dimension, 1);
        assert_eq!(later.iso, 1);
        assert_eq!(later.count, 2);
        assert_eq!(later.u_bounds, [-0.5, 1.]);
        assert_eq!(later.v_bounds, [0., 0.75]);
        assert_eq!(later.first_parameter, calls[0].first_parameter + 0.125);
    }
}
