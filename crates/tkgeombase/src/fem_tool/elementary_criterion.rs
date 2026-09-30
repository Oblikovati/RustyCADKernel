// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_ElementaryCriterion.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_ElementaryCriterion`: the abstract quadratic criterion J(E) of one
//! element E of a finite-element curve.
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::{Array2, HArray2};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkmath::math::{Matrix, Vector};
use std::sync::RwLockReadGuard;

/// The protected fields of `FEmTool_ElementaryCriterion`: the element coefficients
/// (`myCoeff`, one row per basis function and one column per dimension) and the
/// element interval (`myFirst`, `myLast`).
#[derive(Clone, Default)]
pub struct CriterionFields {
    /// `myCoeff`; null until `Set(Coeff)`.
    pub coeff: Handle<HArray2<f64>>,
    /// `myFirst` (0 initially).
    pub first: f64,
    /// `myLast` (0 initially).
    pub last: f64,
}

/// The state of `FEmTool_ElementaryCriterion`, shared by every handle to the
/// criterion (the native members are set through handles);
/// `CriterionBase::new().fields().first == 0.0`.
#[derive(Default)]
pub struct CriterionBase {
    state: SharedState<CriterionFields>,
}

impl CriterionBase {
    /// `FEmTool_ElementaryCriterion()`: no coefficients and the interval [0, 0];
    /// `CriterionBase::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Set(Coeff)`: keeps the handle, so later changes of the array are seen;
    /// `base.set_coeff(&coeff)`.
    pub fn set_coeff(&self, coeff: &Handle<HArray2<f64>>) {
        self.state.write().coeff = coeff.clone();
    }

    /// `Set(FirstKnot, LastKnot)`; `base.set_knots(0.0, 1.0)`.
    pub fn set_knots(&self, first_knot: f64, last_knot: f64) {
        let mut fields = self.state.write();
        fields.first = first_knot;
        fields.last = last_knot;
    }

    /// Read access to the protected fields; `base.fields().last`.
    pub fn fields(&self) -> RwLockReadGuard<'_, CriterionFields> {
        self.state.read()
    }
}

/// `FEmTool_ElementaryCriterion`: implemented by the linear criteria; a handle to
/// the trait object (`Handle<dyn ElementaryCriterion>`) is the native base handle.
///
/// ```
/// use rustycad_tkgeombase::fem_tool::{ElementaryCriterion, LinearFlexion};
/// use rustycad_tkmath::geom_abs::Shape;
/// let criterion = LinearFlexion::new(5, Shape::C1)?;
/// criterion.set(0.0, 2.0);
/// assert_eq!(criterion.criterion().fields().last, 2.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait ElementaryCriterion: Transient {
    /// The base-class state (`myCoeff`, `myFirst`, `myLast`); `criterion.criterion()`.
    fn criterion(&self) -> &CriterionBase;

    /// `Set(Coeff)` (not virtual natively); `criterion.set_coeff(&coeff)`.
    fn set_coeff(&self, coeff: &Handle<HArray2<f64>>) {
        self.criterion().set_coeff(coeff);
    }

    /// The virtual `Set(FirstKnot, LastKnot)`; `criterion.set(0.0, 1.0)`.
    fn set(&self, first_knot: f64, last_knot: f64) {
        self.criterion().set_knots(first_knot, last_knot);
    }

    /// `DependenceTable()`: which dimension pairs have a Hessian block;
    /// `criterion.dependence_table()?`.
    fn dependence_table(&self) -> Result<Handle<HArray2<i32>>, Failure>;

    /// `Value()`: J(E) for the current element; `criterion.value()?`.
    fn value(&self) -> Result<f64, Failure>;

    /// `Hessian(Dim1, Dim2, H)`: the Hessian block of J(E) in the dimensions
    /// `dimension1` and `dimension2`; `criterion.hessian(1, 1, &mut h)?`.
    fn hessian(&self, dimension1: i32, dimension2: i32, h: &mut Matrix<'_>) -> Result<(), Failure>;

    /// `Gradient(Dim, G)`: the gradient of J(E) in `dimension`;
    /// `criterion.gradient(1, &mut g)?`.
    fn gradient(&self, dimension: i32, g: &mut Vector<'_>) -> Result<(), Failure>;
}
impl_transient_interface!(
    dyn ElementaryCriterion,
    "FEmTool_ElementaryCriterion",
    StandardTransient
);

/// The element coefficients, or the error the port gives where the native code
/// dereferences a null `myCoeff` (a crash natively);
/// `coefficients(&fields, "FEmTool_LinearFlexion::Value")?`.
pub(crate) fn coefficients(
    fields: &CriterionFields,
    context: &str,
) -> Result<Array2<'static, f64>, Failure> {
    let Some(coeff) = fields.coeff.get() else {
        return Err(crate::native_failure::failure(
            "Standard_NullObject",
            &format!("{context}: the element coefficients are not set; expected Set(Coeff) first"),
        ));
    };
    Ok(coeff.array2().clone())
}
