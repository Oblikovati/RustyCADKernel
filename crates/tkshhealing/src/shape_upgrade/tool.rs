// Rust translation of ModelingAlgorithms/TKShHealing/ShapeUpgrade/ShapeUpgrade_Tool.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeUpgrade_Tool`: the context and tolerances every ShapeUpgrade tool shares
//! (docs/shhealing-design.md section 8).
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};

/// The fields of a `ShapeUpgrade_Tool`.
#[derive(Clone)]
struct ToolFields {
    context: Handle<ShapeBuildReShape>,
    precision: f64,
    min_tol: f64,
    max_tol: f64,
}

/// `ShapeUpgrade_Tool`: a re-shaping context, a working precision and the tolerance
/// limits, the root of the ShapeUpgrade tools.
///
/// ```
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeTool;
/// let tool = ShapeUpgradeTool::new();
/// tool.set_precision(2.0);
/// assert_eq!(tool.max_tolerance(), 2.0);
/// assert_eq!(tool.limit_tolerance(5.0), 2.0);
/// ```
pub struct ShapeUpgradeTool {
    parent: StandardTransient,
    fields: SharedState<ToolFields>,
}
impl_transient!(
    ShapeUpgradeTool,
    "ShapeUpgrade_Tool",
    StandardTransient,
    parent
);

impl Default for ShapeUpgradeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeUpgradeTool {
    /// `ShapeUpgrade_Tool()`: no context, precision and minimal tolerance
    /// `Precision::Confusion()`, maximal tolerance 1 ("Precision::Infinite() ?? pdn");
    /// `ShapeUpgradeTool::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(ToolFields {
                context: Handle::default(),
                precision: precision::confusion(),
                min_tol: precision::confusion(),
                max_tol: 1.0,
            }),
        }
    }

    /// `Set(tool)`: copies the context, the precision and the tolerances of `tool`;
    /// Standard_NullObject for a null tool (a null dereference natively);
    /// `tool.set(&other)?`.
    pub fn set(&self, tool: &Handle<ShapeUpgradeTool>) -> Result<(), Failure> {
        let source = tool.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "ShapeUpgrade_Tool::Set: null tool",
                "",
            )
        })?;
        let copied = source.fields.read().clone();
        *self.fields.write() = copied;
        Ok(())
    }

    /// `SetContext(context)`; `tool.set_context(&reshape)`.
    pub fn set_context(&self, context: &Handle<ShapeBuildReShape>) {
        self.fields.write().context = context.clone();
    }

    /// `Context()`; `tool.context()`.
    pub fn context(&self) -> Handle<ShapeBuildReShape> {
        self.fields.read().context.clone()
    }

    /// `SetPrecision(preci)`: the precision, the maximal tolerance raised to it and the
    /// minimal one lowered to it; `tool.set_precision(1e-3)`.
    pub fn set_precision(&self, preci: f64) {
        let mut fields = self.fields.write();
        fields.precision = preci;
        if fields.max_tol < fields.precision {
            fields.max_tol = fields.precision;
        }
        if fields.min_tol > fields.precision {
            fields.min_tol = fields.precision;
        }
    }

    /// `Precision()`; `tool.precision()`.
    pub fn precision(&self) -> f64 {
        self.fields.read().precision
    }

    /// `SetMinTolerance(mintol)`; `tool.set_min_tolerance(1e-7)`.
    pub fn set_min_tolerance(&self, mintol: f64) {
        self.fields.write().min_tol = mintol;
    }

    /// `MinTolerance()`; `tool.min_tolerance()`.
    pub fn min_tolerance(&self) -> f64 {
        self.fields.read().min_tol
    }

    /// `SetMaxTolerance(maxtol)`; `tool.set_max_tolerance(1.0)`.
    pub fn set_max_tolerance(&self, maxtol: f64) {
        self.fields.write().max_tol = maxtol;
    }

    /// `MaxTolerance()`; `tool.max_tolerance()`.
    pub fn max_tolerance(&self) -> f64 {
        self.fields.read().max_tol
    }

    /// `LimitTolerance(toler)`: `std::min(MaxTolerance(), toler)` (the maximal tolerance
    /// when `toler` is NaN, as `std::min`); `tool.limit_tolerance(5.0)`.
    pub fn limit_tolerance(&self, toler: f64) -> f64 {
        let max_tol = self.fields.read().max_tol;
        if toler < max_tol { toler } else { max_tol }
    }
}
