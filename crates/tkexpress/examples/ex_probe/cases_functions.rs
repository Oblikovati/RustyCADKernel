//! Scenario 3 of `ex_case` (`ex_case_functions` of tools/oracle/tkexpress_probe_cases.cxx):
//! named functions, their derivatives and the function expressions.
use super::cases::{unknown, value};
use super::common::ProbeLine;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkexpress::expr::handles::one_based;
use rustycad_tkexpress::expr::{
    ExprBinaryFunction, ExprCosine, ExprFunctionDerivative, ExprFunctionHandle, ExprHandle,
    ExprNamedFunction, ExprNamedUnknown, ExprPolyFunction, ExprProduct, ExprSine, ExprSquare,
    ExprSum, ExprUnaryFunction,
};
use rustycad_tkexpress::expr::{ExprGeneralFunction, ExprNamedExpression};

/// The functions of the scenario: `f[x] = x*Sin(x)` and `g[x, y] = x*y + y^2`.
struct Functions {
    x: Handle<ExprNamedUnknown>,
    y: Handle<ExprNamedUnknown>,
    f: Handle<ExprNamedFunction>,
    g: Handle<ExprNamedFunction>,
}
impl Functions {
    fn new() -> Self {
        let x = unknown("x");
        let y = unknown("y");
        let (xe, ye): (ExprHandle, ExprHandle) = (x.cast(), y.cast());
        let f_body: ExprHandle = ExprProduct::new_pair(&xe, &ExprSine::new(&xe).cast()).cast();
        let f =
            ExprNamedFunction::new(&AsciiString::new("f"), &f_body, &one_based(vec![x.clone()]));
        let g_body: ExprHandle = ExprSum::new_pair(
            &ExprProduct::new_pair(&xe, &ye).cast(),
            &ExprSquare::new(&ye).cast(),
        )
        .cast();
        let g = ExprNamedFunction::new(
            &AsciiString::new("g"),
            &g_body,
            &one_based(vec![x.clone(), y.clone()]),
        );
        Self { x, y, f, g }
    }
}

/// The construction and evaluation checks.
fn checks(line: &mut ProbeLine, s: &Functions) {
    let (f, g): (ExprFunctionHandle, ExprFunctionHandle) = (s.f.cast(), s.g.cast());
    line.step(|_| ExprFunctionDerivative::new(&f, &s.x, 0).map(|_| ()));
    line.step(|_| ExprUnaryFunction::new(&g, &s.x.cast()).map(|_| ()));
    line.step(|_| ExprBinaryFunction::new(&f, &s.x.cast(), &s.y.cast()).map(|_| ()));
    line.step(|line| {
        let vars = one_based(vec![s.x.clone(), s.y.clone()]);
        Ok(line.real(f.evaluate(&vars, &one_based(vec![1.0]))?))
    });
}

/// The derivative names, expressions and identities.
fn derivatives(line: &mut ProbeLine, s: &Functions) -> Result<(), Failure> {
    let d: ExprFunctionHandle = s.f.derivative_of_degree(&s.x, 2)?;
    line.text(&d.get_string_name()?);
    line.int(d.nb_of_variables());
    let derivative: Handle<ExprFunctionDerivative> = d.cast();
    line.text(&derivative.expression().string()?);
    line.boolean(d.is_linear_on_variable(1)?);
    let dd: ExprFunctionHandle = d.derivative(&s.x)?;
    line.text(&dd.get_string_name()?);
    line.boolean(d.is_identical(&s.f.derivative_of_degree(&s.x, 2)?)?);
    line.boolean(d.is_identical(&dd)?);
    line.text(&d.copy()?.get_string_name()?);
    Ok(())
}

/// The derivatives of the function of two variables.
fn mixed_derivatives(line: &mut ProbeLine, s: &Functions) -> Result<(), Failure> {
    let d: ExprFunctionHandle = s.g.derivative_of_degree(&s.y, 2)?;
    line.text(&d.get_string_name()?);
    let dx: ExprFunctionHandle = d.derivative(&s.x)?;
    line.text(&dx.get_string_name()?);
    let derivative: Handle<ExprFunctionDerivative> = dx.cast();
    line.text(&derivative.expression().string()?);
    let vars = one_based(vec![s.x.clone(), s.y.clone()]);
    line.real(dx.evaluate(&vars, &one_based(vec![0.75, -1.5]))?);
    Ok(())
}

/// The function expressions of one, two and several operands.
fn applications(line: &mut ProbeLine, s: &Functions) {
    let (f, g): (ExprFunctionHandle, ExprFunctionHandle) = (s.f.cast(), s.g.cast());
    let ye: ExprHandle = s.y.cast();
    line.step(|line| {
        let u: ExprHandle = ExprUnaryFunction::new(&f, &ExprSquare::new(&ye).cast())?.cast();
        line.text(&u.string()?);
        line.text(&u.derivative(&s.y)?.string()?);
        line.boolean(u.is_linear()?);
        line.text(&u.copy()?.string()?);
        line.boolean(u.is_identical(&u.copy()?)?);
        let folded: ExprHandle = ExprUnaryFunction::new(&f, &value(0.25))?.cast();
        Ok(line.text(&folded.shallow_simplified()?.string()?))
    });
    line.step(|line| {
        let b: ExprHandle =
            ExprBinaryFunction::new(&g, &ExprSine::new(&ye).cast(), &value(2.0))?.cast();
        line.text(&b.string()?);
        line.text(&b.derivative(&s.y)?.string()?);
        line.boolean(b.is_linear()?);
        let folded: ExprHandle = ExprBinaryFunction::new(&g, &value(1.5), &value(2.0))?.cast();
        Ok(line.text(&folded.shallow_simplified()?.string()?))
    });
    line.step(|line| {
        let p: ExprHandle =
            ExprPolyFunction::new(&g, &[ye.clone(), ExprCosine::new(&ye).cast()]).cast();
        line.text(&p.string()?);
        line.text(&p.derivative(&s.y)?.string()?);
        line.boolean(p.is_linear()?);
        line.boolean(p.is_identical(&p.copy()?)?);
        let folded: ExprHandle = ExprPolyFunction::new(&g, &[value(0.5), value(2.0)]).cast();
        Ok(line.text(&folded.shallow_simplified()?.string()?))
    });
}

/// 3: named functions, their derivatives and the function expressions.
pub fn functions(line: &mut ProbeLine) {
    let s: Functions = Functions::new();
    checks(line, &s);
    line.step(|line| derivatives(line, &s));
    line.step(|line| mixed_derivatives(line, &s));
    applications(line, &s);
    let (f, g): (ExprFunctionHandle, ExprFunctionHandle) = (s.f.cast(), s.g.cast());
    line.step(|line| {
        s.f.set_name(&AsciiString::new("h"));
        line.text(&f.get_string_name()?);
        s.f.set_expression(&ExprCosine::new(&s.x.cast()).cast());
        line.text(&s.f.expression().string()?);
        line.boolean(f.is_identical(&f.copy()?)?);
        line.boolean(f.is_identical(&g)?);
        Ok(line.text(&f.variable(1)?.get_name()))
    });
    line.step(|line| Ok(line.text(&f.variable(2)?.get_name())));
}
