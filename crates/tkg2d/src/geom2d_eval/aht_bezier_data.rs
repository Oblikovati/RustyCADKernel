// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_AHTBezierCurve.cxx and ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierCurve.cxx (the members, constructors and EvalD0..EvalDN that GeomEval repeats with gp_XYZ) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fields, constructor checks and evaluation of `Geom2dEval_AHTBezierCurve` and
//! `GeomEval_AHTBezierCurve`, generic over the pole type; the classes convert the
//! returned coordinates into their points and vectors.
use super::bezier_curve_data::{check_weights, owned};
use super::eval_basis::EvalPole;
use super::eval_basis::aht::{
    AhtView, basis_dimension, mixed, polynomial, quotient_d1, quotient_d2, quotient_d3,
};
use super::eval_basis::aht_dn;
use super::eval_basis::sums::BasisSums;
use crate::geom2d::{dump_field, dump_flag, errors};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use std::io;

/// The members of an AHT-Bezier curve (`myPoles`, `myWeights`, `myAlgDegree`,
/// `myAlpha`, `myBeta`, `myRational`);
/// `AhtBezierData::checked("GeomEval_AHTBezierCurve", &poles, None, 2, 0.0, 0.0)?`.
#[derive(Clone)]
pub struct AhtBezierData<P: EvalPole + 'static> {
    /// The poles with their original bounds.
    pub poles: Array1<'static, P>,
    /// The weights; one (natively uninitialized, here 0.0) element when non-rational.
    pub weights: Array1<'static, f64>,
    /// The algebraic polynomial degree.
    pub alg_degree: i32,
    /// The hyperbolic frequency, 0 without hyperbolic terms.
    pub alpha: f64,
    /// The trigonometric frequency, 0 without trigonometric terms.
    pub beta: f64,
    /// Whether the curve was built with weights.
    pub rational: bool,
}

// The checks both constructors share, in native order.
fn check_basis(
    class: &str,
    count: usize,
    alg_degree: i32,
    [alpha, beta]: [f64; 2],
) -> Result<(), Failure> {
    let negative = |name: &str, value: f64| {
        errors::construction(&format!("{class}: {name} = {value}; expected >= 0"))
    };
    if alg_degree < 0 {
        return Err(errors::construction(&format!(
            "{class}: algebraic degree = {alg_degree}; expected >= 0"
        )));
    }
    if alpha < 0.0 {
        return Err(negative("alpha", alpha));
    }
    if beta < 0.0 {
        return Err(negative("beta", beta));
    }
    let dimension: i32 = basis_dimension(alg_degree, alpha, beta);
    if count as i64 != i64::from(dimension) {
        return Err(errors::construction(&format!(
            "{class}: {count} poles; expected basisDimension = {dimension}"
        )));
    }
    Ok(())
}

impl<P: EvalPole + Clone + Default + 'static> AhtBezierData<P> {
    /// The two native constructors: the basis checks, then the weight checks when
    /// `weights` is given; Standard_ConstructionError naming `class`;
    /// `AhtBezierData::checked(CLASS, &poles, Some(&w), 1, 1.0, 1.0)?`.
    pub fn checked(
        class: &str,
        poles: &Array1<'_, P>,
        weights: Option<&Array1<'_, f64>>,
        alg_degree: i32,
        [alpha, beta]: [f64; 2],
    ) -> Result<Self, Failure> {
        check_basis(class, poles.len(), alg_degree, [alpha, beta])?;
        let stored: Array1<'static, f64> = match weights {
            Some(weights) => {
                check_weights(class, weights, poles.len())?;
                owned(weights)
            }
            None => Array1::with_bounds(1, 1).map_err(errors::array_bounds)?,
        };
        Ok(Self {
            poles: owned(poles),
            weights: stored,
            alg_degree,
            alpha,
            beta,
            rational: weights.is_some(),
        })
    }
    fn view(&self) -> AhtView<'_, P> {
        AhtView {
            poles: self.poles.as_slice(),
            weights: self.rational.then(|| self.weights.as_slice()),
            alg_degree: self.alg_degree as usize,
            alpha: self.alpha,
            beta: self.beta,
        }
    }
    // The purely algebraic basis uses Horner's scheme; any pair uses the direct sums.
    fn sums<const MAX: usize>(&self, u: f64) -> BasisSums<P::Coord> {
        if self.alpha <= 0.0 && self.beta <= 0.0 {
            return polynomial::<MAX, _>(&self.view(), u);
        }
        mixed::<MAX, _>(&self.view(), u)
    }
    /// `EvalD0` coordinates; `data.eval_d0(0.5)`.
    pub fn eval_d0(&self, u: f64) -> P::Coord {
        let sums: BasisSums<P::Coord> = self.sums::<0>(u);
        if !self.rational {
            return sums.n[0];
        }
        sums.n[0] / sums.w[0]
    }
    /// `EvalD1` coordinates; `let (c, d1) = data.eval_d1(0.5);`.
    pub fn eval_d1(&self, u: f64) -> (P::Coord, P::Coord) {
        let sums: BasisSums<P::Coord> = self.sums::<1>(u);
        let [c, d1, _, _] = sums.n;
        if self.rational {
            return quotient_d1(&sums);
        }
        (c, d1)
    }
    /// `EvalD2` coordinates; `let [c, d1, d2] = data.eval_d2(0.5);`.
    pub fn eval_d2(&self, u: f64) -> [P::Coord; 3] {
        let sums: BasisSums<P::Coord> = self.sums::<2>(u);
        let [mut c, mut d1, mut d2, _] = sums.n;
        if self.rational {
            (c, d1) = quotient_d1(&sums);
            d2 = quotient_d2(&sums, c, d1);
        }
        [c, d1, d2]
    }
    /// `EvalD3` coordinates; `let [c, d1, d2, d3] = data.eval_d3(0.5);`.
    pub fn eval_d3(&self, u: f64) -> [P::Coord; 4] {
        let sums: BasisSums<P::Coord> = self.sums::<3>(u);
        if self.rational {
            return quotient_d3(&sums);
        }
        sums.n
    }
    /// The `EvalDN` branch for `n > 3`, through the basis derivatives;
    /// `data.eval_high_dn(0.5, 4)`.
    pub fn eval_high_dn(&self, u: f64, n: i32) -> P::Coord {
        let view: AhtView<'_, P> = self.view();
        match view.weights {
            None => aht_dn::non_rational_dn(&view, u, n),
            Some(weights) => aht_dn::rational_dn(&view, weights, u, n),
        }
    }
    /// The `DumpJson` fields after the bounded-curve base; each pole is dumped under
    /// the key "Pole" (`DumpFieldToName("&aPole")`) by `dump_pole`;
    /// `data.dump_fields(&mut stream, 1, |pole, s| pole.dump_json(s))?`.
    pub fn dump_fields(
        &self,
        stream: &mut DumpStream,
        depth: i32,
        dump_pole: impl Fn(&P, &mut DumpStream) -> io::Result<()>,
    ) -> io::Result<()> {
        dump::dump_integer(stream, "AlgDegree", self.alg_degree)?;
        dump::dump_real(stream, "Alpha", self.alpha)?;
        dump::dump_real(stream, "Beta", self.beta)?;
        dump_flag(stream, "Rational", self.rational)?;
        for pole in self.poles.as_slice() {
            dump_field(stream, depth, "Pole", |nested, _| dump_pole(pole, nested))?;
        }
        Ok(())
    }
}

/// The public read accessors of the AHT-Bezier curve classes over their
/// [`AhtBezierData`] field `data`; `rustycad_tkg2d::aht_bezier_accessors!(Point3);`.
#[macro_export]
macro_rules! aht_bezier_accessors {
    ($point:ty) => {
        /// A copy of the poles with their original bounds (`Poles`); `curve.poles().len()`.
        pub fn poles(&self) -> ::rustycad_tkernel::ncollection::Array1<'static, $point> {
            self.data.poles.clone()
        }
        /// A copy of the weights (`Weights`); one element for a non-rational curve, whose
        /// native value is uninitialized; `curve.weights().len()`.
        pub fn weights(&self) -> ::rustycad_tkernel::ncollection::Array1<'static, f64> {
            self.data.weights.clone()
        }
        /// The algebraic polynomial degree (`AlgDegree`); `curve.alg_degree()`.
        pub fn alg_degree(&self) -> i32 {
            self.data.alg_degree
        }
        /// The hyperbolic frequency, 0 without hyperbolic terms (`Alpha`); `curve.alpha()`.
        pub fn alpha(&self) -> f64 {
            self.data.alpha
        }
        /// The trigonometric frequency, 0 without trigonometric terms (`Beta`); `curve.beta()`.
        pub fn beta(&self) -> f64 {
            self.data.beta
        }
        /// The number of poles (`NbPoles`); `curve.nb_poles()`.
        pub fn nb_poles(&self) -> i32 {
            self.data.poles.len() as i32
        }
        /// Whether the curve was built with weights (`IsRational`); `curve.is_rational()`.
        pub fn is_rational(&self) -> bool {
            self.data.rational
        }
    };
}

/// `EvalD0`..`EvalDN` of the AHT-Bezier curve classes over their [`AhtBezierData`]
/// field `data`; orders 1 to 3 of `EvalDN` delegate to the explicit evaluators, as
/// natively; `check_derivative_order` must be in scope;
/// `rustycad_tkg2d::aht_bezier_evaluators!(Point3, Vector3, GeomCurveD1, GeomCurveD2, GeomCurveD3, "GeomEval_AHTBezierCurve");`.
#[macro_export]
macro_rules! aht_bezier_evaluators {
    ($point:ident, $vector:ident, $d1:ident, $d2:ident, $d3:ident, $class:literal) => {
        fn eval_d0(&self, u: f64) -> Result<$point, Failure> {
            Ok($point::from(self.data.eval_d0(u)))
        }
        fn eval_d1(&self, u: f64) -> Result<$d1, Failure> {
            let (c, d1) = self.data.eval_d1(u);
            Ok($d1 {
                point: $point::from(c),
                d1: $vector::from(d1),
            })
        }
        fn eval_d2(&self, u: f64) -> Result<$d2, Failure> {
            let [c, d1, d2] = self.data.eval_d2(u);
            let [d1, d2] = [d1, d2].map($vector::from);
            Ok($d2 {
                point: $point::from(c),
                d1,
                d2,
            })
        }
        fn eval_d3(&self, u: f64) -> Result<$d3, Failure> {
            let [c, d1, d2, d3] = self.data.eval_d3(u);
            let [d1, d2, d3] = [d1, d2, d3].map($vector::from);
            Ok($d3 {
                point: $point::from(c),
                d1,
                d2,
                d3,
            })
        }
        fn eval_dn(&self, u: f64, n: i32) -> Result<$vector, Failure> {
            check_derivative_order(concat!($class, "::EvalDN"), n)?;
            match n {
                1 => return Ok(self.eval_d1(u)?.d1),
                2 => return Ok(self.eval_d2(u)?.d2),
                3 => return Ok(self.eval_d3(u)?.d3),
                _ => {}
            }
            Ok($vector::from(self.data.eval_high_dn(u, n)))
        }
    };
}
