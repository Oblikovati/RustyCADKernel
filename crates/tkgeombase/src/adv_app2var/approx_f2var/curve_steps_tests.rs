use super::*;
use crate::adv_app2var::AdvApp2VarEvaluation;
use std::cell::RefCell;

struct SkipSecondVOrder(RefCell<Vec<i32>>);

impl AdvApp2VarEvaluatorFunc2Var for SkipSecondVOrder {
    fn evaluate(&self, _: &AdvApp2VarEvaluation<'_>, _: &mut [f64], _: &mut i32) {
        unreachable!("test callback uses the native mutable protocol")
    }

    fn evaluate_native(
        &self,
        call: &mut AdvApp2VarEvaluationMut<'_>,
        _: &mut [f64],
        error: &mut i32,
    ) {
        self.0.borrow_mut().push(*call.v_order);
        if *call.v_order == 1 {
            *call.v_order = 2;
        }
        *error = 0;
    }
}

#[test]
fn mutable_v_order_is_incremented_after_the_callback_returns() {
    let func = SkipSecondVOrder(RefCell::new(Vec::new()));
    let mut state = sample_state();
    let (mut parameters, mut result, mut error) = ([0.5], [0.; 8], 0);
    let mut output = F2cArrayMut::new(&mut result, 1);
    evaluate_cross_orders(
        &mut state,
        &func,
        &mut parameters,
        &mut output,
        2,
        1,
        true,
        &mut error,
    );
    assert_eq!(func.0.borrow().as_slice(), &[1]);
    assert_eq!((state.v_order, error), (3, 0));
}

fn sample_state() -> SamplingCall {
    SamplingCall {
        dimension: 1,
        u_start_end: [0., 1.],
        v_start_end: [0., 1.],
        favor_iso: 1,
        const_param: 0.,
        nb_params: 1,
        u_order: 0,
        v_order: 1,
    }
}
