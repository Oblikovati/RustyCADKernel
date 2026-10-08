//! `ex_case <n>`: the direct API scenarios of tools/oracle/tkexpress_probe_cases.cxx.
use super::cases_functions::functions;
use super::common::ProbeLine;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkexpress::expr::handles::one_based;
use rustycad_tkexpress::expr::operators::{
    add_real, divide_real, expr_add, expr_divide, expr_multiply, expr_negate, expr_subtract,
    multiply_real, real_add, real_divide, real_multiply, real_subtract, subtract_real,
};
use rustycad_tkexpress::expr::{
    ExprAbsolute, ExprBinaryExpression, ExprCosine, ExprDifference, ExprDivision, ExprEqual,
    ExprExponentiate, ExprGeneralExpression, ExprHandle, ExprNamedConstant, ExprNamedExpression,
    ExprNamedUnknown, ExprNumericValue, ExprPolyExpression, ExprProduct, ExprRUIterator,
    ExprRelationHandle, ExprRelationIterator, ExprSign, ExprSine, ExprSum, ExprUnaryExpression,
    ExprUnaryMinus, ExprUnknownIterator,
};

/// `new Expr_NamedUnknown(name)` (`ex_x`).
pub fn unknown(name: &str) -> Handle<ExprNamedUnknown> {
    ExprNamedUnknown::new(&AsciiString::new(name))
}
/// `new Expr_NumericValue(value)` (`ex_v`).
pub fn value(value: f64) -> ExprHandle {
    ExprNumericValue::expression(value)
}

/// 0: SetOperand checks of the unary, binary and poly expressions.
fn operands(line: &mut ProbeLine) {
    let x: ExprHandle = unknown("x").cast();
    let s = ExprSine::new(&x);
    let m: ExprHandle = ExprUnaryMinus::new(&s.cast()).cast();
    line.step(|_| s.set_operand(&s.cast()));
    line.step(|_| s.set_operand(&m));
    line.step(|line| {
        s.set_operand(&value(2.0))?;
        Ok(line.text(&m.string()?))
    });
    let d = ExprDifference::new(&x, &value(1.0));
    let p = ExprSum::new_pair(&d.cast(), &x);
    line.step(|_| d.set_first_operand(&p.cast()));
    line.step(|_| d.set_second_operand(&d.cast()));
    line.step(|line| {
        d.set_second_operand(&value(3.0))?;
        Ok(line.text(&p.string()?))
    });
    line.step(|_| p.set_operand(&value(4.0), 3));
    line.step(|_| p.set_operand(&p.cast(), 1));
    line.step(|line| {
        p.set_operand(&value(5.0), 2)?;
        Ok(line.text(&p.string()?))
    });
    line.step(|line| Ok(line.text(&p.sub_expression(3)?.string()?)));
    line.step(|line| Ok(line.text(&d.sub_expression(0)?.string()?)));
    line.step(|line| Ok(line.text(&s.sub_expression(2)?.string()?)));
}

/// The first half of scenario 1: assignment and the members that follow it.
fn assigned_unknowns(
    line: &mut ProbeLine,
    x: &Handle<ExprNamedUnknown>,
    y: &Handle<ExprNamedUnknown>,
) {
    line.step(|line| Ok(line.text(&x.assigned_expression()?.string()?)));
    line.step(|line| Ok(line.text(&x.sub_expression(1)?.string()?)));
    line.step(|_| x.assign(&expr_add(&x.cast(), &value(1.0)).cast()));
    line.step(|line| {
        x.assign(&expr_add(&y.cast(), &value(1.0)).cast())?;
        line.boolean(x.is_assigned());
        line.int(x.nb_sub_expressions());
        Ok(())
    });
    line.step(|line| {
        line.text(&x.string()?);
        line.text(&x.simplified()?.string()?);
        Ok(line.text(&x.shallow_simplified()?.string()?))
    });
    line.step(|line| {
        line.boolean(x.contains_unknowns()?);
        line.boolean(x.contains(&y.cast())?);
        Ok(line.boolean(x.is_linear()?))
    });
    line.step(|line| {
        line.text(&x.derivative(y)?.string()?);
        Ok(line.text(&x.derivative(x)?.string()?))
    });
    line.step(|line| {
        let copy: ExprHandle = x.copy()?;
        line.text(&copy.string()?);
        line.boolean(copy.is_identical(&x.cast())?);
        Ok(line.boolean(copy == *x))
    });
}

/// `x->Evaluate` with the variables `vars` from bound `lower` and the values `vals`
/// from bound `values_lower`.
fn evaluate_with(
    line: &mut ProbeLine,
    x: &Handle<ExprNamedUnknown>,
    vars: (i32, Vec<Handle<ExprNamedUnknown>>),
    vals: (i32, Vec<f64>),
) {
    line.step(|line| {
        let mut variables = one_based(vars.1);
        variables.update_lower_bound(vars.0);
        let mut values = one_based(vals.1);
        values.update_lower_bound(vals.0);
        Ok(line.real(x.evaluate(&variables, &values)?))
    });
}

/// 1: named unknowns: assignment, its checks and the members that follow it.
fn unknowns(line: &mut ProbeLine) {
    let x = unknown("x");
    let y = unknown("y");
    assigned_unknowns(line, &x, &y);
    evaluate_with(line, &x, (1, vec![y.clone()]), (1, vec![2.5]));
    line.step(|line| {
        x.replace(&y, &value(7.0))?;
        Ok(line.text(&x.assigned_expression()?.string()?))
    });
    line.step(|line| {
        x.deassign();
        line.boolean(x.is_assigned());
        line.int(x.nb_sub_expressions());
        Ok(line.boolean(x.contains_unknowns()?))
    });
    evaluate_with(line, &x, (1, vec![y.clone()]), (1, vec![2.5]));
    evaluate_with(
        line,
        &x,
        (0, vec![y.clone(), x.clone()]),
        (5, vec![1.0, 9.0]),
    );
    line.step(|line| {
        let k: ExprHandle = ExprNamedConstant::new(&AsciiString::new("K"), 2.0).cast();
        line.text(&k.string()?);
        line.boolean(k.is_identical(&x.cast())?);
        line.text(&k.n_derivative(&x, 0)?.string()?);
        Ok(line.text(&k.simplified()?.string()?))
    });
}

/// 2: the N <= 0 checks of NDerivative and the derivative members of the operators.
fn derivatives(line: &mut ProbeLine) {
    let x = unknown("x");
    let xe: ExprHandle = x.cast();
    let s: ExprHandle = ExprSum::new_pair(&ExprSine::new(&xe).cast(), &xe).cast();
    line.step(|line| Ok(line.text(&s.n_derivative(&x, 0)?.string()?)));
    line.step(|line| Ok(line.text(&value(2.0).n_derivative(&x, -1)?.string()?)));
    line.step(|line| Ok(line.text(&ExprSine::new(&xe).n_derivative(&x, 0)?.string()?)));
    line.step(|line| Ok(line.text(&ExprUnaryMinus::new(&xe).n_derivative(&x, 0)?.string()?)));
    line.step(|line| {
        Ok(line.text(
            &ExprDifference::new(&xe, &value(1.0))
                .n_derivative(&x, 0)?
                .string()?,
        ))
    });
    let difference = ExprDifference::new(&ExprSine::new(&xe).cast(), &xe);
    line.step(|line| Ok(line.text(&difference.n_derivative(&x, 3)?.string()?)));
    let opposite = ExprUnaryMinus::new(&ExprCosine::new(&xe).cast());
    line.step(|line| Ok(line.text(&opposite.n_derivative(&x, 2)?.string()?)));
    line.step(|line| Ok(line.text(&ExprAbsolute::new(&value(3.0)).derivative(&x)?.string()?)));
    line.step(|line| Ok(line.text(&ExprSign::new(&xe).derivative(&x)?.string()?)));
}

/// 4: replacement inside expressions and relations.
fn replacements(line: &mut ProbeLine) {
    let x = unknown("x");
    let y = unknown("y");
    let (xe, ye): (ExprHandle, ExprHandle) = (x.cast(), y.cast());
    let e: ExprHandle =
        expr_multiply(&ExprSine::new(&xe).cast(), &expr_add(&xe, &ye).cast()).cast();
    line.step(|line| {
        e.replace(&x, &expr_add(&ye, &value(1.0)).cast())?;
        Ok(line.text(&e.string()?))
    });
    line.step(|_| {
        let s = ExprSine::new(&ye);
        s.replace(&y, &ExprUnaryMinus::new(&s.cast()).cast())
    });
    line.step(|line| {
        let q: ExprHandle =
            ExprDivision::new(&ye, &ExprExponentiate::new(&ye, &value(2.0)).cast()).cast();
        q.replace(&y, &value(3.0))?;
        line.text(&q.string()?);
        line.text(&q.shallow_simplified()?.string()?);
        Ok(line.text(&q.simplified()?.string()?))
    });
    line.step(|line| {
        let z = unknown("z");
        z.assign(&expr_add(&xe, &value(1.0)).cast())?;
        z.replace(&x, &ye)?;
        Ok(line.text(&z.assigned_expression()?.string()?))
    });
}

/// 5: the interpreter outside a successful parse.
fn interpreter(line: &mut ProbeLine) {
    let g = rustycad_tkexpress::expr_intrp::ExprIntrpGenExp::create();
    line.step(|line| {
        line.boolean(g.is_done());
        Ok(line.text(&g.expression()?.string()?))
    });
    line.step(|line| {
        g.process(&AsciiString::new(""));
        Ok(line.boolean(g.is_done()))
    });
    line.step(|line| {
        g.process(&AsciiString::new("x+y"));
        line.boolean(g.is_done());
        line.text(&g.expression()?.string()?);
        line.boolean(g.get_named(&AsciiString::new("x")).is_null());
        line.boolean(g.get_named(&AsciiString::new("q")).is_null());
        Ok(line.boolean(g.get_function(&AsciiString::new("x")).is_null()))
    });
    line.step(|line| {
        g.process(&AsciiString::new("x+"));
        line.boolean(g.is_done());
        Ok(line.text(&g.expression()?.string()?))
    });
}

/// 6: the iterators past their end.
fn iterators(line: &mut ProbeLine) {
    let x: ExprHandle = unknown("x").cast();
    let e: ExprHandle = ExprSum::new_pair(&x, &ExprSine::new(&x).cast()).cast();
    line.step(|line| {
        let mut it = ExprUnknownIterator::new(&e)?;
        line.boolean(it.more());
        it.next()?;
        line.boolean(it.more());
        it.next()
    });
    line.step(|line| {
        let mut it = ExprUnknownIterator::new(&e)?;
        it.next()?;
        Ok(line.text(&it.value()?.get_name()))
    });
    let q: ExprRelationHandle = ExprEqual::new(&x, &value(1.0)).cast();
    line.step(|line| {
        let mut it = ExprRelationIterator::new(&q)?;
        it.next()?;
        Ok(line.text(&it.value()?.string()?))
    });
    line.step(|_| {
        let mut it = ExprRelationIterator::new(&q)?;
        it.next()?;
        it.next()
    });
    line.step(|line| {
        let mut it = ExprRUIterator::new(&q)?;
        it.next()?;
        Ok(line.text(&it.value()?.get_name()))
    });
    line.step(|line| Ok(line.text(&q.sub_relation(1)?.string()?)));
}

/// 7: the operators with reals keep the native operand order.
fn operators(line: &mut ProbeLine) {
    let e: ExprHandle = unknown("x").cast();
    line.step(|line| {
        line.text(&add_real(&e, 2.0).string()?);
        line.text(&real_add(2.0, &e).string()?);
        line.text(&subtract_real(&e, 2.0).string()?);
        Ok(line.text(&real_subtract(2.0, &e).string()?))
    });
    line.step(|line| {
        line.text(&multiply_real(&e, 2.0).string()?);
        line.text(&real_multiply(2.0, &e).string()?);
        line.text(&divide_real(&e, 2.0).string()?);
        Ok(line.text(&real_divide(2.0, &e).string()?))
    });
    line.step(|line| {
        line.text(&expr_negate(&e).string()?);
        line.text(&expr_add(&e, &e).string()?);
        line.text(&expr_subtract(&e, &e).string()?);
        line.text(&expr_multiply(&e, &e).string()?);
        Ok(line.text(&expr_divide(&e, &e).string()?))
    });
}

/// 8: IsIdentical of sums and products, operand order and repeated operands.
fn identical(line: &mut ProbeLine) {
    let (x, y): (ExprHandle, ExprHandle) = (unknown("x").cast(), unknown("y").cast());
    let a = [x.clone(), x.clone(), y.clone()];
    let b = [x.clone(), y.clone(), x.clone()];
    let c = [x.clone(), y.clone(), y.clone()];
    line.step(|line| {
        let (p, q, r): (ExprHandle, ExprHandle, ExprHandle) = (
            ExprProduct::new(&a).cast(),
            ExprProduct::new(&b).cast(),
            ExprProduct::new(&c).cast(),
        );
        line.boolean(p.is_identical(&q)?);
        line.boolean(q.is_identical(&p)?);
        line.boolean(p.is_identical(&r)?);
        Ok(line.boolean(r.is_identical(&p)?))
    });
    line.step(|line| {
        let (p, q, r): (ExprHandle, ExprHandle, ExprHandle) = (
            ExprSum::new(&a).cast(),
            ExprSum::new(&b).cast(),
            ExprSum::new(&c).cast(),
        );
        line.boolean(p.is_identical(&q)?);
        line.boolean(p.is_identical(&r)?);
        Ok(line.boolean(p.is_identical(&ExprProduct::new(&a).cast())?))
    });
    line.step(|line| {
        let p = ExprSum::new(&a);
        p.set_operand(&value(1.0), 1)?;
        line.text(&p.string()?);
        Ok(line.text(&p.shallow_simplified()?.string()?))
    });
    line.step(|line| {
        let d: ExprHandle = ExprDifference::new(&x, &y).cast();
        line.boolean(d.is_identical(&ExprDifference::new(&x, &y).cast())?);
        line.boolean(d.is_identical(&ExprDifference::new(&y, &x).cast())?);
        let power: ExprHandle = ExprExponentiate::new(&x, &y).cast();
        line.boolean(power.is_identical(&ExprExponentiate::new(&x, &y).cast())?);
        let quotient: ExprHandle = ExprDivision::new(&x, &y).cast();
        Ok(line.boolean(quotient.is_identical(&ExprDivision::new(&y, &x).cast())?))
    });
}

/// `ex_case <n>`.
pub fn case_case(fields: &[&str]) {
    let mut line: ProbeLine = ProbeLine::default();
    let scenarios: [fn(&mut ProbeLine); 9] = [
        operands,
        unknowns,
        derivatives,
        functions,
        replacements,
        interpreter,
        iterators,
        operators,
        identical,
    ];
    let n: usize = fields
        .first()
        .and_then(|n| n.parse().ok())
        .unwrap_or(usize::MAX);
    if let Some(scenario) = scenarios.get(n) {
        scenario(&mut line);
    }
    line.end();
}
