// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Patch.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Patch`: the approximation of the function on one rectangle of the
//! cutting grid. `Discretise` lives in `patch_discretise`, `MakeApprox` in
//! `patch_make`, `AddConstraints` and `AddErrors` in `patch_constraints`.
use super::arrays::one_based_from;
use super::context::AdvApp2VarContext;
use super::criterion::{AdvApp2VarCriterion, AdvApp2VarCriterionType};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::convert::GridPolynomialToPoles;
use rustycad_tkmath::gp::Point3;

/// The locked fields of a patch.
#[derive(Clone, Default)]
pub(super) struct PatchState {
    pub(super) u0: f64,
    pub(super) u1: f64,
    pub(super) v0: f64,
    pub(super) v1: f64,
    pub(super) ord_in_u: i32,
    pub(super) ord_in_v: i32,
    pub(super) nb_coeff_in_u: i32,
    pub(super) nb_coeff_in_v: i32,
    pub(super) appr_is_done: bool,
    pub(super) has_result: bool,
    pub(super) equation: Option<Array1<'static, f64>>,
    pub(super) max_errors: Option<Array1<'static, f64>>,
    pub(super) moy_errors: Option<Array1<'static, f64>>,
    pub(super) iso_errors: Option<Array2<'static, f64>>,
    pub(super) cut_sense: i32,
    pub(super) disc_is_done: bool,
    pub(super) soso_tab: Option<Array1<'static, f64>>,
    pub(super) diso_tab: Option<Array1<'static, f64>>,
    pub(super) sodi_tab: Option<Array1<'static, f64>>,
    pub(super) didi_tab: Option<Array1<'static, f64>>,
    pub(super) crit_value: f64,
}

/// The polynomial approximation of the function on `[U0,U1]x[V0,V1]`, with its
/// discretisation, errors and cutting decision (`AdvApp2Var_Patch`).
///
/// Shared through handles; every member takes `&self`.
/// `AdvApp2VarPatch::with_domain([0.0, 1.0, 0.0, 1.0], 1, 1).u_order() == 1`.
pub struct AdvApp2VarPatch {
    parent: StandardTransient,
    pub(super) state: SharedState<PatchState>,
}
impl_transient!(
    AdvApp2VarPatch,
    "AdvApp2Var_Patch",
    StandardTransient,
    parent
);

impl Default for AdvApp2VarPatch {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn construction(message: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", message)
}

impl AdvApp2VarPatch {
    /// The unit square with orders (0, 0) (`AdvApp2Var_Patch()`); `AdvApp2VarPatch::new()`.
    pub fn new() -> Self {
        Self::with_domain([0.0, 1.0, 0.0, 1.0], 0, 0)
    }

    /// The patch `domain = [U0, U1, V0, V1]` with continuity orders `(iu, iv)`
    /// (`AdvApp2Var_Patch(U0, U1, V0, V1, iu, iv)`);
    /// `AdvApp2VarPatch::with_domain([0.0, 1.0, 0.0, 1.0], 1, 1)`.
    pub fn with_domain(domain: [f64; 4], iu: i32, iv: i32) -> Self {
        let state = PatchState {
            u0: domain[0],
            u1: domain[1],
            v0: domain[2],
            v1: domain[3],
            ord_in_u: iu,
            ord_in_v: iv,
            ..PatchState::default()
        };
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }

    /// Whether `Discretise` succeeded (`IsDiscretised`); `patch.is_discretised()`.
    pub fn is_discretised(&self) -> bool {
        self.state.read().disc_is_done
    }

    /// Whether the approximation met its tolerance (`IsApproximated`);
    /// `patch.is_approximated()`.
    pub fn is_approximated(&self) -> bool {
        self.state.read().appr_is_done
    }

    /// Whether an approximation exists (`HasResult`); `patch.has_result()`.
    pub fn has_result(&self) -> bool {
        self.state.read().has_result
    }

    /// Replaces the domain (`ChangeDomain`); `patch.change_domain(0.0, 0.5, 0.0, 1.0)`.
    pub fn change_domain(&self, a: f64, b: f64, c: f64, d: f64) {
        let mut state = self.state.write();
        (state.u0, state.u1, state.v0, state.v1) = (a, b, c, d);
    }

    /// Forgets the approximation status (`ResetApprox`); `patch.reset_approx()`.
    pub fn reset_approx(&self) {
        let mut state = self.state.write();
        state.appr_is_done = false;
        state.has_result = false;
    }

    /// Accepts an existing result as approximated (`OverwriteApprox`);
    /// `patch.overwrite_approx()`.
    pub fn overwrite_approx(&self) {
        let mut state = self.state.write();
        if state.has_result {
            state.appr_is_done = true;
        }
    }

    /// `U0`; `patch.u0()`.
    pub fn u0(&self) -> f64 {
        self.state.read().u0
    }

    /// `U1`; `patch.u1()`.
    pub fn u1(&self) -> f64 {
        self.state.read().u1
    }

    /// `V0`; `patch.v0()`.
    pub fn v0(&self) -> f64 {
        self.state.read().v0
    }

    /// `V1`; `patch.v1()`.
    pub fn v1(&self) -> f64 {
        self.state.read().v1
    }

    /// The continuity order in U (`UOrder`); `patch.u_order()`.
    pub fn u_order(&self) -> i32 {
        self.state.read().ord_in_u
    }

    /// The continuity order in V (`VOrder`); `patch.v_order()`.
    pub fn v_order(&self) -> i32 {
        self.state.read().ord_in_v
    }

    /// The cutting decision of the last `MakeApprox`: 0 none, 1 U, 2 V, 3 both
    /// (`CutSense()`); `patch.cut_sense()`.
    pub fn cut_sense(&self) -> i32 {
        self.state.read().cut_sense
    }

    /// The cutting decision under a criterion (`CutSense(Crit, NumDec)`): the
    /// approximation's for a relative criterion not approximated, else 0 when the
    /// criterion holds and `num_dec` otherwise. `patch.cut_sense_with(&crit, 3)`.
    pub fn cut_sense_with(&self, crit: &dyn AdvApp2VarCriterion, num_dec: i32) -> i32 {
        let crit_rel: bool = crit.criterion_type() == AdvApp2VarCriterionType::Relative;
        if crit_rel && !self.is_approximated() {
            return self.cut_sense();
        }
        if crit.is_satisfied(self) { 0 } else { num_dec }
    }

    /// The coefficient count in U (`NbCoeffInU`); `patch.nb_coeff_in_u()`.
    pub fn nb_coeff_in_u(&self) -> i32 {
        self.state.read().nb_coeff_in_u
    }

    /// The coefficient count in V (`NbCoeffInV`); `patch.nb_coeff_in_v()`.
    pub fn nb_coeff_in_v(&self) -> i32 {
        self.state.read().nb_coeff_in_v
    }

    /// Raises the coefficient counts to at least the given ones (`ChangeNbCoeff`);
    /// `patch.change_nb_coeff(6, 6)`.
    pub fn change_nb_coeff(&self, nb_coeff_u: i32, nb_coeff_v: i32) {
        let mut state = self.state.write();
        if state.nb_coeff_in_u < nb_coeff_u {
            state.nb_coeff_in_u = nb_coeff_u;
        }
        if state.nb_coeff_in_v < nb_coeff_v {
            state.nb_coeff_in_v = nb_coeff_v;
        }
    }

    /// The canonical coefficients of subspace `ssp_index`, only for one subspace
    /// (`Coefficients`): `(NDIMEN, NCFLMV, NCFLMU)` then scratch values of
    /// `MakeApprox` up to the scratch size (uninitialised natively, zero-based here).
    /// Standard_ConstructionError for another index or context. `patch.coefficients(1, &ctx)?`.
    pub fn coefficients(
        &self,
        ssp_index: i32,
        cond: &AdvApp2VarContext,
    ) -> Result<Option<Array1<'static, f64>>, Failure> {
        if cond.total_number_ssp() != 1 || ssp_index != 1 {
            return Err(construction(
                "AdvApp2Var_Patch::Poles :  SSPIndex out of range",
            ));
        }
        Ok(self.state.read().equation.clone())
    }

    /// The Bezier poles of the patch on [-1,1]x[-1,1] (`Poles`), converted by
    /// `Convert_GridPolynomialToPoles`; Standard_ConstructionError for another
    /// subspace. `patch.poles(1, &ctx)?`.
    pub fn poles(
        &self,
        ssp_index: i32,
        cond: &AdvApp2VarContext,
    ) -> Result<Array2<'static, Point3>, Failure> {
        let equation = self.coefficients(ssp_index, cond)?.ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "AdvApp2Var_Patch::Poles: no approximation",
            )
        })?;
        let interval: Array1<'static, f64> = one_based_from(&[-1.0, 1.0]);
        let (u, v) = (self.nb_coeff_in_u(), self.nb_coeff_in_v());
        let counts: Array1<'static, i32> = one_based_from(&[u, v]);
        let conversion = GridPolynomialToPoles::from_single_patch(
            cond.u_limit() - 1,
            cond.v_limit() - 1,
            &counts,
            &equation,
            &interval,
            &interval,
        )?;
        Ok(conversion.poles()?.clone())
    }

    /// The maximum error of each subspace (`MaxErrors`); `patch.max_errors()`.
    pub fn max_errors(&self) -> Option<Array1<'static, f64>> {
        self.state.read().max_errors.clone()
    }

    /// The average error of each subspace (`AverageErrors`); `patch.average_errors()`.
    pub fn average_errors(&self) -> Option<Array1<'static, f64>> {
        self.state.read().moy_errors.clone()
    }

    /// The maximum errors on the four borders `(NBSESP, 4)` (`IsoErrors`);
    /// `patch.iso_errors()`.
    pub fn iso_errors(&self) -> Option<Array2<'static, f64>> {
        self.state.read().iso_errors.clone()
    }

    /// The criterion value stored by `SetCritValue` (`CritValue`); `patch.crit_value()`.
    pub fn crit_value(&self) -> f64 {
        self.state.read().crit_value
    }

    /// Stores a criterion value (`SetCritValue`); `patch.set_crit_value(0.1)`.
    pub fn set_crit_value(&self, dist: f64) {
        self.state.write().crit_value = dist;
    }
}
