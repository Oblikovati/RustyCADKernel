// Rust translation of ModelingAlgorithms/TKShHealing/ShapeProcess/ShapeProcess.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The package ShapeProcess: the process-wide registry of named operators and the two
//! ways of running them on a context (a resource sequence or a set of operation flags).
use super::shape_context::statistics::{message, send};
use super::{ShapeProcessContext, ShapeProcessOperator};
use rustycad_tkernel::message::{Gravity, ProgressRange, ProgressScope};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

/// `ShapeProcess::Operation`: the predefined operations, in the native order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ShapeProcessOperation {
    /// DirectFaces (also First).
    DirectFaces,
    /// SameParameter.
    SameParameter,
    /// SetTolerance.
    SetTolerance,
    /// SplitAngle.
    SplitAngle,
    /// BSplineRestriction.
    BSplineRestriction,
    /// ElementaryToRevolution.
    ElementaryToRevolution,
    /// SweptToElementary.
    SweptToElementary,
    /// SurfaceToBSpline.
    SurfaceToBSpline,
    /// ToBezier.
    ToBezier,
    /// SplitContinuity.
    SplitContinuity,
    /// SplitClosedFaces.
    SplitClosedFaces,
    /// FixWireGaps.
    FixWireGaps,
    /// FixFaceSize.
    FixFaceSize,
    /// DropSmallSolids.
    DropSmallSolids,
    /// DropSmallEdges.
    DropSmallEdges,
    /// FixShape.
    FixShape,
    /// SplitClosedEdges.
    SplitClosedEdges,
    /// SplitCommonVertex (also Last).
    SplitCommonVertex,
}

impl ShapeProcessOperation {
    /// `Operation::First`.
    pub const FIRST: Self = Self::DirectFaces;
    /// `Operation::Last`.
    pub const LAST: Self = Self::SplitCommonVertex;
    /// Every operation in the native order.
    pub const ALL: [Self; 18] = [
        Self::DirectFaces,
        Self::SameParameter,
        Self::SetTolerance,
        Self::SplitAngle,
        Self::BSplineRestriction,
        Self::ElementaryToRevolution,
        Self::SweptToElementary,
        Self::SurfaceToBSpline,
        Self::ToBezier,
        Self::SplitContinuity,
        Self::SplitClosedFaces,
        Self::FixWireGaps,
        Self::FixFaceSize,
        Self::DropSmallSolids,
        Self::DropSmallEdges,
        Self::FixShape,
        Self::SplitClosedEdges,
        Self::SplitCommonVertex,
    ];

    /// The private `toOperationName`: the registered name of the operation.
    pub fn name(self) -> &'static str {
        const NAMES: [&str; 18] = [
            "DirectFaces",
            "SameParameter",
            "SetTolerance",
            "SplitAngle",
            "BSplineRestriction",
            "ElementaryToRevolution",
            "SweptToElementary",
            "SurfaceToBSpline",
            "ToBezier",
            "SplitContinuity",
            "SplitClosedFaces",
            "FixWireGaps",
            "FixFaceSize",
            "DropSmallSolids",
            "DropSmallEdges",
            "FixShape",
            "SplitClosedEdges",
            "SplitCommonVertex",
        ];
        NAMES[self as usize]
    }
}

/// `ShapeProcess::OperationsFlags` (`std::bitset<Operation::Last + 1>`): one bit per
/// operation, bit `op as usize` for `op`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShapeProcessOperationsFlags(u32);

impl ShapeProcessOperationsFlags {
    /// `set(op)`; `flags.set(ShapeProcessOperation::FixShape)`.
    pub fn set(&mut self, operation: ShapeProcessOperation) {
        self.0 |= 1 << operation as u32;
    }

    /// `test(op)`; `flags.test(ShapeProcessOperation::FixShape)`.
    pub fn test(&self, operation: ShapeProcessOperation) -> bool {
        self.0 & (1 << operation as u32) != 0
    }
}

/// The registered operators by name.
type OperatorMap = HashMap<Vec<u8>, Handle<dyn ShapeProcessOperator>>;

/// The native static `aMapOfOperators`.
static OPERATORS: Mutex<Option<OperatorMap>> = Mutex::new(None);

fn operators() -> MutexGuard<'static, Option<OperatorMap>> {
    OPERATORS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The native `ScopeLock`: the scope is pushed on creation and popped on drop.
struct ScopeLock<'a> {
    context: &'a ShapeProcessContext,
}

impl<'a> ScopeLock<'a> {
    fn new(context: &'a ShapeProcessContext, scope: &str) -> Self {
        context.set_scope(scope);
        Self { context }
    }
}

impl Drop for ScopeLock<'_> {
    fn drop(&mut self) {
        self.context.un_set_scope();
    }
}

/// The package `ShapeProcess` (static functions only).
///
/// ```
/// use rustycad_tkshhealing::shape_process::{ShapeProcess, ShapeProcessOperation};
/// assert_eq!(ShapeProcess::to_operation_flag(Some("FixShape")), (ShapeProcessOperation::FixShape, true));
/// assert!(!ShapeProcess::to_operation_flag(Some("Unknown")).1);
/// ```
pub struct ShapeProcess;

impl ShapeProcess {
    /// `RegisterOperator(name, op)`: false (and nothing changed) when the name is taken;
    /// `ShapeProcess::register_operator("MyOp", &op)`.
    pub fn register_operator(name: &str, operator: &Handle<dyn ShapeProcessOperator>) -> bool {
        let mut map = operators();
        let map = map.get_or_insert_with(HashMap::new);
        if map.contains_key(name.as_bytes()) {
            return false;
        }
        map.insert(name.as_bytes().to_vec(), operator.clone());
        true
    }

    /// `FindOperator(name, op)`: the registered operator; `None` for an unknown name or a
    /// null operator; `ShapeProcess::find_operator("FixShape")`.
    pub fn find_operator(name: &str) -> Option<Handle<dyn ShapeProcessOperator>> {
        let map = operators();
        let operator = map.as_ref()?.get(name.as_bytes())?.clone();
        (!operator.is_null()).then_some(operator)
    }

    /// `ToOperationFlag(theName)`: the operation of a name, `(First, false)` for an unknown
    /// or missing name.
    pub fn to_operation_flag(name: Option<&str>) -> (ShapeProcessOperation, bool) {
        let Some(name) = name else {
            return (ShapeProcessOperation::FIRST, false);
        };
        ShapeProcessOperation::ALL
            .into_iter()
            .find(|operation| operation.name() == name)
            .map_or((ShapeProcessOperation::FIRST, false), |operation| {
                (operation, true)
            })
    }

    /// `Perform(context, seq, theProgress)`: runs the operators listed in the `exec.op`
    /// parameter of scope `seq` (separated by blanks, tabs, commas or semicolons), each
    /// under its own scope; true when one of them reported a change. A missing sequence
    /// or operator and an operator exception are reported to the messenger.
    /// Example: `ShapeProcess::perform_sequence(&context, "read", &ProgressRange::default())?`.
    pub fn perform_sequence(
        context: &Handle<ShapeProcessContext>,
        sequence: &str,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let ctx = context.get().ok_or_else(null_context)?;
        let _sequence_scope = ScopeLock::new(ctx, sequence);
        let Some(text) = ctx.get_string("exec.op") else {
            if ctx.trace_level() > 0 {
                let mut warning = message("SP.Sequence.Warn.NoSeq");
                warning.arg_text(sequence.as_bytes())?;
                send(&ctx.messenger(), &mut warning, Gravity::Warning)?;
            }
            return Ok(false);
        };
        let names = sequence_operators(&text);
        if ctx.trace_level() >= 2 {
            let mut info = message("SP.Sequence.Info.Seq");
            info.arg_text(&names.join(&b","[..]))?;
            send(&ctx.messenger(), &mut info, Gravity::Info)?;
        }
        run_sequence(context, ctx, &names, progress)
    }

    /// `Perform(theContext, theOperations, theProgress)`: runs the registered operators of
    /// the set flags in the native order, each under its own scope; false without a
    /// context or operator. Example:
    /// `ShapeProcess::perform_operations(&context, flags, &ProgressRange::default())?`.
    pub fn perform_operations(
        context: &Handle<ShapeProcessContext>,
        operations: ShapeProcessOperationsFlags,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(ctx) = context.get() else {
            return Ok(false);
        };
        let selected = Self::selected_operators(operations);
        if selected.is_empty() {
            return Ok(false);
        }
        let scope = ProgressScope::unnamed(progress, selected.len() as f64, false);
        let mut any = false;
        for (name, operator) in selected {
            let range = scope.next(1.0);
            let _operation_scope = ScopeLock::new(ctx, name);
            match operator.perform(context, &range) {
                Ok(done) => any |= done,
                Err(failure) => report_exception(ctx, name.as_bytes(), &failure)?,
            }
        }
        Ok(any)
    }

    /// The private `getOperators(theFlags)`: the registered operators of the set flags.
    fn selected_operators(
        flags: ShapeProcessOperationsFlags,
    ) -> Vec<(&'static str, Handle<dyn ShapeProcessOperator>)> {
        ShapeProcessOperation::ALL
            .into_iter()
            .filter(|operation| flags.test(*operation))
            .filter_map(|operation| {
                Some((operation.name(), Self::find_operator(operation.name())?))
            })
            .collect()
    }
}

fn null_context() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeProcess::Perform: null ShapeProcess_Context",
        "",
    )
}

/// The tokens of the `exec.op` text, as `Token(" \t,;", i)` until an empty token.
fn sequence_operators(text: &AsciiString) -> Vec<Vec<u8>> {
    let mut names = Vec::new();
    for i in 1.. {
        let name = text.token(" \t,;", i);
        if name.is_empty() {
            break;
        }
        names.push(name.as_bytes().to_vec());
    }
    names
}

fn run_sequence(
    context: &Handle<ShapeProcessContext>,
    ctx: &ShapeProcessContext,
    names: &[Vec<u8>],
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let scope = ProgressScope::unnamed(progress, names.len() as f64, false);
    let mut done = false;
    for (index, name) in names.iter().enumerate() {
        if !scope.more() {
            break;
        }
        let range = scope.next(1.0);
        if ctx.trace_level() >= 2 {
            let mut info = message("SP.Sequence.Info.Operator");
            info.arg_integer(index as i32 + 1)?
                .arg_integer(names.len() as i32)?
                .arg_text(name)?;
            send(&ctx.messenger(), &mut info, Gravity::Alarm)?;
        }
        let text = String::from_utf8_lossy(name).into_owned();
        let Some(operator) = ShapeProcess::find_operator(&text) else {
            if ctx.trace_level() > 0 {
                let mut error = message("SP.Sequence.Error.NoOp");
                error.arg_text(name)?;
                send(&ctx.messenger(), &mut error, Gravity::Alarm)?;
            }
            continue;
        };
        let _operation_scope = ScopeLock::new(ctx, &text);
        match operator.perform(context, &range) {
            Ok(changed) => done |= changed,
            Err(failure) => report_exception(ctx, name, &failure)?,
        }
    }
    Ok(done)
}

/// "Operator %s failed with exception %s".
fn report_exception(
    ctx: &ShapeProcessContext,
    name: &[u8],
    failure: &Failure,
) -> Result<(), Failure> {
    let mut error = message("SP.Sequence.Error.Except");
    error.arg_text(name)?.arg_text(failure.what())?;
    send(&ctx.messenger(), &mut error, Gravity::Alarm)
}
