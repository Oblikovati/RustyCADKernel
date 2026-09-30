//! `AdvApp2Var_Patch::MakeApprox`: the Jacobi approximation of the discretised
//! patch, its canonical form with the boundary constraints added back, and the
//! reduction of its degrees.
use super::approx_f2var::{
    PatchApproxOutput, PatchApproxRequest, PatchGrid, SquareSamples, mma2can_, mma2ce1_, mma2fx6_,
};
use super::arrays::{one_based_from, required};
use super::context::AdvApp2VarContext;
use super::framework::AdvApp2VarFramework;
use super::math_base::mmfmca9_;
use super::patch::{AdvApp2VarPatch, PatchState, construction};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

const MAKE_APPROX: &str = "AdvApp2Var_Patch::MakeApprox";

/// `std::max` for ints.
fn at_least(a: i32, b: i32) -> i32 {
    a.max(b)
}

/// The minimum degree in one direction: `2*order + 1` at least 1, raised to the
/// degrees of the two neighbouring border isos (`NDMINU`, `NDMINV`).
fn minimum_degree(order: i32, iso_coeffs: [i32; 2]) -> i32 {
    let ndmin: i32 = at_least(1, 2 * order + 1);
    at_least(at_least(ndmin, iso_coeffs[0] - 1), iso_coeffs[1] - 1)
}

/// The value `(row, col)` of a tolerance table.
fn toler(table: &Array2<'static, f64>, row: i32, col: i32) -> Result<f64, Failure> {
    Ok(*table
        .value(row, col)
        .map_err(|e| e.into_failure(MAKE_APPROX))?)
}

/// The approximation tolerance `EPSAPR` and the frontier tolerances `EPSFRO`
/// `(NBSESP, 8)`: FToler columns 1..4, then CToler columns 1..4.
fn tolerances(
    conditions: &AdvApp2VarContext,
    nbsesp: i32,
) -> Result<(Vec<f64>, Vec<f64>), Failure> {
    let itoler = required(conditions.i_toler(), MAKE_APPROX)?;
    let ftoler = required(conditions.f_toler(), MAKE_APPROX)?;
    let ctoler = required(conditions.c_toler(), MAKE_APPROX)?;
    let n: usize = nbsesp.max(0) as usize;
    let (mut epsapr, mut epsfro) = (vec![0.; n], vec![0.; 8 * n]);
    for iesp in 1..=nbsesp {
        let i: usize = (iesp - 1) as usize;
        epsapr[i] = *itoler
            .value(iesp)
            .map_err(|e| e.into_failure(MAKE_APPROX))?;
        for col in 1..=4 {
            epsfro[i + (col as usize - 1) * n] = toler(ftoler, iesp, col)?;
            epsfro[i + (col as usize + 3) * n] = toler(ctoler, iesp, col)?;
        }
    }
    Ok((epsapr, epsfro))
}

/// The transposition of the canonical patch `(NCFLMU, NCFLMV, NDIMEN)` into the
/// C++ order `(NDIMEN, NCFLMV, NCFLMU)`, written over the head of `pataux`.
fn transpose_into(patcan: &[f64], pataux: &mut [f64], ndimen: i32, ncflmu: i32, ncflmv: i32) {
    for dim in 1..=ndimen {
        let a_in: i32 = (dim - 1) * ncflmu * ncflmv;
        for ii in 1..=ncflmu {
            let a_iu: i32 = (ii - 1) * ndimen * ncflmv;
            for jj in 1..=ncflmv {
                pataux[(dim + ndimen * (jj - 1) + a_iu - 1) as usize] =
                    patcan[(ii + ncflmu * (jj - 1) + a_in - 1) as usize];
            }
        }
    }
}

/// The sizes of the arrays of one `MakeApprox` call.
struct MakeSizes {
    ndimen: i32,
    nbsesp: i32,
    ncflmu: i32,
    ncflmv: i32,
    ndjacu: i32,
    ndjacv: i32,
}

impl AdvApp2VarPatch {
    /// Approximates the discretised patch by a Jacobi patch of the context degrees
    /// (`MakeApprox`): when no cutting is required the result is converted to the
    /// canonical base, the constraints of `constraints` and their errors are added
    /// back and the degrees are reduced; the coefficients are then stored in the
    /// C++ order `(NDIMEN, NCFLMV, NCFLMU)`. `num_dec` gives the allowed cuttings.
    /// Standard_ConstructionError when the canonical conversion fails.
    /// `patch.make_approx(&ctx, &frame, 5)?`.
    pub fn make_approx(
        &self,
        conditions: &AdvApp2VarContext,
        constraints: &AdvApp2VarFramework,
        num_dec: i32,
    ) -> Result<(), Failure> {
        let s: PatchState = self.state.read().clone();
        let sizes = MakeSizes {
            ndimen: conditions.total_dimension(),
            nbsesp: conditions.total_number_ssp(),
            ncflmu: conditions.u_limit(),
            ncflmv: conditions.v_limit(),
            ndjacu: conditions.u_jac_deg(),
            ndjacv: conditions.v_jac_deg(),
        };
        let uroots = required(conditions.u_roots(), MAKE_APPROX)?.length();
        let vroots = required(conditions.v_roots(), MAKE_APPROX)?.length();
        let nbpntu: i32 = uroots - if s.ord_in_u > -1 { 2 } else { 0 };
        let nbpntv: i32 = vroots - if s.ord_in_v > -1 { 2 } else { 0 };
        let iso_v = [
            constraints.iso_v(s.u0, s.u1, s.v0)?.nb_coeff(),
            constraints.iso_v(s.u0, s.u1, s.v1)?.nb_coeff(),
        ];
        let iso_u = [
            constraints.iso_u(s.u0, s.v0, s.v1)?.nb_coeff(),
            constraints.iso_u(s.u1, s.v0, s.v1)?.nb_coeff(),
        ];
        let (epsapr, epsfro) = tolerances(conditions, sizes.nbsesp)?;
        let jac_size: usize = ((1 + sizes.ndjacu) * (1 + sizes.ndjacv) * sizes.ndimen) as usize;
        let (mut patjac, n) = (vec![0.; jac_size], sizes.nbsesp.max(0) as usize);
        let (mut errmax, mut errmoy) = (vec![0.; n], vec![0.; n]);
        let request = PatchApproxRequest {
            numdec: num_dec,
            ndimen: sizes.ndimen,
            nbsesp: sizes.nbsesp,
            ndimse: &[3],
            ndminu: minimum_degree(s.ord_in_u, iso_v),
            ndminv: minimum_degree(s.ord_in_v, iso_u),
            ndguli: sizes.ncflmu - 1,
            ndgvli: sizes.ncflmv - 1,
            ndjacu: sizes.ndjacu,
            ndjacv: sizes.ndjacv,
            iordru: s.ord_in_u,
            iordrv: s.ord_in_v,
            nbpntu,
            nbpntv,
            epsapr: &epsapr,
        };
        let samples = SquareSamples {
            sosotb: required(s.soso_tab.as_ref(), MAKE_APPROX)?.as_slice(),
            disotb: required(s.diso_tab.as_ref(), MAKE_APPROX)?.as_slice(),
            soditb: required(s.sodi_tab.as_ref(), MAKE_APPROX)?.as_slice(),
            diditb: required(s.didi_tab.as_ref(), MAKE_APPROX)?.as_slice(),
        };
        let mut output = PatchApproxOutput {
            patjac: &mut patjac,
            errmax: &mut errmax,
            errmoy: &mut errmoy,
            ndegpu: 0,
            ndegpv: 0,
            itydec: 0,
        };
        let mut iercod: i32 = 0;
        mma2ce1_(&request, &samples, &mut output, &mut iercod);
        let (ncoefu, ncoefv, itydec) = (output.ndegpu, output.ndegpv, output.itydec);
        self.state.write().cut_sense = itydec;
        if itydec != 0 || iercod > 0 {
            let mut state = self.state.write();
            state.appr_is_done = false;
            state.has_result = false;
            return Ok(());
        }
        self.store_jacobi(iercod, [ncoefu + 1, ncoefv + 1], &errmax, &errmoy);
        let pataux: Vec<f64> = self.canonical(&sizes, patjac)?;
        self.add_constraints(conditions, constraints)?;
        self.add_errors(constraints)?;
        self.reduce_degrees(&sizes, [s.ord_in_u, s.ord_in_v], [&epsapr, &epsfro], pataux)
    }

    /// Records the result of `mma2ce1_`: status, coefficient counts and errors.
    fn store_jacobi(&self, iercod: i32, counts: [i32; 2], errmax: &[f64], errmoy: &[f64]) {
        let mut state = self.state.write();
        state.has_result = true;
        state.appr_is_done = iercod == 0;
        (state.nb_coeff_in_u, state.nb_coeff_in_v) = (counts[0], counts[1]);
        state.max_errors = Some(one_based_from(errmax));
        state.moy_errors = Some(one_based_from(errmoy));
    }

    /// Compacts the Jacobi patch in place (`mmfmca9_`) and converts it to the
    /// canonical base on [-1,1] (`mma2can_`), stored as the equation; returns the
    /// `PATAUX` scratch that becomes the final equation.
    fn canonical(&self, sizes: &MakeSizes, mut patjac: Vec<f64>) -> Result<Vec<f64>, Failure> {
        let s: PatchState = self.state.read().clone();
        let (njacu, njacv) = (sizes.ndjacu + 1, sizes.ndjacv + 1);
        // mmfmca9_ is called with PATJAC as both input and output: the copy keeps
        // the values the native in-place compaction reads, as checked by the probe.
        let tabini: Vec<f64> = patjac.clone();
        mmfmca9_(
            njacu,
            njacv,
            sizes.ndimen,
            s.nb_coeff_in_u,
            s.nb_coeff_in_v,
            sizes.ndimen,
            &tabini,
            &mut patjac,
        );
        let mut pataux: Vec<f64> = vec![0.; 2 * patjac.len()];
        let mut patcan: Vec<f64> = vec![0.; (sizes.ncflmu * sizes.ncflmv * sizes.ndimen) as usize];
        let mut iercod: i32 = 0;
        let (u, v) = (s.nb_coeff_in_u, s.nb_coeff_in_v);
        mma2can_(
            sizes.ncflmu,
            sizes.ncflmv,
            sizes.ndimen,
            s.ord_in_u,
            s.ord_in_v,
            u,
            v,
            &patjac,
            &mut pataux,
            &mut patcan,
            &mut iercod,
        );
        if iercod != 0 {
            return Err(construction(
                "AdvApp2Var_Patch::MakeApprox : Error in FORTRAN",
            ));
        }
        self.state.write().equation = Some(one_based_from(&patcan));
        Ok(pataux)
    }

    /// Lowers the coefficient counts while the dropped coefficients stay under the
    /// tolerances (`mma2fx6_`, with the errors updated by `AddErrors`), then stores
    /// the transposed equation in `pataux` as the result.
    fn reduce_degrees(
        &self,
        sizes: &MakeSizes,
        orders: [i32; 2],
        tolerances: [&[f64]; 2],
        mut pataux: Vec<f64>,
    ) -> Result<(), Failure> {
        let s: PatchState = self.state.read().clone();
        let patcan: Vec<f64> = required(s.equation.as_ref(), MAKE_APPROX)?
            .as_slice()
            .to_vec();
        let errmax: Vec<f64> = required(s.max_errors.as_ref(), MAKE_APPROX)?
            .as_slice()
            .to_vec();
        let grid = PatchGrid {
            ncfmxu: sizes.ncflmu,
            ncfmxv: sizes.ncflmv,
            ndimen: sizes.ndimen,
            nbsesp: sizes.nbsesp,
            ndimse: &[3],
            nbupat: 1,
            nbvpat: 1,
            iordru: orders[0],
            iordrv: orders[1],
            epsapr: tolerances[0],
            epsfro: tolerances[1],
            patcan: &patcan,
            errmax: &errmax,
        };
        let (mut ncoefu, mut ncoefv) = ([s.nb_coeff_in_u], [s.nb_coeff_in_v]);
        mma2fx6_(&grid, &mut ncoefu, &mut ncoefv);
        transpose_into(
            &patcan,
            &mut pataux,
            sizes.ndimen,
            sizes.ncflmu,
            sizes.ncflmv,
        );
        let mut state = self.state.write();
        (state.nb_coeff_in_u, state.nb_coeff_in_v) = (ncoefu[0], ncoefv[0]);
        state.equation = Some(one_based_from(&pataux));
        Ok(())
    }
}
