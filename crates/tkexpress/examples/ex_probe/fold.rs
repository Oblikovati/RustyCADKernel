//! `ex_fold` and `ex_fold2`: the values the Expr classes compute from numeric operands.
use super::common::{ProbeLine, bits};
use rustycad_tkernel::standard::Handle;
use rustycad_tkexpress::expr::{
    ExprAbsolute, ExprArcCosine, ExprArcSine, ExprArcTangent, ExprArgCosh, ExprArgSinh,
    ExprArgTanh, ExprCosh, ExprCosine, ExprDifference, ExprDivision, ExprExponential,
    ExprExponentiate, ExprHandle, ExprLogOf10, ExprLogOfe, ExprNumericValue, ExprProduct, ExprSign,
    ExprSine, ExprSinh, ExprSquare, ExprSquareRoot, ExprSum, ExprTangent, ExprTanh, ExprUnaryMinus,
};

/// The classes of `ex_fold`, by code (`ex_unary`).
fn unary(code: i32, op: &ExprHandle) -> ExprHandle {
    match code {
        0 => ExprAbsolute::new(op).cast(),
        1 => ExprArcCosine::new(op).cast(),
        2 => ExprArcSine::new(op).cast(),
        3 => ExprArcTangent::new(op).cast(),
        4 => ExprArgCosh::new(op).cast(),
        5 => ExprArgSinh::new(op).cast(),
        6 => ExprArgTanh::new(op).cast(),
        7 => ExprCosh::new(op).cast(),
        8 => ExprCosine::new(op).cast(),
        9 => ExprExponential::new(op).cast(),
        10 => ExprLogOf10::new(op).cast(),
        11 => ExprLogOfe::new(op).cast(),
        12 => ExprSign::new(op).cast(),
        13 => ExprSine::new(op).cast(),
        14 => ExprSinh::new(op).cast(),
        15 => ExprSquare::new(op).cast(),
        16 => ExprSquareRoot::new(op).cast(),
        17 => ExprTangent::new(op).cast(),
        18 => ExprTanh::new(op).cast(),
        _ => ExprUnaryMinus::new(op).cast(),
    }
}

/// The classes of `ex_fold2`, by code (`ex_binary`).
fn binary(code: i32, a: &ExprHandle, b: &ExprHandle) -> ExprHandle {
    match code {
        0 => ExprSum::new_pair(a, b).cast(),
        1 => ExprDifference::new(a, b).cast(),
        2 => ExprProduct::new_pair(a, b).cast(),
        3 => ExprDivision::new(a, b).cast(),
        _ => ExprExponentiate::new(a, b).cast(),
    }
}

/// The simplified value (or its text), the evaluated value and the text (`ex_folded`).
fn folded(line: &mut ProbeLine, e: &ExprHandle) {
    line.step(|line| {
        let simplified: ExprHandle = e.shallow_simplified()?;
        let value: Handle<ExprNumericValue> = simplified.cast();
        match value.get() {
            Some(value) => line.real(value.get_value()),
            None => line.text(&simplified.string()?),
        }
        Ok(())
    });
    line.step(|line| Ok(line.real(e.evaluate_numeric()?)));
    line.step(|line| Ok(line.text(&e.string()?)));
}

/// `ex_fold <code> <bits>`.
pub fn fold_case(fields: &[&str]) {
    let mut line: ProbeLine = ProbeLine::default();
    let code: i32 = fields
        .first()
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let value: ExprHandle = ExprNumericValue::expression(bits(fields.get(1).unwrap_or(&"0")));
    folded(&mut line, &unary(code, &value));
    line.end();
}

/// `ex_fold2 <code> <bits> <bits>`.
pub fn fold2_case(fields: &[&str]) {
    let mut line: ProbeLine = ProbeLine::default();
    let code: i32 = fields
        .first()
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let a: ExprHandle = ExprNumericValue::expression(bits(fields.get(1).unwrap_or(&"0")));
    let b: ExprHandle = ExprNumericValue::expression(bits(fields.get(2).unwrap_or(&"0")));
    folded(&mut line, &binary(code, &a, &b));
    line.end();
}
