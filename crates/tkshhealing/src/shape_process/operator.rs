// Rust translation of ModelingAlgorithms/TKShHealing/ShapeProcess/ShapeProcess_Operator,
// ShapeProcess_UOperator and ShapeProcess_OperFunc at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The operators of ShapeProcess: the abstract `ShapeProcess_Operator` and the operator
//! `ShapeProcess_UOperator` that calls a function.
use super::ShapeProcessContext;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};

/// `ShapeProcess_OperFunc`: the function of a user operator; true when it changed the
/// shape of the context. A native exception is the `Err` value.
pub type ShapeProcessOperFunc =
    fn(&Handle<ShapeProcessContext>, &ProgressRange<'_>) -> Result<bool, Failure>;

/// `ShapeProcess_Operator`: one named step of a ShapeProcess sequence.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkshhealing::shape_process::{ShapeProcessContext, ShapeProcessOperator, ShapeProcessUOperator};
/// let op = ShapeProcessUOperator::new(|_, _| Ok(true));
/// let context = Handle::new(ShapeProcessContext::new());
/// assert!(op.perform(&context, &Default::default()).unwrap());
/// ```
pub trait ShapeProcessOperator: Transient {
    /// `Perform(context, theProgress)`: runs the operator on the context.
    fn perform(
        &self,
        context: &Handle<ShapeProcessContext>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure>;
}
impl_transient_interface!(
    dyn ShapeProcessOperator,
    "ShapeProcess_Operator",
    StandardTransient
);

/// `ShapeProcess_UOperator`: an operator that calls its function.
pub struct ShapeProcessUOperator {
    parent: StandardTransient,
    function: ShapeProcessOperFunc,
}
impl_transient!(
    ShapeProcessUOperator,
    "ShapeProcess_UOperator",
    dyn ShapeProcessOperator,
    parent,
    interfaces: [dyn ShapeProcessOperator]
);

impl ShapeProcessUOperator {
    /// `ShapeProcess_UOperator(func)`; `ShapeProcessUOperator::new(my_operator)`.
    pub fn new(function: ShapeProcessOperFunc) -> Self {
        Self {
            parent: StandardTransient::default(),
            function,
        }
    }
}

impl ShapeProcessOperator for ShapeProcessUOperator {
    /// `Perform`: the result of the function.
    fn perform(
        &self,
        context: &Handle<ShapeProcessContext>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        (self.function)(context, progress)
    }
}
