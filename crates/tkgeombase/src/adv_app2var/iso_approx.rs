//! `AdvApp2Var_Iso::MakeApprox`: the approximation of an iso-curve and of its
//! cross derivatives with `mma2fnc_`, one piece per derivative order.
use super::approx_f2var::{IsoApproxOutput, IsoApproxRequest, mma2fnc_};
use super::context::AdvApp2VarContext;
use super::evaluator::AdvApp2VarEvaluatorFunc2Var;
use super::iso::{AdvApp2VarIso, IsoState};
use super::node::AdvApp2VarNode;
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::Point3;

/// The tables of a successful `MakeApprox`.
pub(super) struct IsoTables {
    pub(super) equation: Array1<'static, f64>,
    pub(super) nb_coeff: i32,
    pub(super) max_errors: Array2<'static, f64>,
    pub(super) moy_errors: Array2<'static, f64>,
    pub(super) som_tab: Array1<'static, f64>,
    pub(super) dif_tab: Array1<'static, f64>,
}

/// The outcome of `MakeApprox`; the tables are kept only when there is a result.
pub(super) struct IsoResult {
    pub(super) done: bool,
    pub(super) has_result: bool,
    pub(super) tables: Option<IsoTables>,
}

fn array_failure(error: ArrayError) -> Failure {
    error.into_failure("AdvApp2Var_Iso::MakeApprox")
}

fn missing(table: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("AdvApp2Var_Iso::MakeApprox: the context has no {table}"),
    )
}

/// The iso-dependent inputs of `mma2fnc_` (the type switch of `MakeApprox`).
struct IsoSetup {
    isofav: i32,
    tabdec: [f64; 2],
    uvfonc: [f64; 4],
    nbroot: i32,
    rootlg: Vec<f64>,
    ndgjac: i32,
    ncflim: i32,
    epsapr: Vec<f64>,
}

fn setup(
    state: &IsoState,
    conditions: &AdvApp2VarContext,
    domain: [f64; 4],
) -> Result<IsoSetup, Failure> {
    let mut uvfonc: [f64; 4] = domain;
    let roots = |r: Option<&Array1<'static, f64>>| -> Result<(i32, Vec<f64>), Failure> {
        let roots = r.ok_or_else(|| missing("roots"))?;
        let nbroot: i32 = roots.length() - if state.extrem_order > -1 { 2 } else { 0 };
        Ok((nbroot, roots.as_slice().to_vec()))
    };
    let (isofav, tabdec, (nbroot, rootlg), ndgjac, ncflim) = match state.iso_type {
        IsoType::IsoV => {
            (uvfonc[0], uvfonc[1]) = (state.u0, state.u1);
            (
                2,
                [state.u0, state.u1],
                roots(conditions.u_roots())?,
                conditions.u_jac_deg(),
                conditions.u_limit(),
            )
        }
        IsoType::IsoU => {
            (uvfonc[2], uvfonc[3]) = (state.v0, state.v1);
            (
                1,
                [state.v0, state.v1],
                roots(conditions.v_roots())?,
                conditions.v_jac_deg(),
                conditions.v_limit(),
            )
        }
        IsoType::NoneIso => (0, [0.0; 2], (0, Vec::new()), 0, 1),
    };
    let epsapr: Vec<f64> = tolerances(state.position, conditions)?;
    Ok(IsoSetup {
        isofav,
        tabdec,
        uvfonc,
        nbroot,
        rootlg,
        ndgjac,
        ncflim,
        epsapr,
    })
}

/// The tolerance of each subspace for the iso's position: the cutting tolerance of
/// an inner line, the frontier tolerance of a border (`HEPSAPR`). Another position
/// leaves the native array uninitialised; the port uses zeros.
fn tolerances(position: i32, conditions: &AdvApp2VarContext) -> Result<Vec<f64>, Failure> {
    let nbsesp: i32 = conditions.total_number_ssp();
    let table: Option<&Array2<'static, f64>> = match position {
        0 => conditions.c_toler(),
        1..=4 => conditions.f_toler(),
        _ => return Ok(vec![0.0; nbsesp.max(0) as usize]),
    };
    let table = table.ok_or_else(|| missing("tolerances"))?;
    let column: i32 = position.max(1);
    (1..=nbsesp)
        .map(|iesp| table.value(iesp, column).copied().map_err(array_failure))
        .collect()
}

impl AdvApp2VarIso {
    /// Approximates the function and its cross derivatives up to the derivative
    /// order on the iso within `[u0, u1] x [v0, v1]`, and stores in `node_begin` and
    /// `node_end` the values and derivatives up to the end order at the iso's ends
    /// (`MakeApprox`). An evaluator failure leaves the iso without result and keeps
    /// the previous tables, as natively. Only 3D functions (one subspace) are
    /// supported, as the native code assumes. `iso.make_approx(&ctx, 0.0, 1.0, 0.0,
    /// 1.0, &f, &n1, &n2)?`.
    #[allow(clippy::too_many_arguments)] // The native member list.
    pub fn make_approx(
        &self,
        conditions: &AdvApp2VarContext,
        u0: f64,
        u1: f64,
        v0: f64,
        v1: f64,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
        node_begin: &AdvApp2VarNode,
        node_end: &AdvApp2VarNode,
    ) -> Result<(), Failure> {
        let state: IsoState = self.state.read().clone();
        let setup: IsoSetup = setup(&state, conditions, [u0, u1, v0, v1])?;
        let result: IsoResult =
            approximate(&state, conditions, setup, func, [node_begin, node_end])?;
        self.store(result);
        Ok(())
    }
}

/// The derivative loop of `MakeApprox`: one `mma2fnc_` call per derivative order,
/// with the end constraints copied into the nodes and the errors into the tables.
fn approximate(
    state: &IsoState,
    conditions: &AdvApp2VarContext,
    mut s: IsoSetup,
    func: &dyn AdvApp2VarEvaluatorFunc2Var,
    nodes: [&AdvApp2VarNode; 2],
) -> Result<IsoResult, Failure> {
    let (ndimen, nbsesp) = (conditions.total_dimension(), conditions.total_number_ssp());
    let (iordre, ideriv) = (state.extrem_order, state.deriv_order);
    let szcrb: i32 = ndimen * s.ncflim;
    let sztab: i32 = (1 + s.nbroot / 2) * ndimen;
    let one_based = |n: i32| Array1::<f64>::with_bounds(1, n).map_err(array_failure);
    let mut courbe = one_based(szcrb * (ideriv + 1))?;
    let mut somtab = one_based(sztab * (ideriv + 1))?;
    let mut diftab = one_based(sztab * (ideriv + 1))?;
    let (mut contr1, mut contr2) = (
        one_based((iordre + 2) * ndimen)?,
        one_based((iordre + 2) * ndimen)?,
    );
    let mut errmax: Array2<'static, f64> =
        Array2::with_bounds(1, nbsesp, 1, ideriv + 1).map_err(array_failure)?;
    let mut errmoy: Array2<'static, f64> =
        Array2::with_bounds(1, nbsesp, 1, ideriv + 1).map_err(array_failure)?;
    let (mut emxapp, mut emyapp) = (
        vec![0.0; nbsesp.max(0) as usize],
        vec![0.0; nbsesp.max(0) as usize],
    );
    let (mut iercod, mut ncoeff) = (0, 0);
    for iapp in 0..=ideriv {
        let request = IsoApproxRequest {
            ndimen,
            nbsesp,
            ndimse: &[3],
            uvfonc: s.uvfonc,
            tconst: state.const_par,
            isofav: s.isofav,
            nbroot: s.nbroot,
            rootlg: &s.rootlg,
            iordre,
            ideriv: iapp,
            ndgjac: s.ndgjac,
            nbcrmx: 1,
            ncflim: s.ncflim,
            epsapr: &s.epsapr,
        };
        let (mut ncfapp, mut ierapp) = ([0], 0);
        let (crb, som, dif) = (
            (iapp * szcrb) as usize,
            (iapp * sztab) as usize,
            (iapp * sztab) as usize,
        );
        let mut output = IsoApproxOutput {
            ncoeff: &mut ncfapp,
            courbe: &mut courbe.as_mut_slice()[crb..],
            nbcrbe: 0,
            somtab: &mut somtab.as_mut_slice()[som..],
            diftab: &mut diftab.as_mut_slice()[dif..],
            contr1: contr1.as_mut_slice(),
            contr2: contr2.as_mut_slice(),
            tabdec: &mut s.tabdec,
            errmax: &mut emxapp,
            errmoy: &mut emyapp,
        };
        mma2fnc_(&request, func, &mut output, &mut ierapp);
        if ierapp > 0 {
            return Ok(IsoResult {
                done: false,
                has_result: false,
                tables: None,
            });
        }
        if ncoeff <= ncfapp[0] {
            ncoeff = ncfapp[0];
        }
        if ierapp == -1 {
            iercod = -1;
        }
        store_end_points(nodes, [&contr1, &contr2], s.isofav, iordre, iapp)?;
        for iesp in 1..=nbsesp {
            errmax
                .set_value(iesp, iapp + 1, emxapp[(iesp - 1) as usize])
                .map_err(array_failure)?;
            errmoy
                .set_value(iesp, iapp + 1, emyapp[(iesp - 1) as usize])
                .map_err(array_failure)?;
        }
    }
    let tables = IsoTables {
        equation: courbe,
        nb_coeff: ncoeff,
        max_errors: errmax,
        moy_errors: errmoy,
        som_tab: somtab,
        dif_tab: diftab,
    };
    Ok(IsoResult {
        done: iercod == 0,
        has_result: true,
        tables: Some(tables),
    })
}

/// Copies the end constraints of orders `0..=iordre` of derivative `iapp` into the
/// nodes: `(ider, iapp)` for a V-iso, `(iapp, ider)` for a U-iso.
fn store_end_points(
    nodes: [&AdvApp2VarNode; 2],
    contr: [&Array1<'static, f64>; 2],
    isofav: i32,
    iordre: i32,
    iapp: i32,
) -> Result<(), Failure> {
    for (node, table) in nodes.into_iter().zip(contr) {
        let values: &[f64] = table.as_slice();
        for ider in 0..=iordre {
            let jpos: usize = (3 * ider) as usize;
            let pt = Point3::new(values[jpos], values[jpos + 1], values[jpos + 2]);
            if isofav == 2 {
                node.set_point(ider, iapp, pt)?;
            } else {
                node.set_point(iapp, ider, pt)?;
            }
        }
    }
    Ok(())
}
