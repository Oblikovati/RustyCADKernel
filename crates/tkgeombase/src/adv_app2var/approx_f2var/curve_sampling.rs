use super::super::curve::IsoApproxRequest;
use crate::adv_app2var::evaluator::{AdvApp2VarEvaluationMut, AdvApp2VarEvaluatorFunc2Var};
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::math_base::pow__di;

/// The per-piece tables of `mma1fdi_`, `mma1cdi_` and `mma1cnt_`: the sums and
/// differences `(0:nbroot/2, ndimen)` and the end constraints `(ndimen, iordre+1)`.
pub(crate) struct PieceTables<'a> {
    pub(crate) somtab: &'a mut [f64],
    pub(crate) diftab: &'a mut [f64],
    pub(crate) contr1: &'a mut [f64],
    pub(crate) contr2: &'a mut [f64],
}

/// `mma1fdi_`: samples the derivative `ideriv` of the function across the iso at
/// the parameters `ttable` into `fpntab`, folds the samples into sums and
/// differences and reads the end constraints up to order `iordre`, all scaled to
/// [-1,1]. `iercod` is the evaluator code plus 100 when it stops the sampling.
pub(crate) fn mma1fdi_(
    req: &IsoApproxRequest<'_>,
    uvfonc: &[f64; 4],
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    ttable: &mut [f64],
    fpntab: &mut [f64],
    piece: &mut PieceTables<'_>,
    iercod: &mut i32,
) {
    *iercod = 0;
    sample_iso(req, uvfonc, foncnp, ttable, fpntab, piece, iercod);
    if *iercod > 0 {
        *iercod += 100;
    }
}

fn sample_iso(
    req: &IsoApproxRequest<'_>,
    uvfonc: &[f64; 4],
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    ttable: &mut [f64],
    fpntab: &mut [f64],
    piece: &mut PieceTables<'_>,
    iercod: &mut i32,
) {
    let (ndimen, nbroot, iordre) = (req.ndimen, req.nbroot, req.iordre);
    let (ideb, ifin) = sampling_limits(iordre, nbroot);
    let (u_axis, span) = sampling_axis(req.isofav, uvfonc);
    let renor = pow__di(span / 2., req.ideriv);
    let mut state = sampling_call(ndimen, *uvfonc, req, ifin - ideb + 1, u_axis);
    let fd1 = ndimen;
    initial_sample(&mut state, foncnp, ttable, fpntab, ideb, fd1, iercod);
    if *iercod > 0 {
        return;
    }
    fold_samples(state.dimension, fd1, nbroot, renor, fpntab, piece);
    if iordre < 0 {
        return;
    }
    finish_endpoint_samples(
        &mut state, req, foncnp, uvfonc, renor, ttable, fpntab, piece, iercod,
    );
}

fn sampling_limits(iordre: i32, nbroot: i32) -> (i32, i32) {
    if iordre >= 0 {
        (0, nbroot + 1)
    } else {
        (1, nbroot)
    }
}

fn sampling_axis(isofav: i32, uvfonc: &[f64; 4]) -> (bool, f64) {
    if isofav == 1 {
        (true, uvfonc[1] - uvfonc[0])
    } else {
        (false, uvfonc[3] - uvfonc[2])
    }
}

fn sampling_call(
    dimension: i32,
    uvfonc: [f64; 4],
    req: &IsoApproxRequest<'_>,
    nb_params: i32,
    u_axis: bool,
) -> SamplingCall {
    let orders = if u_axis {
        (req.ideriv, 0)
    } else {
        (0, req.ideriv)
    };
    SamplingCall::new(dimension, uvfonc, req, nb_params, orders.0, orders.1)
}

fn initial_sample(
    state: &mut SamplingCall,
    func: &dyn AdvApp2VarEvaluatorFunc2Var,
    ttable: &mut [f64],
    fpntab: &mut [f64],
    first: i32,
    stride: i32,
    error_code: &mut i32,
) {
    let parameter_start = first as usize;
    let result_start = (first * stride) as usize;
    state.evaluate(
        func,
        &mut ttable[parameter_start..],
        &mut fpntab[result_start..],
        error_code,
    );
}

fn fill_endpoint_constraints(
    state: &SamplingCall,
    renor: f64,
    nbroot: i32,
    stride: i32,
    fpntab: &[f64],
    piece: &mut PieceTables<'_>,
) {
    let samples = F2cArray::new(fpntab, 1);
    let (mut first, mut second) = (
        F2cArrayMut::new(piece.contr1, stride + 1),
        F2cArrayMut::new(piece.contr2, stride + 1),
    );
    for component in 1..=state.dimension {
        first[component + stride] = renor * samples[component];
        second[component + stride] = renor * samples[component + (nbroot + 1) * stride];
    }
}

fn finish_endpoint_samples(
    state: &mut SamplingCall,
    req: &IsoApproxRequest<'_>,
    func: &dyn AdvApp2VarEvaluatorFunc2Var,
    uvfonc: &[f64; 4],
    renor: f64,
    ttable: &mut [f64],
    fpntab: &[f64],
    piece: &mut PieceTables<'_>,
    error_code: &mut i32,
) {
    fill_endpoint_constraints(state, renor, req.nbroot, req.ndimen, fpntab, piece);
    evaluate_endpoint_derivatives(state, req, func, ttable, error_code, piece);
    if *error_code <= 0 {
        normalize_endpoint_derivatives(state, uvfonc, renor, req, piece);
    }
}

fn evaluate_endpoint_derivatives(
    state: &mut SamplingCall,
    req: &IsoApproxRequest<'_>,
    func: &dyn AdvApp2VarEvaluatorFunc2Var,
    ttable: &mut [f64],
    error_code: &mut i32,
    piece: &mut PieceTables<'_>,
) {
    state.nb_params = 1;
    let varying_v = state.favor_iso == 1;
    let mut sampler = EndpointSampler::new(func, ttable, req, varying_v);
    sampler.evaluate(state, 0, piece.contr1, error_code);
    if *error_code > 0 {
        return;
    }
    sampler.evaluate(state, req.nbroot + 1, piece.contr2, error_code);
}

struct EndpointSampler<'a> {
    func: &'a dyn AdvApp2VarEvaluatorFunc2Var,
    parameters: &'a mut [f64],
    max_order: i32,
    stride: i32,
    varying_v: bool,
}

impl<'a> EndpointSampler<'a> {
    fn new(
        func: &'a dyn AdvApp2VarEvaluatorFunc2Var,
        parameters: &'a mut [f64],
        req: &IsoApproxRequest<'_>,
        varying_v: bool,
    ) -> Self {
        Self {
            func,
            parameters,
            max_order: req.iordre,
            stride: req.ndimen,
            varying_v,
        }
    }

    fn evaluate(
        &mut self,
        state: &mut SamplingCall,
        parameter: i32,
        constraints: &mut [f64],
        error_code: &mut i32,
    ) {
        set_cross_order(state, self.varying_v);
        let mut values = F2cArrayMut::new(constraints, self.stride + 1);
        evaluate_cross_orders(
            state,
            self.func,
            &mut self.parameters[parameter as usize..],
            &mut values,
            self.max_order,
            self.stride,
            self.varying_v,
            error_code,
        );
    }
}

fn set_cross_order(state: &mut SamplingCall, varying_v: bool) {
    let selected = if varying_v {
        &mut state.v_order
    } else {
        &mut state.u_order
    };
    *selected = 1;
}

fn normalize_endpoint_derivatives(
    state: &SamplingCall,
    uvfonc: &[f64; 4],
    renor: f64,
    req: &IsoApproxRequest<'_>,
    piece: &mut PieceTables<'_>,
) {
    let span = if state.favor_iso == 1 {
        uvfonc[3] - uvfonc[2]
    } else {
        uvfonc[1] - uvfonc[0]
    };
    let (mut first, mut second) = (
        F2cArrayMut::new(piece.contr1, req.ndimen + 1),
        F2cArrayMut::new(piece.contr2, req.ndimen + 1),
    );
    let mut scale = renor;
    for order in 1..=req.iordre {
        scale *= span / 2.;
        scale_endpoint_order(
            order,
            state.dimension,
            req.ndimen,
            scale,
            &mut first,
            &mut second,
        );
    }
}

fn scale_endpoint_order(
    order: i32,
    dimension: i32,
    stride: i32,
    scale: f64,
    first: &mut F2cArrayMut<'_, f64>,
    second: &mut F2cArrayMut<'_, f64>,
) {
    for component in 1..=dimension {
        first[component + (order + 1) * stride] *= scale;
        second[component + (order + 1) * stride] *= scale;
    }
}

struct SamplingCall {
    dimension: i32,
    u_start_end: [f64; 2],
    v_start_end: [f64; 2],
    favor_iso: i32,
    const_param: f64,
    nb_params: i32,
    u_order: i32,
    v_order: i32,
}

impl SamplingCall {
    fn new(
        dimension: i32,
        uvfonc: [f64; 4],
        req: &IsoApproxRequest<'_>,
        nb_params: i32,
        u_order: i32,
        v_order: i32,
    ) -> Self {
        Self {
            dimension,
            u_start_end: [uvfonc[0], uvfonc[1]],
            v_start_end: [uvfonc[2], uvfonc[3]],
            favor_iso: req.isofav,
            const_param: req.tconst,
            nb_params,
            u_order,
            v_order,
        }
    }

    fn evaluate(
        &mut self,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        parameters: &mut [f64],
        result: &mut [f64],
        error_code: &mut i32,
    ) {
        let mut call = AdvApp2VarEvaluationMut {
            dimension: &mut self.dimension,
            u_start_end: &mut self.u_start_end,
            v_start_end: &mut self.v_start_end,
            favor_iso: &mut self.favor_iso,
            const_param: &mut self.const_param,
            nb_params: &mut self.nb_params,
            parameters,
            u_order: &mut self.u_order,
            v_order: &mut self.v_order,
        };
        func.evaluate_native(&mut call, result, error_code);
    }
}

fn evaluate_cross_orders(
    state: &mut SamplingCall,
    func: &dyn AdvApp2VarEvaluatorFunc2Var,
    parameters: &mut [f64],
    table: &mut F2cArrayMut<'_, f64>,
    max_order: i32,
    stride: i32,
    varying_v: bool,
    error_code: &mut i32,
) {
    while cross_order(state, varying_v) <= max_order {
        let order = cross_order(state, varying_v);
        let result_start = (order + 1) * stride + 1;
        state.evaluate(func, parameters, table.tail_mut(result_start), error_code);
        if *error_code > 0 {
            return;
        }
        increment_cross_order(state, varying_v);
    }
}

fn cross_order(state: &SamplingCall, varying_v: bool) -> i32 {
    if varying_v {
        state.v_order
    } else {
        state.u_order
    }
}

fn increment_cross_order(state: &mut SamplingCall, varying_v: bool) {
    if varying_v {
        state.v_order += 1;
    } else {
        state.u_order += 1;
    }
}

/// The folding of `mma1fdi_`: sums and differences of the samples at `+t` and
/// `-t`, times `renor`, and the middle sample of an odd count.
fn fold_samples(
    ndimen: i32,
    stride: i32,
    nbroot: i32,
    renor: f64,
    fpntab: &[f64],
    piece: &mut PieceTables<'_>,
) {
    let fpnt = F2cArray::new(fpntab, 1);
    let sd1: i32 = nbroot / 2 + 1;
    let mut som = F2cArrayMut::new(piece.somtab, sd1);
    let mut dif = F2cArrayMut::new(piece.diftab, sd1);
    let nroo2: i32 = nbroot / 2;
    for nd in 1..=ndimen {
        for ii in 1..=nroo2 {
            let iip: i32 = (nbroot + 1) / 2 + ii;
            let iim: i32 = nroo2 - ii + 1;
            let (bid1, bid2) = (fpnt[nd + iim * stride], fpnt[nd + iip * stride]);
            som[ii + nd * sd1] = renor * (bid2 + bid1);
            dif[ii + nd * sd1] = renor * (bid2 - bid1);
        }
    }
    for nd in 1..=ndimen {
        let middle: f64 = if nbroot % 2 == 1 {
            renor * fpnt[nd + (nroo2 + 1) * stride]
        } else {
            0.
        };
        som[nd * sd1] = middle;
        dif[nd * sd1] = middle;
    }
}

#[cfg(test)]
#[path = "curve_steps_tests.rs"]
mod tests;
