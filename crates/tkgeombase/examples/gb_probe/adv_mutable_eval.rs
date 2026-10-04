//! Native callback-pointer mutation coverage for `mma1fdi_`.
use super::adv_math::fill;
use super::common::{end, int};
use rustycad_tkgeombase::adv_app2var::approx_f2var::{
    self as f2, IsoApproxOutput, IsoApproxRequest, mma2roo_,
};
use rustycad_tkgeombase::adv_app2var::{
    AdvApp2VarEvaluation, AdvApp2VarEvaluationMut, AdvApp2VarEvaluatorFunc2Var,
};
use std::cell::RefCell;

#[derive(Default)]
struct MutatingOrder {
    calls: RefCell<Vec<[i32; 3]>>,
}

impl AdvApp2VarEvaluatorFunc2Var for MutatingOrder {
    fn evaluate(&self, _: &AdvApp2VarEvaluation<'_>, _: &mut [f64], _: &mut i32) {
        unreachable!("native mutable callback must dispatch through evaluate_native")
    }

    fn evaluate_native(
        &self,
        call: &mut AdvApp2VarEvaluationMut<'_>,
        result: &mut [f64],
        error: &mut i32,
    ) {
        self.calls
            .borrow_mut()
            .push([*call.nb_params, *call.u_order, *call.v_order]);
        write_samples(call, result);
        skip_second_v_order(call);
        *error = 0;
    }
}

fn write_samples(call: &AdvApp2VarEvaluationMut<'_>, result: &mut [f64]) {
    for i in 0..*call.nb_params as usize {
        result[i * *call.dimension as usize] = *call.const_param
            + call.parameters[i]
            + 10. * f64::from(*call.u_order)
            + 100. * f64::from(*call.v_order);
    }
}

fn skip_second_v_order(call: &mut AdvApp2VarEvaluationMut<'_>) {
    if *call.nb_params == 1 && *call.u_order == 0 && *call.v_order == 1 {
        *call.v_order = 2;
    }
}

pub fn mma2fnc_mut_order_case(_: &[f64]) {
    let (mut roots, mut other_roots) = (fill(8), fill(8));
    mma2roo_(8, 8, &mut roots, &mut other_roots);
    let mut buffers = OutputBuffers::new();
    let mut ier = -9;
    let request = iso_request(&roots);
    let mut output = buffers.output();
    let function = MutatingOrder::default();
    f2::mma2fnc_(&request, &function, &mut output, &mut ier);
    int(ier);
    int(output.nbcrbe);
    print_calls(&function.calls.borrow());
    end();
}

fn iso_request(roots: &[f64]) -> IsoApproxRequest<'_> {
    IsoApproxRequest {
        ndimen: 1,
        nbsesp: 1,
        ndimse: &[1],
        uvfonc: [0., 1., 0., 1.],
        tconst: 0.3,
        isofav: 1,
        nbroot: 8,
        rootlg: roots,
        iordre: 2,
        ideriv: 0,
        ndgjac: 12,
        nbcrmx: 2,
        ncflim: 10,
        epsapr: &[1e-6],
    }
}

struct OutputBuffers {
    ncoeff: Vec<i32>,
    curve: Vec<f64>,
    sums: Vec<f64>,
    differences: Vec<f64>,
    first: Vec<f64>,
    second: Vec<f64>,
    cuts: Vec<f64>,
    max_errors: Vec<f64>,
    mean_errors: Vec<f64>,
}

impl OutputBuffers {
    fn new() -> Self {
        Self {
            ncoeff: vec![-1; 2],
            curve: fill(20),
            sums: fill(10),
            differences: fill(10),
            first: fill(8),
            second: fill(8),
            cuts: fill(3),
            max_errors: fill(2),
            mean_errors: fill(2),
        }
    }

    fn output(&mut self) -> IsoApproxOutput<'_> {
        IsoApproxOutput {
            ncoeff: &mut self.ncoeff,
            courbe: &mut self.curve,
            nbcrbe: -1,
            somtab: &mut self.sums,
            diftab: &mut self.differences,
            contr1: &mut self.first,
            contr2: &mut self.second,
            tabdec: &mut self.cuts,
            errmax: &mut self.max_errors,
            errmoy: &mut self.mean_errors,
        }
    }
}

fn print_calls(calls: &[[i32; 3]]) {
    int(calls.len() as i32);
    for call in calls {
        for value in call {
            int(*value);
        }
    }
}
