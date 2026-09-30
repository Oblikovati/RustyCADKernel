// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_ApproxAFunc2Var.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_ApproxAFunc2Var`: the approximation of a function of two
//! parameters by B-spline surfaces. The constructors, `Init` and `InitGrid` live
//! here; the cutting loops in `approx_a_func2var_perform`, the errors, the
//! conversion and the getters in `approx_a_func2var_results`.
use super::context::{AdvApp2VarContext, FrontierTolerances, SubspaceTolerances};
use super::criterion::AdvApp2VarCriterion;
use super::evaluator::AdvApp2VarEvaluatorFunc2Var;
use super::framework::{AdvApp2VarFramework, AdvApp2VarIsoStrip};
use super::iso::AdvApp2VarIso;
use super::network::AdvApp2VarNetwork;
use super::node::AdvApp2VarNode;
use super::patch::{AdvApp2VarPatch, construction};
use rustycad_tkernel::ncollection::{Array1, Array2, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adv_approx::AdvApproxCutting;
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkmath::geom_abs::{IsoType, Shape};
use rustycad_tkmath::gp::Xy;

/// The error tables of one dimension: maximum, average and on the U and V
/// frontiers (`my3DMaxError`...); only the 3D ones are ever computed.
#[derive(Clone, Default)]
pub(super) struct DimensionErrors {
    pub(super) max: Option<Array1<'static, f64>>,
    pub(super) average: Option<Array1<'static, f64>>,
    pub(super) u_front: Option<Array1<'static, f64>>,
    pub(super) v_front: Option<Array1<'static, f64>>,
}

/// The approximation of a two-parameter function by one B-spline surface per 3D
/// subspace, cut into patches until the tolerances hold (`AdvApp2Var_ApproxAFunc2Var`).
///
/// The native object computes everything in its constructor; a failure there is
/// the Standard_ConstructionError (or other exception) the native code throws.
/// `let approx = AdvApp2VarApproxAFunc2Var::new(0, 0, 1, None, None, Some(&tol), None, None, Some(&tolfr), 0.0, 1.0, 0.0, 1.0, IsoType::IsoV, Shape::C1, Shape::C1, 1, 9, 9, 10, &f, &cut, &cut)?;`
pub struct AdvApp2VarApproxAFunc2Var {
    pub(super) num_sub_spaces: [i32; 3],
    pub(super) tolerances: [Option<Array1<'static, f64>>; 3],
    pub(super) tol_on_front: [Option<Array2<'static, f64>>; 3],
    pub(super) first_par_in_u: f64,
    pub(super) last_par_in_u: f64,
    pub(super) first_par_in_v: f64,
    pub(super) last_par_in_v: f64,
    favorite_iso: IsoType,
    cont_in_u: Shape,
    cont_in_v: Shape,
    precision_code: i32,
    pub(super) max_deg_in_u: i32,
    pub(super) max_deg_in_v: i32,
    pub(super) max_patches: i32,
    pub(super) conditions: AdvApp2VarContext,
    pub(super) result: AdvApp2VarNetwork,
    pub(super) constraints: AdvApp2VarFramework,
    pub(super) done: bool,
    pub(super) has_result: bool,
    pub(super) surfaces: Vec<Handle<GeomBSplineSurface>>,
    pub(super) degree_in_u: i32,
    pub(super) degree_in_v: i32,
    pub(super) errors: [DimensionErrors; 3],
    pub(super) criterion_error: f64,
}

/// The continuity order of a shape, C0..C2 only (`Init`); `which` names the
/// direction in the ConstructionError.
fn continuity_order(shape: Shape, which: &str) -> Result<i32, Failure> {
    match shape {
        Shape::C0 => Ok(0),
        Shape::C1 => Ok(1),
        Shape::C2 => Ok(2),
        _ => Err(construction(&format!(
            "AdvApp2Var_ApproxAFunc2Var : {which}Continuity Error"
        ))),
    }
}

impl AdvApp2VarApproxAFunc2Var {
    /// Approximates `func` on `[first_in_u, last_in_u] x [first_in_v, last_in_v]`
    /// with the tolerances of each subspace, the continuities (C0..C2), the
    /// maximum degrees and patch count, cutting the patches where `u_choice` and
    /// `v_choice` say (`AdvApp2Var_ApproxAFunc2Var(..., Func, UChoice, VChoice)`).
    /// Standard_ConstructionError for a continuity above C2 or a failed
    /// discretisation or approximation. See the type example.
    #[allow(clippy::too_many_arguments)] // The native constructor.
    pub fn new(
        num_1d_ss: i32,
        num_2d_ss: i32,
        num_3d_ss: i32,
        one_d_tol: Option<&Array1<'static, f64>>,
        two_d_tol: Option<&Array1<'static, f64>>,
        three_d_tol: Option<&Array1<'static, f64>>,
        one_d_tol_fr: Option<&Array2<'static, f64>>,
        two_d_tol_fr: Option<&Array2<'static, f64>>,
        three_d_tol_fr: Option<&Array2<'static, f64>>,
        first_in_u: f64,
        last_in_u: f64,
        first_in_v: f64,
        last_in_v: f64,
        favor_iso: IsoType,
        cont_in_u: Shape,
        cont_in_v: Shape,
        precis_code: i32,
        max_deg_in_u: i32,
        max_deg_in_v: i32,
        max_patch: i32,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
    ) -> Result<Self, Failure> {
        let mut approx = Self::stored(
            [num_1d_ss, num_2d_ss, num_3d_ss],
            [one_d_tol.cloned(), two_d_tol.cloned(), three_d_tol.cloned()],
            [
                one_d_tol_fr.cloned(),
                two_d_tol_fr.cloned(),
                three_d_tol_fr.cloned(),
            ],
            [first_in_u, last_in_u, first_in_v, last_in_v],
            (favor_iso, cont_in_u, cont_in_v),
            [precis_code, max_deg_in_u, max_deg_in_v, max_patch],
        );
        approx.init()?;
        approx.perform(u_choice, v_choice, func, None)?;
        approx.convert_bs()?;
        Ok(approx)
    }

    /// As [`new`](Self::new), with a criterion every patch must also satisfy
    /// (`AdvApp2Var_ApproxAFunc2Var(..., Func, Crit, UChoice, VChoice)`).
    #[allow(clippy::too_many_arguments)] // The native constructor.
    pub fn with_criterion(
        num_1d_ss: i32,
        num_2d_ss: i32,
        num_3d_ss: i32,
        one_d_tol: Option<&Array1<'static, f64>>,
        two_d_tol: Option<&Array1<'static, f64>>,
        three_d_tol: Option<&Array1<'static, f64>>,
        one_d_tol_fr: Option<&Array2<'static, f64>>,
        two_d_tol_fr: Option<&Array2<'static, f64>>,
        three_d_tol_fr: Option<&Array2<'static, f64>>,
        first_in_u: f64,
        last_in_u: f64,
        first_in_v: f64,
        last_in_v: f64,
        favor_iso: IsoType,
        cont_in_u: Shape,
        cont_in_v: Shape,
        precis_code: i32,
        max_deg_in_u: i32,
        max_deg_in_v: i32,
        max_patch: i32,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        crit: &dyn AdvApp2VarCriterion,
        u_choice: &dyn AdvApproxCutting,
        v_choice: &dyn AdvApproxCutting,
    ) -> Result<Self, Failure> {
        let mut approx = Self::stored(
            [num_1d_ss, num_2d_ss, num_3d_ss],
            [one_d_tol.cloned(), two_d_tol.cloned(), three_d_tol.cloned()],
            [
                one_d_tol_fr.cloned(),
                two_d_tol_fr.cloned(),
                three_d_tol_fr.cloned(),
            ],
            [first_in_u, last_in_u, first_in_v, last_in_v],
            (favor_iso, cont_in_u, cont_in_v),
            [precis_code, max_deg_in_u, max_deg_in_v, max_patch],
        );
        approx.init()?;
        approx.perform(u_choice, v_choice, func, Some(crit))?;
        approx.convert_bs()?;
        Ok(approx)
    }

    /// The member initialisation list of both constructors.
    fn stored(
        num_sub_spaces: [i32; 3],
        tolerances: [Option<Array1<'static, f64>>; 3],
        tol_on_front: [Option<Array2<'static, f64>>; 3],
        domain: [f64; 4],
        shapes: (IsoType, Shape, Shape),
        codes: [i32; 4],
    ) -> Self {
        Self {
            num_sub_spaces,
            tolerances,
            tol_on_front,
            first_par_in_u: domain[0],
            last_par_in_u: domain[1],
            first_par_in_v: domain[2],
            last_par_in_v: domain[3],
            favorite_iso: shapes.0,
            cont_in_u: shapes.1,
            cont_in_v: shapes.2,
            precision_code: codes[0],
            max_deg_in_u: codes[1],
            max_deg_in_v: codes[2],
            max_patches: codes[3],
            conditions: AdvApp2VarContext::default(),
            result: AdvApp2VarNetwork::default(),
            constraints: AdvApp2VarFramework::default(),
            done: false,
            has_result: false,
            surfaces: Vec::new(),
            degree_in_u: 0,
            degree_in_v: 0,
            errors: Default::default(),
            criterion_error: 0.0,
        }
    }

    /// Builds the context from the orders, degrees and tolerances, then the
    /// one-patch grid (`Init`).
    fn init(&mut self) -> Result<(), Failure> {
        let ifav: i32 = if self.favorite_iso == IsoType::IsoU {
            1
        } else {
            2
        };
        let iu: i32 = continuity_order(self.cont_in_u, "U")?;
        let iv: i32 = continuity_order(self.cont_in_v, "V")?;
        let ndu: i32 = (self.max_deg_in_u + 1).max(2 * iu + 2);
        let ndv: i32 = (self.max_deg_in_v + 1).max(2 * iv + 2);
        if ndu < 2 * iu + 2 {
            return Err(construction(
                "AdvApp2Var_ApproxAFunc2Var : UMaxDegree Error",
            ));
        }
        if ndv < 2 * iv + 2 {
            return Err(construction(
                "AdvApp2Var_ApproxAFunc2Var : VMaxDegree Error",
            ));
        }
        self.precision_code = 0.max(self.precision_code.min(3));
        let [t1, t2, t3] = &self.tolerances;
        let tol = SubspaceTolerances {
            one_d: t1.as_ref(),
            two_d: t2.as_ref(),
            three_d: t3.as_ref(),
        };
        let [f1, f2, f3] = &self.tol_on_front;
        let tof = FrontierTolerances {
            one_d: f1.as_ref(),
            two_d: f2.as_ref(),
            three_d: f3.as_ref(),
        };
        self.conditions = AdvApp2VarContext::new(
            ifav,
            iu,
            iv,
            ndu,
            ndv,
            self.precision_code,
            self.num_sub_spaces,
            tol,
            tof,
        )?;
        self.init_grid(1)
    }

    /// Replaces the network and the framework by a regular grid of `nb_int` x
    /// `nb_int` patches (`InitGrid`).
    pub(super) fn init_grid(&mut self, nb_int: i32) -> Result<(), Failure> {
        let (iu, iv) = (self.conditions.u_order(), self.conditions.v_order());
        let (u0, u1, v0, v1) = (
            self.first_par_in_u,
            self.last_par_in_u,
            self.first_par_in_v,
            self.last_par_in_v,
        );
        let mut net: Sequence<Handle<AdvApp2VarPatch>> = Sequence::default();
        net.append(Handle::new(AdvApp2VarPatch::with_domain(
            [u0, u1, v0, v1],
            iu,
            iv,
        )));
        let (mut the_u, mut the_v) = (Sequence::default(), Sequence::default());
        the_u.append(u0);
        the_v.append(v0);
        the_u.append(u1);
        the_v.append(v1);
        let mut result = AdvApp2VarNetwork::new(net, the_u, the_v);
        let mut bag: Sequence<Handle<AdvApp2VarNode>> = Sequence::default();
        for (u, v) in [(u0, v0), (u1, v0), (u0, v1), (u1, v1)] {
            bag.append(Handle::new(AdvApp2VarNode::with_coord(
                Xy::new(u, v),
                iu,
                iv,
            )?));
        }
        let iso = |iso_type: IsoType, cte: f64, pos: i32| {
            Handle::new(AdvApp2VarIso::with_domain(
                iso_type,
                cte,
                [u0, u1, v0, v1],
                pos,
                iu,
                iv,
            ))
        };
        let (mut bu0, mut bv0): (AdvApp2VarIsoStrip, AdvApp2VarIsoStrip) =
            (Sequence::default(), Sequence::default());
        bu0.append(iso(IsoType::IsoV, v0, 1));
        bu0.append(iso(IsoType::IsoV, v1, 2));
        bv0.append(iso(IsoType::IsoU, u0, 3));
        bv0.append(iso(IsoType::IsoU, u1, 4));
        let (mut u_strip, mut v_strip) = (Sequence::default(), Sequence::default());
        u_strip.append(bu0);
        v_strip.append(bv0);
        let mut constraints = AdvApp2VarFramework::new(bag, u_strip, v_strip);
        let deltu: f64 = (u1 - u0) / f64::from(nb_int);
        let deltv: f64 = (v1 - v0) / f64::from(nb_int);
        for iint in 1..nb_int {
            result.update_in_u(u0 + f64::from(iint) * deltu)?;
            constraints.update_in_u(u0 + f64::from(iint) * deltu)?;
            result.update_in_v(v0 + f64::from(iint) * deltv)?;
            constraints.update_in_v(v0 + f64::from(iint) * deltv)?;
        }
        self.result = result;
        self.constraints = constraints;
        Ok(())
    }
}
