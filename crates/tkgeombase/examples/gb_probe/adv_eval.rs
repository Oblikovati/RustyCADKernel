//! The AdvApp2Var test functions; the counterpart of
//! tools/oracle/tkgeombase_probe_advapp2var_eval.cxx.
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarEvaluation, AdvApp2VarEvaluatorFunc2Var};

/// One factor of a separable term: 0 `x^a`, 1 `cos(a x)`, 2 `sin(a x)`, 3 `exp(a x)`.
#[derive(Clone, Copy)]
struct Factor(i32, f64);

/// A term `c * g(u) * h(v)` of component `comp`.
#[derive(Clone, Copy)]
struct Term(usize, f64, Factor, Factor);

fn factor(f: Factor, k: i32, x: f64) -> f64 {
    let Factor(kind, a) = f;
    if kind == 0 {
        let n = a as i32;
        if k > n {
            return 0.;
        }
        let mut c = 1.;
        for i in 0..k {
            c *= f64::from(n - i);
        }
        let mut p = 1.;
        for _ in 0..n - k {
            p *= x;
        }
        return c * p;
    }
    let mut s = 1.;
    for _ in 0..k {
        s *= a;
    }
    let t = a * x;
    if kind == 3 {
        return s * t.exp();
    }
    let phase = (if kind == 1 { k } else { k + 3 }) % 4;
    let v = match phase {
        0 => t.cos(),
        1 => -t.sin(),
        2 => -t.cos(),
        _ => t.sin(),
    };
    s * v
}

fn terms(id: i32) -> Vec<Term> {
    let p = |n: f64| Factor(0, n);
    match id {
        0 => vec![
            Term(0, 1., p(1.), p(0.)),
            Term(1, 1., p(0.), p(1.)),
            Term(2, 1., p(1.), p(1.)),
            Term(2, 0.5, p(2.), p(0.)),
        ],
        1 => vec![
            Term(0, 1., Factor(1, 1.), Factor(1, 1.)),
            Term(1, 1., Factor(2, 1.), Factor(1, 1.)),
            Term(2, 1., p(0.), Factor(2, 1.)),
        ],
        2 => vec![
            Term(0, 1., p(1.), p(0.)),
            Term(1, 1., p(0.), p(1.)),
            Term(2, 1., Factor(3, 0.7), Factor(2, 2.)),
            Term(2, 0.3, p(2.), p(1.)),
        ],
        _ => vec![
            Term(0, 1., p(1.), p(0.)),
            Term(0, 0.2, p(0.), p(3.)),
            Term(1, 1., p(0.), p(1.)),
            Term(1, -0.1, p(2.), p(0.)),
            Term(2, 1., Factor(1, 1.), Factor(3, -0.5)),
        ],
    }
}

/// The probe function `id` (GbAaFunction).
pub struct Function(pub i32);

impl AdvApp2VarEvaluatorFunc2Var for Function {
    fn evaluate(
        &self,
        request: &AdvApp2VarEvaluation<'_>,
        result: &mut [f64],
        error_code: &mut i32,
    ) {
        let terms = terms(self.0 % 10);
        let dim = request.dimension as usize;
        for i in 0..request.nb_params as usize {
            let (u, v) = if request.favor_iso == 1 {
                (request.const_param, request.parameters[i])
            } else {
                (request.parameters[i], request.const_param)
            };
            for c in 0..dim {
                let mut sum = 0.;
                for t in terms.iter().filter(|t| t.0 == c % 3) {
                    sum += t.1 * factor(t.2, request.u_order, u) * factor(t.3, request.v_order, v);
                }
                result[c + i * dim] = sum;
            }
        }
        *error_code = if self.0 >= 10 && request.const_param > 0.5 {
            7
        } else {
            0
        };
    }
}
