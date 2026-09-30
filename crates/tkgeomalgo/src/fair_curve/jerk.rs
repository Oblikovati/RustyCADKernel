// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_DistributionOfJerk.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_DistributionOfJerk`: the local jerk `H^3/12 * W^2` with
//! `W = ((C' ^ C''') - 3 (C' ^ C'') (C'.C'') / |C'|^2) / |C'|^2.5`, and its
//! derivatives in the poles.
use super::FairCurveBattenLaw;
use super::distribution::{FairCurveDistributionOfEnergy, LocalBasis, walk_pole_hessian};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmath::math::{FunctionSet, Vector};
use std::ops::{Deref, DerefMut};

/// The jerk density of a minimal-variation curve;
/// `FairCurveDistributionOfJerk::new(10, &flat, &poles, 1, &law, 0)`.
#[derive(Clone)]
pub struct FairCurveDistributionOfJerk {
    density: FairCurveDistributionOfEnergy,
    law: FairCurveBattenLaw,
}

/// The point quantities of the native `Value`.
struct JerkPoint {
    derivatives: [Xy; 3],
    norme_cprim: f64,
    inv: f64,
    inv2: f64,
    produit_c1c2: f64,
    derive_norme_cprim: f64,
    num_rho: f64,
    numerateur: f64,
    denominateur: f64,
    mesure: f64,
    wval: f64,
}

/// The gradient pieces, 1-based like the native vectors.
struct JerkGradient {
    wgrad: Vector<'static>,
    num_grad: Vector<'static>,
    grad_norme_cprim: Vector<'static>,
    ngdnc_prim: Vector<'static>,
    grad_derive_norme_cprim: Vector<'static>,
    grad_num_rho: Vector<'static>,
    numdu_grad: Vector<'static>,
}

impl FairCurveDistributionOfJerk {
    /// The density of `derivative_order` over the B-spline of order `bspl_order`
    /// (it keeps a copy of `law`); `nb_val_aux` is 1 for a free sliding.
    pub fn new(
        bspl_order: i32,
        flat_knots: &Handle<HArray1<f64>>,
        poles: &Handle<HArray1<Point2>>,
        derivative_order: i32,
        law: &FairCurveBattenLaw,
        nb_val_aux: i32,
    ) -> Self {
        let density = FairCurveDistributionOfEnergy::new(
            bspl_order,
            flat_knots,
            poles,
            derivative_order,
            nb_val_aux,
        );
        Self { density, law: *law }
    }
    /// `Value(TParam, Jerk)`: the density, then its derivatives, at `t`.
    fn evaluate(&self, t: f64, f: &mut Vector<'_>) -> bool {
        f.init(0.0);
        let Some(basis) = self.density.local_basis(3, t) else {
            return false;
        };
        let Some(point) = self.point(&basis, t) else {
            return false;
        };
        let lower: i32 = f.lower();
        f[lower] = point.mesure * (point.wval * point.wval);
        if self.density.derivative_order >= 1 {
            self.derivatives(&basis, &point, f);
        }
        true
    }
    /// The point quantities, or None when the height is not positive.
    fn point(&self, basis: &LocalBasis, t: f64) -> Option<JerkPoint> {
        let [cprim, csecn, ctroi] = [0, 1, 2].map(|k| basis.derivatives[k]);
        let norme_cprim: f64 = cprim.modulus();
        let inv: f64 = 1.0 / norme_cprim;
        let inv2: f64 = inv * inv;
        let produit_c1c2: f64 = cprim * csecn;
        let derive_norme_cprim: f64 = produit_c1c2 * inv2;
        let num_rho: f64 = cprim ^ csecn;
        let numerateur: f64 = (cprim ^ ctroi) - 3.0 * num_rho * derive_norme_cprim;
        let denominateur: f64 = norme_cprim.powf(2.5);
        let (positive, hauteur) = self.law.height(t);
        if !positive {
            return None;
        }
        let mesure: f64 = hauteur.powf(3.0) / 12.0;
        let wval: f64 = numerateur / denominateur;
        Some(JerkPoint {
            derivatives: [cprim, csecn, ctroi],
            norme_cprim,
            inv,
            inv2,
            produit_c1c2,
            derive_norme_cprim,
            num_rho,
            numerateur,
            denominateur,
            mesure,
            wval,
        })
    }
    fn derivatives(&self, basis: &LocalBasis, p: &JerkPoint, f: &mut Vector<'_>) {
        let order: i32 = self.density.bspl_order;
        let g = gradient(basis, p, order, self.density.nb_val_aux);
        let facteur: f64 = 2.0 * p.mesure * p.wval;
        let first: i32 = f.lower() + basis.first_non_zero();
        for (kk, jj) in (first..).zip(1..=2 * order) {
            f[kk] = facteur * g.wgrad[jj];
        }
        let last_gradient_index: i32 = self.density.last_gradient_index(f.lower());
        if self.density.nb_val_aux == 1 {
            f[last_gradient_index] = 0.0;
        }
        if self.density.derivative_order >= 2 {
            let hessian = JerkHessian::new(basis, p, &g);
            let frame = [basis.first_non_zero(), last_gradient_index, order];
            walk_pole_hessian(
                f,
                frame,
                |ii, jj| hessian.pair(ii, jj),
                |ii| hessian.diagonal(ii),
            );
        }
    }
}

/// The gradient vectors; entry `2i - 1` derives in the X of pole i, `2i` in its Y.
fn gradient(basis: &LocalBasis, p: &JerkPoint, order: i32, nb_val_aux: i32) -> JerkGradient {
    let zero = |size: i32| Vector::filled(1, size, 0.0).expect("gradient size");
    let mut g = JerkGradient {
        wgrad: zero(2 * order + nb_val_aux),
        num_grad: zero(2 * order),
        grad_norme_cprim: zero(2 * order),
        ngdnc_prim: zero(2 * order),
        grad_derive_norme_cprim: zero(2 * order),
        grad_num_rho: zero(2 * order),
        numdu_grad: zero(2 * order),
    };
    let [cprim, csecn, ctroi] = p.derivatives;
    let inv_denominateur: f64 = 1.0 / p.denominateur;
    let aux: f64 = 2.5 * p.numerateur * p.inv;
    let aux_bis: f64 = 2.0 * p.produit_c1c2 * p.inv;
    for ii in 1..=order {
        let [b2, b3, b4] = [2, 3, 4].map(|row| basis.at(row, ii));
        // (prim, secn) of the derived coordinate, then the rho and numerator terms.
        let x_parts = (
            (cprim.x(), csecn.x()),
            ysecn_term(csecn.y(), cprim.y(), b2, b3),
            ysecn_term(ctroi.y(), cprim.y(), b2, b4),
        );
        let y_parts = (
            (cprim.y(), csecn.y()),
            -csecn.x() * b2 + cprim.x() * b3,
            -ctroi.x() * b2 + cprim.x() * b4,
        );
        for (offset, ((prim, secn), rho, ngrad1)) in [(0, x_parts), (1, y_parts)] {
            let jj: i32 = 2 * ii - 1 + offset;
            g.grad_norme_cprim[jj] = prim * b2 * p.inv;
            g.ngdnc_prim[jj] = (prim * b3 + secn * b2) - aux_bis * g.grad_norme_cprim[jj];
            g.grad_derive_norme_cprim[jj] = g.ngdnc_prim[jj] * p.inv2;
            g.grad_num_rho[jj] = rho;
            let ngrad2: f64 = -3.0
                * (p.num_rho * g.grad_derive_norme_cprim[jj]
                    + g.grad_num_rho[jj] * p.derive_norme_cprim);
            g.num_grad[jj] = ngrad1 + ngrad2;
            g.numdu_grad[jj] = g.num_grad[jj] - aux * g.grad_norme_cprim[jj];
            g.wgrad[jj] = inv_denominateur * g.numdu_grad[jj];
        }
    }
    g
}

/// `a * b2 - c * b3`: the X-derivative forms `YSecn*B2 - YPrim*B3` and `YTroi*B2 - YPrim*B4`.
fn ysecn_term(a: f64, c: f64, b2: f64, b3: f64) -> f64 {
    a * b2 - c * b3
}

/// The Hessian terms of a point; the parity of an index gives its coordinate
/// (odd: X, even: Y).
struct JerkHessian<'a> {
    basis: &'a LocalBasis,
    p: &'a JerkPoint,
    g: &'a JerkGradient,
    facteurs: [f64; 3],
    facteur_w: f64,
    inv_denominateur: f64,
    facteur: f64,
    aux_bis: f64,
}

/// The basis products of a pair of poles.
struct PairProducts {
    produit: f64,
    produit2: f64,
    produit_v: f64,
    hnum_rho: f64,
}

impl<'a> JerkHessian<'a> {
    fn new(basis: &'a LocalBasis, p: &'a JerkPoint, g: &'a JerkGradient) -> Self {
        let [cprim, _, _] = p.derivatives;
        let (x, y) = (cprim.x() * p.inv, cprim.y() * p.inv);
        Self {
            basis,
            p,
            g,
            // FacteurX, FacteurY, FacteurXY.
            facteurs: [(1.0 - x * x) * p.inv, (1.0 - y * y) * p.inv, -x * y * p.inv],
            facteur_w: p.wval * p.inv,
            inv_denominateur: 1.0 / p.denominateur,
            facteur: 2.0 * p.mesure,
            aux_bis: 2.0 * p.produit_c1c2 * p.inv,
        }
    }
    fn is_x(index: i32) -> bool {
        index % 2 == 1
    }
    /// XPrim or YPrim, XSecn or YSecn of the coordinate of `index`.
    fn prim_secn(&self, index: i32) -> (f64, f64) {
        let [cprim, csecn, _] = self.p.derivatives;
        if Self::is_x(index) {
            (cprim.x(), csecn.x())
        } else {
            (cprim.y(), csecn.y())
        }
    }
    /// `(Aux1, DeriveAuxBis)` of the column `b` of pole `big_j`.
    fn aux1_derive(&self, b: i32, big_j: i32) -> (f64, f64) {
        let (prim, secn) = self.prim_secn(b);
        let aux1: f64 = self.p.inv2 * self.g.grad_norme_cprim[b];
        let (b2, b3) = (self.basis.at(2, big_j), self.basis.at(3, big_j));
        let derive: f64 = 2.0 * ((prim * b3 + secn * b2) * self.p.inv - self.p.produit_c1c2 * aux1);
        (aux1, derive)
    }
    /// `HDeriveNormeCPrim` of the entry (a, b) of poles (big_i, big_j).
    fn h_derive(&self, (a, b): (i32, i32), (big_i, big_j): (i32, i32), pp: &PairProducts) -> f64 {
        let (aux1, derive) = self.aux1_derive(b, big_j);
        let prim_a: f64 = self.prim_secn(a).0;
        let (inv, inv2) = (self.p.inv, self.p.inv2);
        let gn_a: f64 = self.g.grad_norme_cprim[a];
        let b2i: f64 = self.basis.at(2, big_i);
        let head: f64 = if Self::is_x(a) == Self::is_x(b) {
            pp.produit2 - derive * gn_a - self.aux_bis * (pp.produit * inv - prim_a * b2i * aux1)
        } else {
            -derive * gn_a + self.aux_bis * prim_a * b2i * aux1
        };
        head * inv2 - 2.0 * self.g.ngdnc_prim[a] * aux1 * inv
    }
    /// `GradNumRho(a)*GradDerive(b) + NumRho*HDerive + GradNumRho(b)*GradDerive(a)`.
    fn rho_sum(&self, a: i32, b: i32, h_derive: f64) -> f64 {
        let g = self.g;
        g.grad_num_rho[a] * g.grad_derive_norme_cprim[b]
            + self.p.num_rho * h_derive
            + g.grad_num_rho[b] * g.grad_derive_norme_cprim[a]
    }
    /// `NormeCPrim*NSeconde + NumGrad(a)*GradNorm(b) - 2.5*(NumGrad(b)*GradNorm(a) + D*Num)`.
    fn aux_pair(&self, (a, b): (i32, i32), nseconde: f64, dseconde: f64) -> f64 {
        let g = self.g;
        self.p.norme_cprim * nseconde + g.num_grad[a] * g.grad_norme_cprim[b]
            - 2.5 * (g.num_grad[b] * g.grad_norme_cprim[a] + dseconde * self.p.numerateur)
    }
    /// `NSeconde*NormeCPrim - 1.5*NumGrad(a)*GradNorm(a) - 2.5*D*Num`.
    fn aux_own(&self, a: i32, nseconde: f64, dseconde: f64) -> f64 {
        let g = self.g;
        nseconde * self.p.norme_cprim
            - 1.5 * g.num_grad[a] * g.grad_norme_cprim[a]
            - 2.5 * dseconde * self.p.numerateur
    }
    /// `Facteur * (WGrad(a)*WGrad(b) + FacteurW * VIntermed)`.
    fn entry(&self, aux: f64, (a, b): (i32, i32)) -> f64 {
        let g = self.g;
        let vintermed: f64 =
            self.inv_denominateur * (aux - 3.5 * g.grad_norme_cprim[b] * g.numdu_grad[a]);
        self.facteur * (g.wgrad[a] * g.wgrad[b] + self.facteur_w * vintermed)
    }
    /// The factor of `Produit` in `DSeconde` for the coordinates of (a, b).
    fn dseconde_factor(&self, a: i32, b: i32) -> f64 {
        match (Self::is_x(a), Self::is_x(b)) {
            (true, true) => self.facteurs[0],
            (false, false) => self.facteurs[1],
            _ => self.facteurs[2],
        }
    }
    /// The XiXj, XiYj, YiXj and YiYj entries of the even indices `jj < ii`.
    fn pair(&self, ii: i32, jj: i32) -> [f64; 4] {
        let (big_i, big_j) = (ii / 2, jj / 2);
        let b = |row: i32, k: i32| self.basis.at(row, k);
        let pp = PairProducts {
            produit: b(2, big_i) * b(2, big_j),
            produit2: b(2, big_i) * b(3, big_j) + b(3, big_i) * b(2, big_j),
            produit_v: b(2, big_i) * b(4, big_j) - b(4, big_i) * b(2, big_j),
            hnum_rho: b(2, big_i) * b(3, big_j) - b(3, big_i) * b(2, big_j),
        };
        [(ii - 1, jj - 1), (ii - 1, jj), (ii, jj - 1), (ii, jj)]
            .map(|ab| self.pair_entry(ab, (big_i, big_j), &pp))
    }
    fn pair_entry(&self, (a, b): (i32, i32), poles: (i32, i32), pp: &PairProducts) -> f64 {
        let h_derive: f64 = self.h_derive((a, b), poles, pp);
        let sum: f64 = self.rho_sum(a, b, h_derive);
        let hd: f64 = pp.hnum_rho * self.p.derive_norme_cprim;
        let nseconde: f64 = match (Self::is_x(a), Self::is_x(b)) {
            (true, false) => pp.produit_v - 3.0 * (sum + hd),
            (false, true) => -pp.produit_v - 3.0 * (sum - hd),
            _ => -3.0 * sum,
        };
        let dseconde: f64 = self.dseconde_factor(a, b) * pp.produit;
        self.entry(self.aux_pair((a, b), nseconde, dseconde), (a, b))
    }
    /// The XiXi, XiYi and YiYi entries of the even index `ii`.
    fn diagonal(&self, ii: i32) -> [f64; 3] {
        let big_i: i32 = ii / 2;
        let b2: f64 = self.basis.at(2, big_i);
        let pp = PairProducts {
            produit: b2 * b2,
            produit2: 2.0 * b2 * self.basis.at(3, big_i),
            produit_v: 0.0,
            hnum_rho: 0.0,
        };
        let poles = (big_i, big_i);
        let own = |a: i32| {
            let h_derive: f64 = self.h_derive((a, a), poles, &pp);
            let rho: f64 = 2.0 * self.g.grad_num_rho[a] * self.g.grad_derive_norme_cprim[a];
            let nseconde: f64 = -3.0 * (rho + self.p.num_rho * h_derive);
            let dseconde: f64 = self.dseconde_factor(a, a) * pp.produit;
            self.entry(self.aux_own(a, nseconde, dseconde), (a, a))
        };
        let h_derive: f64 = self.h_derive((ii - 1, ii), poles, &pp);
        let nseconde: f64 = -3.0 * self.rho_sum(ii - 1, ii, h_derive);
        let dseconde: f64 = self.facteurs[2] * pp.produit;
        let xy: f64 = self.entry(
            self.aux_pair((ii, ii - 1), nseconde, dseconde),
            (ii, ii - 1),
        );
        [own(ii - 1), xy, own(ii)]
    }
}

impl Deref for FairCurveDistributionOfJerk {
    type Target = FairCurveDistributionOfEnergy;
    fn deref(&self) -> &FairCurveDistributionOfEnergy {
        &self.density
    }
}

impl DerefMut for FairCurveDistributionOfJerk {
    fn deref_mut(&mut self) -> &mut FairCurveDistributionOfEnergy {
        &mut self.density
    }
}

impl FunctionSet for FairCurveDistributionOfJerk {
    fn variables(&self) -> i32 {
        self.density.nb_variables()
    }
    fn equations(&self) -> i32 {
        self.density.nb_equations()
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        self.evaluate(point[point.lower()], residual)
    }
}
