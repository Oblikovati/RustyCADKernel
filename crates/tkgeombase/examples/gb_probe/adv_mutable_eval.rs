//! Native callback-pointer mutation coverage for the AdvApp2Var samplers.
use super::adv_f2var::{Square, roots};
use super::adv_math::{fill, print};
use super::common::{end, int, real};
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

pub fn mma2ds1_mut_state_case(_: &[f64]) {
    let (nu, nv, nd) = (3, 3, 2);
    let (ur, vr) = roots(nu, nv);
    let mut square = Square::new(nu, nv, nd, 0.25);
    let (mut fp, mut tt, mut ier) = (fill(nd * nu), fill(nu), -1);
    let request = f2::SampledSquare {
        ndimen: nd,
        uintfn: [0., 1.],
        vintfn: [0., 1.],
        nbpntu: nu,
        nbpntv: nv,
        urootb: &ur,
        vrootb: &vr,
        isofav: 2,
    };
    let function = MutatingSquareState::default();
    f2::mma2ds1_(
        &request,
        &function,
        &mut square.tables(),
        &mut fp,
        &mut tt,
        &mut ier,
    );
    int(ier);
    print_square_calls(&function.calls.borrow());
    print(&square.soso);
    print(&square.diso);
    print(&square.sodi);
    print(&square.didi);
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

#[derive(Default)]
struct MutatingSquareState {
    calls: RefCell<Vec<SquareCall>>,
}

#[derive(Clone, Copy)]
struct SquareCall {
    ints: [i32; 3],
    reals: [f64; 5],
}

impl AdvApp2VarEvaluatorFunc2Var for MutatingSquareState {
    fn evaluate(&self, _: &AdvApp2VarEvaluation<'_>, _: &mut [f64], _: &mut i32) {
        unreachable!("native mutable callback must dispatch through evaluate_native")
    }

    fn evaluate_native(
        &self,
        call: &mut AdvApp2VarEvaluationMut<'_>,
        result: &mut [f64],
        error: &mut i32,
    ) {
        self.record_and_write(call, result);
        self.mutate_first_call(call);
        *error = 0;
    }
}

impl MutatingSquareState {
    fn record_and_write(&self, call: &AdvApp2VarEvaluationMut<'_>, result: &mut [f64]) {
        self.record(call);
        for point in 0..*call.nb_params as usize {
            write_square_value(call, result, point);
        }
    }

    fn record(&self, call: &AdvApp2VarEvaluationMut<'_>) {
        self.calls.borrow_mut().push(SquareCall {
            ints: [*call.dimension, *call.favor_iso, *call.nb_params],
            reals: [
                call.u_start_end[0],
                call.u_start_end[1],
                call.v_start_end[0],
                call.v_start_end[1],
                *call.parameters.first().unwrap_or(&0.),
            ],
        });
    }

    fn mutate_first_call(&self, call: &mut AdvApp2VarEvaluationMut<'_>) {
        if self.calls.borrow().len() != 1 {
            return;
        }
        *call.dimension = 1;
        *call.favor_iso = 1;
        *call.nb_params = 2;
        call.u_start_end[0] = -0.5;
        call.v_start_end[1] = 0.75;
        call.parameters[0] += 0.125;
    }
}

fn write_square_value(call: &AdvApp2VarEvaluationMut<'_>, result: &mut [f64], point: usize) {
    for component in 0..*call.dimension as usize {
        let index = point * *call.dimension as usize + component;
        result[index] = 10. * *call.const_param
            + call.parameters[point]
            + call.u_start_end[0]
            + 3. * call.v_start_end[1]
            + 100. * component as f64;
    }
}

fn print_square_calls(calls: &[SquareCall]) {
    int(calls.len() as i32);
    for call in calls {
        call.ints.iter().for_each(|value| int(*value));
        call.reals.iter().for_each(|value| real(*value));
    }
}
