// Rust translation of the function actions of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp_yaccintrf.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The semantic actions of function calls and function definitions.
use super::context::ExprIntrpParseContext;
use super::errors::syntax_error;
use crate::expr::errors::out_of_range;
use crate::expr::handles::deref;
use crate::expr::package::copy_share;
use crate::expr::{
    ExprAbsolute, ExprArcCosine, ExprArcSine, ExprArcTangent, ExprArgCosh, ExprArgSinh,
    ExprArgTanh, ExprBinaryFunction, ExprCosh, ExprCosine, ExprExponential, ExprFunctionHandle,
    ExprGeneralExpression, ExprHandle, ExprLogOf10, ExprLogOfe, ExprNamedFunction,
    ExprNamedUnknown, ExprPolyFunction, ExprSign, ExprSine, ExprSinh, ExprSquareRoot, ExprTangent,
    ExprTanh, ExprUnaryFunction,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// A constructor of a standard function class.
type StandardConstructor = fn(&ExprHandle) -> ExprHandle;

/// The names of the standard functions, in the order `ExprIntrp_StandardFunction`
/// tests them, with their classes.
const STANDARD_FUNCTIONS: [(&[&str], StandardConstructor); 18] = [
    (&["abs", "Abs"], |op| ExprAbsolute::new(op).cast()),
    (&["acos", "ACos"], |op| ExprArcCosine::new(op).cast()),
    (&["asin", "ASin"], |op| ExprArcSine::new(op).cast()),
    (&["atan", "ATan"], |op| ExprArcTangent::new(op).cast()),
    (&["acosh", "ACosh"], |op| ExprArgCosh::new(op).cast()),
    (&["asinh", "ASinh"], |op| ExprArgSinh::new(op).cast()),
    (&["atanh", "ATanh"], |op| ExprArgTanh::new(op).cast()),
    (&["cosh", "Cosh"], |op| ExprCosh::new(op).cast()),
    (&["cos", "Cos"], |op| ExprCosine::new(op).cast()),
    (&["exp", "Exp"], |op| ExprExponential::new(op).cast()),
    (&["log"], |op| ExprLogOf10::new(op).cast()),
    (&["Log", "Ln"], |op| ExprLogOfe::new(op).cast()),
    (&["sign", "Sign"], |op| ExprSign::new(op).cast()),
    (&["sin", "Sin"], |op| ExprSine::new(op).cast()),
    (&["sinh", "Sinh"], |op| ExprSinh::new(op).cast()),
    (&["sqrt", "Sqrt"], |op| ExprSquareRoot::new(op).cast()),
    (&["tan", "Tan"], |op| ExprTangent::new(op).cast()),
    (&["tanh", "Tanh"], |op| ExprTanh::new(op).cast()),
];

/// `ExprIntrp_StandardFunction`: the standard function `name` of `op`, null when
/// `name` names none; `standard_function(&AsciiString::new("Sin"), &x)`.
pub fn standard_function(name: &AsciiString, op: &ExprHandle) -> ExprHandle {
    STANDARD_FUNCTIONS
        .iter()
        .find(|(names, _)| names.iter().any(|candidate| name == candidate))
        .map(|(_, constructor)| constructor(op))
        .unwrap_or_default()
}

impl ExprIntrpParseContext<'_> {
    /// `ExprIntrp_StartFunction`: remember the function name and count its arguments
    /// from 0.
    pub(super) fn start_function(&mut self) -> Result<(), Failure> {
        let name = self.result();
        self.recept.push_name(&name);
        self.nbargs = 0;
        Ok(())
    }
    /// `ExprIntrp_NextFuncArg` and `ExprIntrp_EndFuncArg`: count one argument.
    pub(super) fn count_argument(&mut self) -> Result<(), Failure> {
        self.nbargs += 1;
        Ok(())
    }
    /// The named function `name` of the parse; ExprIntrp_SyntaxError when unknown.
    pub(super) fn known_function(
        &self,
        name: &AsciiString,
    ) -> Result<Handle<ExprNamedFunction>, Failure> {
        let function: Handle<ExprNamedFunction> = self.recept.get_function(name);
        if function.is_null() {
            return Err(syntax_error(&format!(
                "{} is neither a standard nor a defined function",
                String::from_utf8_lossy(name.as_bytes())
            )));
        }
        Ok(function)
    }
    /// `ExprIntrp_EndFunction`: a standard function of the operand, simplified one
    /// level, or a defined function applied to as many stacked arguments as it has
    /// variables (the parsed argument count is not checked natively).
    pub(super) fn end_function(&mut self) -> Result<(), Failure> {
        let name: AsciiString = self.recept.pop_name();
        let operand: ExprHandle = self.recept.pop();
        let standard: ExprHandle = standard_function(&name, &operand);
        if let Some(standard) = standard.get() {
            let simplified: ExprHandle = standard.shallow_simplified()?;
            self.recept.push(&simplified);
            return Ok(());
        }
        let function: ExprFunctionHandle = self.known_function(&name)?.cast();
        let application: ExprHandle = self.apply_function(&function, operand)?;
        self.recept.push(&application);
        Ok(())
    }
    /// `function` applied to `last` and the arguments stacked before it.
    fn apply_function(
        &mut self,
        function: &ExprFunctionHandle,
        last: ExprHandle,
    ) -> Result<ExprHandle, Failure> {
        let count: i32 = deref(function, "ExprIntrp_EndFunction")?.nb_of_variables();
        match count {
            1 => Ok(ExprUnaryFunction::new(function, &last)?.cast()),
            2 => {
                let first: ExprHandle = self.pop_required("ExprIntrp_EndFunction")?;
                Ok(ExprBinaryFunction::new(function, &first, &last)?.cast())
            }
            _ if count < 1 => Err(out_of_range(&format!(
                "ExprIntrp_EndFunction: the function has {count} variables; tabarg({count}) is out of [1, {count}]"
            ))),
            _ => {
                let mut arguments: Vec<ExprHandle> = vec![last];
                for _ in 1..count {
                    arguments.push(self.pop_required("ExprIntrp_EndFunction")?);
                }
                arguments.reverse();
                Ok(ExprPolyFunction::new(function, &arguments).cast())
            }
        }
    }
    /// `ExprIntrp_DefineFunction`: remember the name of the defined function and stack
    /// its number of variables.
    pub(super) fn define_function(&mut self) -> Result<(), Failure> {
        self.funcdefname = self.recept.pop_name();
        let count: i32 = self.nbargs;
        self.recept.push_value(count);
        Ok(())
    }
    /// `ExprIntrp_EndOfFuncDef`: the function `name[x1, ...] <- body`, whose variables
    /// are copies of the parsed unknowns, made known to the parse and the generator.
    pub(super) fn end_of_func_def(&mut self) -> Result<(), Failure> {
        let body: ExprHandle = self.recept.pop();
        let count: i32 = self.recept.pop_value();
        let mut variables: Vec<Handle<ExprNamedUnknown>> = Vec::new();
        let mut internal: Vec<Handle<ExprNamedUnknown>> = Vec::new();
        for _ in 0..count.max(0) {
            let variable: Handle<ExprNamedUnknown> = self.recept.pop().cast();
            let copy: Handle<ExprNamedUnknown> =
                deref(&variable, "ExprIntrp_EndOfFuncDef")?.copy()?.cast();
            variables.push(variable);
            internal.push(copy);
        }
        variables.reverse();
        internal.reverse();
        let body: ExprHandle = replaced_variables(copy_share(&body)?, &variables, &internal)?;
        let name: AsciiString = self.funcdefname.clone();
        let function =
            ExprNamedFunction::new(&name, &body, &crate::expr::handles::one_based(internal));
        self.recept.use_function(&function);
        Ok(())
    }
}

/// `body` with each of `variables` replaced by the matching internal copy: in place
/// when it occurs inside, or the copy itself when the body is that variable.
fn replaced_variables(
    mut body: ExprHandle,
    variables: &[Handle<ExprNamedUnknown>],
    internal: &[Handle<ExprNamedUnknown>],
) -> Result<ExprHandle, Failure> {
    for (variable, copy) in variables.iter().zip(internal) {
        let expression = deref(&body, "ExprIntrp_EndOfFuncDef")?;
        if expression.contains(&variable.cast())? {
            expression.replace(variable, &copy.cast())?;
        } else if body == *variable {
            body = copy.cast();
        }
    }
    Ok(body)
}
