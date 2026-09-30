// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Energy.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_Energy`: the energy of a fair curve as a function of its free
//! variables (poles, lengths of the constrained end tangents and curvatures, and
//! the free sliding), with the conversions between variables and poles. The
//! gradient and Hessian reductions live in `energy_terms`.
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point2, Vector2, Xy};
use rustycad_tkmath::math::{Matrix, Vector};

/// The variable layout and the constraint forms shared by the fair-curve energies.
///
/// The variables are, in order: the length of the first tangent (constraint order
/// 1 or 2 at the start), the second-derivative term at the start (order 2), the
/// free poles, the end terms mirrored, and the free sliding. `energy.nb_variables()`.
#[derive(Clone)]
pub struct FairCurveEnergy {
    pub(super) poles: Handle<HArray1<Point2>>,
    pub(super) contr_order1: i32,
    pub(super) contr_order2: i32,
    pub(super) with_aux_value: bool,
    pub(super) nb_var: i32,
    pub(super) nb_values: i32,
    pub(super) linear_form: [Xy; 2],
    pub(super) quad_form: [Xy; 2],
}

impl FairCurveEnergy {
    /// The protected constructor; angles are measured from the Ox axis and
    /// `degree` scales the curvature forms;
    /// `FairCurveEnergy::new(&poles, [1, 1], false, [0.0, 0.0], 2, [0.0, 0.0])`.
    pub fn new(
        poles: &Handle<HArray1<Point2>>,
        [contr_order1, contr_order2]: [i32; 2],
        with_aux_value: bool,
        [angle1, angle2]: [f64; 2],
        degree: i32,
        [curvature1, curvature2]: [f64; 2],
    ) -> Self {
        let length: i32 = poles.get().map_or(0, |p| p.array1().len() as i32);
        let aux: i32 = i32::from(with_aux_value);
        let q0 = Xy::new(-angle1.sin(), angle1.cos());
        let q1 = Xy::new(angle2.sin(), angle2.cos());
        let scale: f64 = f64::from(degree) / f64::from(degree - 1);
        Self {
            poles: poles.clone(),
            contr_order1,
            contr_order2,
            with_aux_value,
            nb_var: 2 * (length - 2) - contr_order2 - contr_order1 + aux,
            nb_values: 2 * length + aux,
            linear_form: [
                Xy::new(angle1.cos(), angle1.sin()),
                Xy::new(-angle2.cos(), angle2.sin()),
            ],
            quad_form: [scale * curvature1 * q0, scale * curvature2 * q1],
        }
    }
    /// The number of free variables; `energy.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        self.nb_var
    }
    /// The poles the variables drive; `energy.poles()`.
    pub fn poles(&self) -> &Handle<HArray1<Point2>> {
        &self.poles
    }
    /// The number of intermediate values (`MyNbValues`): 2 per pole and the sliding.
    pub(super) fn nb_values(&self) -> i32 {
        self.nb_values
    }
    /// The packed Hessian index of (i, j), `j <= i`, after the gradient values.
    pub(super) fn indice(&self, i: i32, j: i32) -> i32 {
        self.nb_values + j + i * (i - 1) / 2
    }
    /// `P1 + LinearForm(side) * Lambda`: the pole after an end under a G1 constraint.
    pub(super) fn poles_g1(&self, side: usize, lambda: f64, p1: Point2) -> Point2 {
        Point2::from(p1.xy() + self.linear_form[side] * lambda)
    }
    /// `P1 + LinearForm(side) * (Lambda + Rho) + QuadForm(side) * Lambda^2`: the second
    /// pole after an end under a G2 constraint.
    pub(super) fn poles_g2(&self, side: usize, lambda: f64, rho: f64, p1: Point2) -> Point2 {
        let xy: Xy = p1.xy()
            + self.linear_form[side] * (lambda + rho)
            + self.quad_form[side] * (lambda * lambda);
        Point2::from(xy)
    }
    /// The variables matching the current poles (the sliding is left to the
    /// concrete energy); `energy.variable(&mut x)`.
    pub fn variable(&self, x: &mut Vector<'_>) -> bool {
        let guard = self.poles.get().expect("FairCurve poles").array1();
        let poles = &*guard;
        let (lower, upper) = (poles.lower(), poles.upper());
        let (mut first_pole, mut last_pole) = (lower + 1, upper - 1);
        let mut first_var: i32 = x.lower();
        let mut last_var: i32 = x.upper() - i32::from(self.with_aux_value);
        if self.contr_order1 >= 1 {
            x[first_var] = poles[lower].distance(poles[lower + 1]);
            (first_pole, first_var) = (first_pole + 1, first_var + 1);
        }
        if self.contr_order1 == 2 {
            let b1b2 = Vector2::between(poles[lower + 1], poles[lower + 2]);
            x[first_var] = b1b2.xy() * self.linear_form[0];
            (first_pole, first_var) = (first_pole + 1, first_var + 1);
        }
        if self.contr_order2 == 2 {
            let bn2bn1 = Vector2::between(poles[upper - 2], poles[upper - 1]);
            x[last_var] = -bn2bn1.xy() * self.linear_form[1];
            (last_pole, last_var) = (last_pole - 1, last_var - 1);
        }
        if self.contr_order2 >= 1 {
            x[last_var] = poles[upper].distance(poles[upper - 1]);
            last_pole -= 1;
        }
        for ii in first_pole..=last_pole {
            (x[first_var], x[first_var + 1]) = (poles[ii].x(), poles[ii].y());
            first_var += 2;
        }
        true
    }
    /// Moves the poles to the variables `x` (the sliding is left to the concrete
    /// energy); `energy.compute_poles(&x)`.
    pub fn compute_poles(&self, x: &Vector<'_>) {
        let mut guard = self.poles.get().expect("FairCurve poles").change_array1();
        let poles = &mut *guard;
        let (lower, upper) = (poles.lower(), poles.upper());
        let (mut first_pole, mut last_pole) = (lower + 1, upper - 1);
        let mut first_var: i32 = x.lower();
        let mut last_var: i32 = x.upper() - i32::from(self.with_aux_value);
        if self.contr_order1 >= 1 {
            (first_pole, first_var) = (first_pole + 1, first_var + 1);
            poles[lower + 1] = self.poles_g1(0, x[1], poles[lower]);
        }
        if self.contr_order1 == 2 {
            (first_pole, first_var) = (first_pole + 1, first_var + 1);
            poles[lower + 2] = self.poles_g2(0, x[1], x[2], poles[lower]);
        }
        if self.contr_order2 == 2 {
            (last_pole, last_var) = (last_pole - 1, last_var - 1);
            poles[upper - 2] = self.poles_g2(1, x[last_var], x[last_var + 1], poles[upper]);
        }
        if self.contr_order2 >= 1 {
            last_pole -= 1;
            poles[upper - 1] = self.poles_g1(1, x[last_var], poles[upper]);
        }
        for ii in first_pole..=last_pole {
            poles[ii] = Point2::new(x[first_var], x[first_var + 1]);
            first_var += 2;
        }
    }
}

/// A concrete fair-curve energy: the native virtual `ComputePoles`, `Compute` and
/// `Variable`, and the provided `Value`, `Gradient` and `Values` built on them.
///
/// `FairCurveEnergyOfBatten` and `FairCurveEnergyOfMvc` implement it, and through it
/// `math::MultipleVarFunctionWithHessian`; `energy.value(&x, &mut e)`.
pub trait FairCurveEnergyModel {
    /// The shared variable layout; `model.energy().nb_variables()`.
    fn energy(&self) -> &FairCurveEnergy;
    /// `ComputePoles(X)`: moves the poles (and the sliding) to `x`.
    fn compute_poles(&mut self, x: &Vector<'_>);
    /// `Compute(DerivativeOrder, Result)`: the energy (and derivatives) in the
    /// intermediate format of `energy_terms`; false when an integral fails.
    fn compute(&mut self, derivative_order: i32, result: &mut Vector<'_>) -> bool;
    /// `Variable(X)`: the variables matching the current poles and sliding.
    fn variable(&self, x: &mut Vector<'_>) -> bool;
    /// `NbVariables()`; `model.nb_variables()`.
    fn nb_variables(&self) -> i32 {
        self.energy().nb_var
    }
    /// `Poles()`; `model.poles()`.
    fn poles(&self) -> &Handle<HArray1<Point2>> {
        &self.energy().poles
    }
    /// `Value(X, E)`: the energy at `x`; `model.value(&x, &mut e)`.
    fn value(&mut self, x: &Vector<'_>, e: &mut f64) -> bool {
        let mut energie: Vector<'static> = Vector::filled(0, 0, 0.0).expect("energy size");
        self.compute_poles(x);
        let done: bool = self.compute(0, &mut energie);
        *e = energie[0];
        done
    }
    /// `Gradient(X, G)`: `Values(X, E, G)` without the energy; `model.gradient(&x, &mut g)`.
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        let mut e: f64 = 0.0;
        self.values(x, &mut e, g)
    }
    /// `Values(X, E, G)`: the energy and its gradient; `model.values(&x, &mut e, &mut g)`.
    fn values(&mut self, x: &Vector<'_>, e: &mut f64, g: &mut Vector<'_>) -> bool {
        let size: i32 = self.energy().nb_values();
        let mut buffer: Vector<'static> = Vector::filled(0, size, 0.0).expect("gradient size");
        self.compute_poles(x);
        let done: bool = self.compute(1, &mut buffer);
        if done {
            *e = buffer[0];
            self.energy().gradient1(&buffer, g);
        }
        done
    }
    /// `Values(X, E, G, H)`: the energy, its gradient and its Hessian;
    /// `model.values_hessian(&x, &mut e, &mut g, &mut h)`.
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        e: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        let n: i32 = self.energy().nb_values();
        let mut buffer: Vector<'static> =
            Vector::filled(0, n + n * (n + 1) / 2, 0.0).expect("hessian size");
        self.compute_poles(x);
        let done: bool = self.compute(2, &mut buffer);
        if done {
            *e = buffer[0];
            self.energy().gradient1(&buffer, g);
            self.energy().hessian1(&buffer, h);
        }
        done
    }
}

/// Implements the `math` objective traits of a `FairCurveEnergyModel`.
macro_rules! impl_energy_functions {
    ($energy:ty) => {
        impl rustycad_tkmath::math::MultipleVarFunction for $energy {
            fn variables(&self) -> i32 {
                $crate::fair_curve::FairCurveEnergyModel::nb_variables(self)
            }
            fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
                $crate::fair_curve::FairCurveEnergyModel::value(self, x, value)
            }
        }
        impl rustycad_tkmath::math::MultipleVarFunctionWithGradient for $energy {
            fn gradient(&mut self, x: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
                $crate::fair_curve::FairCurveEnergyModel::gradient(self, x, gradient)
            }
            fn values(
                &mut self,
                x: &Vector<'_>,
                value: &mut f64,
                gradient: &mut Vector<'_>,
            ) -> bool {
                $crate::fair_curve::FairCurveEnergyModel::values(self, x, value, gradient)
            }
        }
        impl rustycad_tkmath::math::MultipleVarFunctionWithHessian for $energy {
            fn values_hessian(
                &mut self,
                x: &Vector<'_>,
                value: &mut f64,
                gradient: &mut Vector<'_>,
                hessian: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> bool {
                $crate::fair_curve::FairCurveEnergyModel::values_hessian(
                    self, x, value, gradient, hessian,
                )
            }
        }
    };
}
pub(super) use impl_energy_functions;
