// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_DistributionOfSagging.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_DistributionOfSagging`: the local flexion `H^3/12 * W^2` with
//! `W = (C' ^ C'') / |C'|^2.5`, and its derivatives in the poles.
use super::FairCurveBattenLaw;
use super::distribution::{FairCurveDistributionOfEnergy, LocalBasis, walk_pole_hessian};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmath::math::{FunctionSet, Vector};
use std::ops::{Deref, DerefMut};

/// The flexion density of a batten;
/// `FairCurveDistributionOfSagging::new(10, &flat, &poles, 1, &law, 0)`.
#[derive(Clone)]
pub struct FairCurveDistributionOfSagging {
    density: FairCurveDistributionOfEnergy,
    law: FairCurveBattenLaw,
}

/// The point quantities the gradient and Hessian terms share.
struct SaggingPoint {
    cprim: Xy,
    csecn: Xy,
    norme_cprim: f64,
    inv_norme_cprim: f64,
    numerateur: f64,
    denominateur: f64,
    mesure: f64,
    wval: f64,
}

/// The gradient pieces of the flexion, 1-based like the native vectors.
struct SaggingGradient {
    wgrad: Vector<'static>,
    num_grad: Vector<'static>,
    grad_norme_cprim: Vector<'static>,
    numdu_grad: Vector<'static>,
}

impl FairCurveDistributionOfSagging {
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
    /// `Value(TParam, Flexion)`: the density, then its derivatives, at `t`.
    fn evaluate(&self, t: f64, f: &mut Vector<'_>) -> bool {
        f.init(0.0);
        let Some(basis) = self.density.local_basis(2, t) else {
            return false;
        };
        let (cprim, csecn): (Xy, Xy) = (basis.derivatives[0], basis.derivatives[1]);
        let norme_cprim: f64 = cprim.modulus();
        let inv_norme_cprim: f64 = 1.0 / norme_cprim;
        let numerateur: f64 = cprim ^ csecn;
        let denominateur: f64 = norme_cprim.powf(2.5);
        let (positive, hauteur) = self.law.height(t);
        if !positive {
            return false;
        }
        let mesure: f64 = hauteur.powf(3.0) / 12.0;
        let wval: f64 = numerateur / denominateur;
        let lower: i32 = f.lower();
        f[lower] = mesure * (wval * wval);
        if self.density.derivative_order >= 1 {
            let point = SaggingPoint {
                cprim,
                csecn,
                norme_cprim,
                inv_norme_cprim,
                numerateur,
                denominateur,
                mesure,
                wval,
            };
            self.derivatives(&basis, &point, f);
        }
        true
    }
    fn derivatives(&self, basis: &LocalBasis, p: &SaggingPoint, f: &mut Vector<'_>) {
        let order: i32 = self.density.bspl_order;
        let size: i32 = 2 * order + self.density.nb_val_aux;
        let zero = || Vector::filled(1, size, 0.0).expect("gradient size");
        let mut g = SaggingGradient {
            wgrad: zero(),
            num_grad: zero(),
            grad_norme_cprim: zero(),
            numdu_grad: zero(),
        };
        let inv_denominateur: f64 = 1.0 / p.denominateur;
        let facteur: f64 = 2.0 * p.mesure * p.wval;
        let aux: f64 = 2.5 * p.numerateur * p.inv_norme_cprim;
        let (xprim, yprim) = (p.cprim.x(), p.cprim.y());
        let (xsecn, ysecn) = (p.csecn.x(), p.csecn.y());
        let mut kk: i32 = f.lower() + basis.first_non_zero();
        for ii in 1..=order {
            let (b2, b3) = (basis.at(2, ii), basis.at(3, ii));
            let x_parts = (ysecn * b2 - yprim * b3, xprim * b2 * p.inv_norme_cprim);
            let y_parts = (-xsecn * b2 + xprim * b3, yprim * b2 * p.inv_norme_cprim);
            // Offset 0 derives in X, offset 1 in Y.
            for (offset, (num, norm)) in [(0, x_parts), (1, y_parts)] {
                let jj: i32 = 2 * ii - 1 + offset;
                g.num_grad[jj] = num;
                g.grad_norme_cprim[jj] = norm;
                g.numdu_grad[jj] = g.num_grad[jj] - aux * g.grad_norme_cprim[jj];
                g.wgrad[jj] = inv_denominateur * g.numdu_grad[jj];
                f[kk + offset] = facteur * g.wgrad[jj];
            }
            kk += 2;
        }
        if self.density.nb_val_aux == 1 {
            let upper: i32 = g.wgrad.upper();
            g.wgrad[upper] = 0.0;
        }
        if self.density.derivative_order >= 2 {
            let last_gradient_index: i32 = self.density.last_gradient_index(f.lower());
            let frame = [basis.first_non_zero(), last_gradient_index, order];
            hessian(basis, p, &g, frame, f);
        }
    }
}

/// The pole block of the flexion Hessian.
fn hessian(
    basis: &LocalBasis,
    p: &SaggingPoint,
    g: &SaggingGradient,
    frame: [i32; 3],
    f: &mut Vector<'_>,
) {
    let inv: f64 = p.inv_norme_cprim;
    let (xprim, yprim) = (p.cprim.x(), p.cprim.y());
    let facteur_x: f64 = (1.0 - (xprim * inv) * (xprim * inv)) * inv;
    let facteur_y: f64 = (1.0 - (yprim * inv) * (yprim * inv)) * inv;
    let facteur_xy: f64 = -(xprim * inv) * (yprim * inv) * inv;
    let facteur_w: f64 = p.wval * inv;
    let inv_denominateur: f64 = 1.0 / p.denominateur;
    let facteur: f64 = 2.0 * p.mesure;
    // One Hessian entry from the native `Aux` numerator of the (a, b) pair.
    let entry = |aux: f64, a: i32, b: i32| {
        let vintermed: f64 =
            inv_denominateur * (aux - 3.5 * g.grad_norme_cprim[b] * g.numdu_grad[a]);
        facteur * (g.wgrad[a] * g.wgrad[b] + facteur_w * vintermed)
    };
    // `NumGrad(a) * GradNorm(b) - 2.5 * (NumGrad(b) * GradNorm(a) + DSeconde * Num)`.
    let cross = |a: i32, b: i32, dseconde: f64| {
        g.num_grad[a] * g.grad_norme_cprim[b]
            - 2.5 * (g.num_grad[b] * g.grad_norme_cprim[a] + dseconde * p.numerateur)
    };
    walk_pole_hessian(
        f,
        frame,
        |ii, jj| {
            let (big_i, big_j) = (ii / 2, jj / 2);
            let produit: f64 = basis.at(2, big_i) * basis.at(2, big_j);
            let nseconde: f64 =
                basis.at(2, big_i) * basis.at(3, big_j) - basis.at(3, big_i) * basis.at(2, big_j);
            let dxy: f64 = facteur_xy * produit;
            let xy_aux: f64 = p.norme_cprim * nseconde
                + g.num_grad[ii - 1] * g.grad_norme_cprim[jj]
                - 2.5 * (g.num_grad[jj] * g.grad_norme_cprim[ii - 1] + dxy * p.numerateur);
            let yx_aux: f64 = -p.norme_cprim * nseconde
                + g.num_grad[ii] * g.grad_norme_cprim[jj - 1]
                - 2.5 * (g.num_grad[jj - 1] * g.grad_norme_cprim[ii] + dxy * p.numerateur);
            [
                entry(cross(ii - 1, jj - 1, facteur_x * produit), ii - 1, jj - 1),
                entry(xy_aux, ii - 1, jj),
                entry(yx_aux, ii, jj - 1),
                entry(cross(ii, jj, facteur_y * produit), ii, jj),
            ]
        },
        |ii| {
            let b: f64 = basis.at(2, ii / 2);
            let produit: f64 = b * b;
            let own = |a: i32, dseconde: f64| {
                -1.5 * g.num_grad[a] * g.grad_norme_cprim[a] - 2.5 * dseconde * p.numerateur
            };
            [
                entry(own(ii - 1, facteur_x * produit), ii - 1, ii - 1),
                entry(cross(ii - 1, ii, facteur_xy * produit), ii - 1, ii),
                entry(own(ii, facteur_y * produit), ii, ii),
            ]
        },
    );
}

impl Deref for FairCurveDistributionOfSagging {
    type Target = FairCurveDistributionOfEnergy;
    fn deref(&self) -> &FairCurveDistributionOfEnergy {
        &self.density
    }
}

impl DerefMut for FairCurveDistributionOfSagging {
    fn deref_mut(&mut self) -> &mut FairCurveDistributionOfEnergy {
        &mut self.density
    }
}

impl FunctionSet for FairCurveDistributionOfSagging {
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
