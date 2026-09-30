// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_DistributionOfEnergy.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_DistributionOfEnergy`: the common state of the energy densities, and
//! the B-spline evaluation and packed-Hessian walk the three densities share.
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmath::math::{Matrix, Vector};

/// The shared state of an energy density along a B-spline: its order, flat knots,
/// poles (shared with the energy that moves them) and the number of equations for
/// the requested derivative order.
///
/// A density of derivative order 1 on 10 poles without auxiliary value has
/// `1 + 2 * 10` equations; `density.nb_equations()`.
#[derive(Clone)]
pub struct FairCurveDistributionOfEnergy {
    pub(super) bspl_order: i32,
    pub(super) flat_knots: Handle<HArray1<f64>>,
    pub(super) poles: Handle<HArray1<Point2>>,
    pub(super) derivative_order: i32,
    nb_var: i32,
    nb_equa: i32,
    pub(super) nb_val_aux: i32,
}

impl FairCurveDistributionOfEnergy {
    /// The protected constructor: one variable (the parameter) and the equation count
    /// of `derivative_order`;
    /// `FairCurveDistributionOfEnergy::new(10, flat, poles, 1, 0)`.
    pub fn new(
        bspl_order: i32,
        flat_knots: &Handle<HArray1<f64>>,
        poles: &Handle<HArray1<Point2>>,
        derivative_order: i32,
        nb_val_aux: i32,
    ) -> Self {
        let mut density = Self {
            bspl_order,
            flat_knots: flat_knots.clone(),
            poles: poles.clone(),
            derivative_order,
            nb_var: 1,
            nb_equa: 0,
            nb_val_aux,
        };
        density.set_derivative_order(derivative_order);
        density
    }
    /// The number of variables, always 1; `density.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        self.nb_var
    }
    /// The value, then the gradient, then the packed lower Hessian;
    /// `density.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        self.nb_equa
    }
    /// Selects the value (0), the gradient (1) or the Hessian (2) as well;
    /// `density.set_derivative_order(2)`.
    pub fn set_derivative_order(&mut self, derivative_order: i32) {
        self.nb_equa = 1;
        if derivative_order >= 1 {
            self.nb_equa += 2 * self.poles_length() + self.nb_val_aux;
        }
        if derivative_order >= 2 {
            self.nb_equa = self.nb_equa + (self.nb_equa - 1) * self.nb_equa / 2;
        }
        self.derivative_order = derivative_order;
    }
    fn poles_length(&self) -> i32 {
        self.poles
            .get()
            .map_or(0, |poles| poles.array1().len() as i32)
    }
    /// `BSplCLib::EvalBsplineBasis` with `request` derivatives at `t`, and the curve
    /// derivatives of order 1..=request; `None` where the native code returns false.
    pub(super) fn local_basis(&self, request: i32, t: f64) -> Option<LocalBasis> {
        let mut base: Matrix<'static> =
            Matrix::with_bounds(1, request + 2, 1, self.bspl_order).ok()?;
        let mut first_non_zero: i32 = 0;
        let flat = self.flat_knots.get()?.array1();
        let ier: i32 = bsplclib::eval_bspline_basis(
            request,
            self.bspl_order,
            &flat,
            t,
            &mut first_non_zero,
            &mut base,
            false,
        )
        .ok()?;
        if ier != 0 {
            return None;
        }
        let last_zero: i32 = first_non_zero - 1;
        let derivatives: Vec<Xy> = (1..=request)
            .map(|order| self.derivative(&base, order, last_zero))
            .collect();
        Some(LocalBasis {
            base,
            last_zero,
            derivatives,
        })
    }
    /// The sum of `Base(order + 1, ii) * Pole(ii + LastZero)` in the native order.
    fn derivative(&self, base: &Matrix<'_>, order: i32, last_zero: i32) -> Xy {
        let poles = self.poles.get().expect("FairCurve poles").array1();
        let mut sum: Xy = Xy::new(0.0, 0.0);
        for ii in 1..=self.bspl_order {
            sum += base[(order + 1, ii)] * poles[ii + last_zero].xy();
        }
        sum
    }
    /// The index of the last gradient entry: `Lower + 2 * NbPoles (+ 1 with sliding)`.
    pub(super) fn last_gradient_index(&self, lower: i32) -> i32 {
        lower + 2 * self.poles_length() + i32::from(self.nb_val_aux == 1)
    }
}

/// The non-zero basis functions at a parameter and the curve derivatives there.
pub(super) struct LocalBasis {
    /// Row `r + 1` holds the r-th derivatives of the non-zero basis functions.
    pub(super) base: Matrix<'static>,
    /// The index of the pole before the first non-zero one.
    pub(super) last_zero: i32,
    /// `derivatives[k - 1]` is the k-th derivative of the curve.
    pub(super) derivatives: Vec<Xy>,
}

impl LocalBasis {
    /// `Base(row, column)`.
    pub(super) fn at(&self, row: i32, column: i32) -> f64 {
        self.base[(row, column)]
    }
    /// The native `FirstNonZero = 2 * LastZero + 1`: the offset of the first
    /// non-zero gradient entry.
    pub(super) fn first_non_zero(&self) -> i32 {
        2 * self.last_zero + 1
    }
}

/// Writes the packed lower triangle of the pole block of a density Hessian as the
/// three densities do: `pair(ii, jj)` gives the XiXj, YiXj (next slot), XiYj and
/// YiYj terms of the even indices `jj < ii`, and `diagonal(ii)` the XiXi, XiYi and
/// YiYi terms. `walk_pole_hessian(f, first, last, order, pair, diagonal)`.
pub(super) fn walk_pole_hessian(
    f: &mut Vector<'_>,
    [first_non_zero, last_gradient_index, bspl_order]: [i32; 3],
    mut pair: impl FnMut(i32, i32) -> [f64; 4],
    mut diagonal: impl FnMut(i32) -> [f64; 3],
) {
    let mut kk: i32 = first_non_zero;
    let mut k2: i32 = last_gradient_index + (kk - 1) * kk / 2;
    for ii in (2..=2 * bspl_order).step_by(2) {
        let mut k1: i32 = k2 + first_non_zero;
        k2 = k1 + kk;
        kk += 2;
        for jj in (2..ii).step_by(2) {
            let [xx, yx, xy, yy] = pair(ii, jj);
            (f[k1], f[k1 + 1], f[k2], f[k2 + 1]) = (xx, yx, xy, yy);
            k1 += 2;
            k2 += 2;
        }
        let [xx, xy, yy] = diagonal(ii);
        (f[k1], f[k2], f[k2 + 1]) = (xx, xy, yy);
        // The native diagonal ends on the YiYi slot: the next row starts after it.
        k2 += 1;
    }
}
