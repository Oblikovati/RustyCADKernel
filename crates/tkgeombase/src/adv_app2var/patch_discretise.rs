//! `AdvApp2Var_Patch::Discretise`: the samples of the function on the patch minus
//! the interpolation of its boundary constraints.
use super::approx_f2var::{
    CornerConstraints, IsoTables, SampledSquare, SquareGrid, SquareTables, mma2cdi_, mma2ds1_,
};
use super::arrays::required;
use super::context::AdvApp2VarContext;
use super::evaluator::AdvApp2VarEvaluatorFunc2Var;
use super::framework::AdvApp2VarFramework;
use super::patch::{AdvApp2VarPatch, PatchState};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// `pow(x, n)` of the C++ code: `std::pow(double, int)` promotes the exponent and
/// calls the libm `pow`, which `powf` also reaches.
pub(super) fn cpp_pow(x: f64, n: i32) -> f64 {
    x.powf(f64::from(n))
}

/// The number of roots a direction samples: the context roots minus the two end
/// slots when the order is at least 0.
fn sampled_roots(roots: &Array1<'static, f64>, order: i32) -> i32 {
    roots.length() - if order > -1 { 2 } else { 0 }
}

/// The four corner nodes of the patch, in the native order (U0,V0), (U1,V0),
/// (U0,V1), (U1,V1).
pub(super) fn corners(s: &PatchState) -> [(f64, f64); 4] {
    [(s.u0, s.v0), (s.u1, s.v0), (s.u0, s.v1), (s.u1, s.v1)]
}

/// The corner constraints `(ndimen, iordru+2, iordrv+2)` of the four nodes, each
/// derivative scaled to [-1,1] by `rho = du^iu dv^iv` and multiplied by `sign`.
pub(super) fn corner_values(
    s: &PatchState,
    frame: &AdvApp2VarFramework,
    ndimen: i32,
    sign: f64,
) -> Result<Vec<f64>, Failure> {
    let size: i32 = ndimen * (s.ord_in_u + 2) * (s.ord_in_v + 2);
    let mut coins: Vec<f64> = vec![0.0; (4 * size) as usize];
    let (du, dv) = ((s.u1 - s.u0) / 2., (s.v1 - s.v0) / 2.);
    for iu in 0..=s.ord_in_u {
        for iv in 0..=s.ord_in_v {
            let rho: f64 = cpp_pow(du, iu) * cpp_pow(dv, iv);
            for (k, (u, v)) in (0..).zip(corners(s)) {
                let point: Point3 = frame.node_at(u, v)?.point(iu, iv)?;
                let base: usize =
                    (k * size + ndimen * iu + ndimen * (s.ord_in_u + 2) * iv) as usize;
                coins[base] = sign * rho * point.x();
                coins[base + 1] = sign * rho * point.y();
                coins[base + 2] = sign * rho * point.z();
            }
        }
    }
    Ok(coins)
}

/// A copy of an iso table (`HArray1->ChangeArray1() = Iso.SomTab()->Array1()`, which
/// takes the length of the copied table), scaled per derivative block by
/// `step^order`; Standard_OutOfRange when a scaled block leaves the table.
/// `scaled_iso_table(iso.som_tab(), 12, 1, 0.5, DISCRETISE)?`.
pub(super) fn scaled_iso_table(
    table: Option<Array1<'static, f64>>,
    block: i32,
    orders: i32,
    step: f64,
    member: &str,
) -> Result<Vec<f64>, Failure> {
    let table = required(table, member)?;
    let mut values: Vec<f64> = table.as_slice().to_vec();
    for order in 1..=orders {
        let rho: f64 = cpp_pow(step, order);
        let range = (order * block) as usize..((order + 1) * block) as usize;
        let Some(scaled) = values.get_mut(range.clone()) else {
            return Err(Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "{member}: iso table of {} values, block {range:?} expected inside",
                    table.length()
                ),
            ));
        };
        for value in scaled {
            *value *= rho;
        }
    }
    Ok(values)
}

const DISCRETISE: &str = "AdvApp2Var_Patch::Discretise";

impl AdvApp2VarPatch {
    /// Samples `func` on the patch at the Legendre roots of the context and removes
    /// the Hermite interpolation of the corner nodes and border isos of
    /// `constraints` (`Discretise`); the four symmetric tables are kept when it
    /// succeeds. Only 3D functions are supported, as natively.
    /// `patch.discretise(&ctx, &frame, &f)?`.
    pub fn discretise(
        &self,
        conditions: &AdvApp2VarContext,
        constraints: &AdvApp2VarFramework,
        func: &dyn AdvApp2VarEvaluatorFunc2Var,
    ) -> Result<(), Failure> {
        let s: PatchState = self.state.read().clone();
        let ndimen: i32 = conditions.total_dimension();
        let uroot = required(
            conditions.u_roots(),
            "AdvApp2Var_Patch::Discretise: U roots",
        )?;
        let vroot = required(
            conditions.v_roots(),
            "AdvApp2Var_Patch::Discretise: V roots",
        )?;
        let (nbpntu, nbpntv) = (
            sampled_roots(uroot, s.ord_in_u),
            sampled_roots(vroot, s.ord_in_v),
        );
        let coins: Vec<f64> = corner_values(&s, constraints, ndimen, 1.0)?;
        let (du, dv) = ((s.u1 - s.u0) / 2., (s.v1 - s.v0) / 2.);
        let (iu0, iu1) = (
            constraints.iso_u(s.u0, s.v0, s.v1)?,
            constraints.iso_u(s.u1, s.v0, s.v1)?,
        );
        let block_u: i32 = (1 + nbpntv / 2) * ndimen;
        let su0 = scaled_iso_table(iu0.som_tab(), block_u, s.ord_in_u, du, DISCRETISE)?;
        let du0 = scaled_iso_table(iu0.dif_tab(), block_u, s.ord_in_u, du, DISCRETISE)?;
        let su1 = scaled_iso_table(iu1.som_tab(), block_u, s.ord_in_u, du, DISCRETISE)?;
        let du1 = scaled_iso_table(iu1.dif_tab(), block_u, s.ord_in_u, du, DISCRETISE)?;
        let (iv0, iv1) = (
            constraints.iso_v(s.u0, s.u1, s.v0)?,
            constraints.iso_v(s.u0, s.u1, s.v1)?,
        );
        let block_v: i32 = (1 + nbpntu / 2) * ndimen;
        let sv0 = scaled_iso_table(iv0.som_tab(), block_v, s.ord_in_v, dv, DISCRETISE)?;
        let dv0 = scaled_iso_table(iv0.dif_tab(), block_v, s.ord_in_v, dv, DISCRETISE)?;
        let sv1 = scaled_iso_table(iv1.som_tab(), block_v, s.ord_in_v, dv, DISCRETISE)?;
        let dv1 = scaled_iso_table(iv1.dif_tab(), block_v, s.ord_in_v, dv, DISCRETISE)?;
        let full: usize = ((1 + nbpntu / 2) * (1 + nbpntv / 2) * ndimen) as usize;
        let half: usize = ((nbpntu / 2) * (nbpntv / 2) * ndimen) as usize;
        let (mut soso, mut didi) = (vec![0.0; full], vec![0.0; full]);
        let (mut sodi, mut diso) = (vec![0.0; half], vec![0.0; half]);
        let mut tables = SquareTables {
            sosotb: &mut soso,
            disotb: &mut diso,
            soditb: &mut sodi,
            diditb: &mut didi,
        };
        let size: usize = coins.len() / 4;
        let (c1, rest) = coins.split_at(size);
        let (c2, rest) = rest.split_at(size);
        let (c3, c4) = rest.split_at(size);
        let grid = SquareGrid {
            ndimen,
            nbpntu,
            urootl: uroot.as_slice(),
            nbpntv,
            vrootl: vroot.as_slice(),
            iordru: s.ord_in_u,
            iordrv: s.ord_in_v,
        };
        let corners = CornerConstraints {
            contr1: c1,
            contr2: c2,
            contr3: c3,
            contr4: c4,
        };
        let isos_u = IsoTables {
            sotb1: &su0,
            sotb2: &su1,
            ditb1: &du0,
            ditb2: &du1,
        };
        let isos_v = IsoTables {
            sotb1: &sv0,
            sotb2: &sv1,
            ditb1: &dv0,
            ditb2: &dv1,
        };
        let mut iercod: i32 = 0;
        mma2cdi_(&grid, &corners, &isos_u, &isos_v, &mut tables, &mut iercod);
        let square = SampledSquare {
            ndimen,
            uintfn: [s.u0, s.u1],
            vintfn: [s.v0, s.v1],
            nbpntu,
            nbpntv,
            urootb: uroot.as_slice(),
            vrootb: vroot.as_slice(),
            isofav: conditions.favor_iso(),
        };
        let size: i32 = nbpntu.max(nbpntv);
        let (mut table, mut points) = (
            vec![0.0; size as usize],
            vec![0.0; (size * ndimen) as usize],
        );
        // mma2ds1_ clears the code first, so an mma2cdi_ failure is not seen, as natively.
        mma2ds1_(
            &square,
            func,
            &mut tables,
            &mut points,
            &mut table,
            &mut iercod,
        );
        self.store_discretisation(iercod, [soso, diso, sodi, didi]);
        Ok(())
    }

    fn store_discretisation(&self, iercod: i32, tables: [Vec<f64>; 4]) {
        let mut state = self.state.write();
        if iercod != 0 {
            state.disc_is_done = false;
            return;
        }
        let [soso, diso, sodi, didi] = tables.map(|values| super::arrays::one_based_from(&values));
        state.disc_is_done = true;
        state.soso_tab = Some(soso);
        state.diso_tab = Some(diso);
        state.sodi_tab = Some(sodi);
        state.didi_tab = Some(didi);
    }
}
