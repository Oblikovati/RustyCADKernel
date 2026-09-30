// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImp/IntImp_ZerParFunc.gxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImp_ZerParFunc`: the distance between two parametric patches along an
//! isoparametric line of one of them, a function of the three free parameters.
use super::captured::captured_function_set;
use super::{IntImpConstIsoparametric, ThePSurfaceTool, compute_tangence};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Direction2, Direction3, Point3, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The distance function of two patches of tool `T`, with U or V of one patch
/// fixed (`ComputeParameters` chooses which).
///
/// `IntWalk_TheFunctionOfTheInt2S` is `IntImpZerParFunc<HSurfaceTool>`;
/// `IntImpZerParFunc::<HSurfaceTool>::new(&s1, &s2)?`.
pub struct IntImpZerParFunc<T: ThePSurfaceTool> {
    surf1: T::Surface,
    surf2: T::Surface,
    pntsol1: Point3,
    pntsol2: Point3,
    f: [f64; 3],
    tangent: bool,
    tgduv: [f64; 4],
    dpuv: [Vector3; 4],
    chx_iso: IntImpConstIsoparametric,
    param_const: f64,
    /// `(ua0, ua1)`, `(va0, va1)`, `(ub0, ub1)`, `(vb0, vb1)` with the resolutions
    /// `ures1`, `vres1`, `ures2`, `vres2`: one entry per native parameter index.
    ranges: [(f64, f64, f64); 4],
    pub(super) failure: FailureCapture,
}

/// The parameter index (1-based) that each isoparametric fixes.
fn const_index(iso: IntImpConstIsoparametric) -> usize {
    match iso {
        IntImpConstIsoparametric::UIsoparametricOnCaro1 => 1,
        IntImpConstIsoparametric::VIsoparametricOnCaro1 => 2,
        IntImpConstIsoparametric::UIsoparametricOnCaro2 => 3,
        IntImpConstIsoparametric::VIsoparametricOnCaro2 => 4,
    }
}

/// The three free parameter indices of an isoparametric, in order.
fn free_indices(iso: IntImpConstIsoparametric) -> [usize; 3] {
    let fixed: usize = const_index(iso);
    let mut free = [0usize; 3];
    for (next, index) in (1..=4).filter(|&i| i != fixed).enumerate() {
        free[next] = index;
    }
    free
}

impl<T: ThePSurfaceTool> IntImpZerParFunc<T> {
    /// The function of the patches `s1` and `s2`, with their bounds and the
    /// resolutions of `Precision::Confusion()`.
    pub fn new(s1: &T::Surface, s2: &T::Surface) -> Result<Self, Failure> {
        let ranges = patch_ranges::<T>(s1, s2)?;
        Ok(Self {
            surf1: s1.clone(),
            surf2: s2.clone(),
            pntsol1: Point3::default(),
            pntsol2: Point3::default(),
            f: [0.0; 3],
            tangent: false,
            tgduv: [0.0; 4],
            dpuv: [Vector3::default(); 4],
            chx_iso: IntImpConstIsoparametric::default(),
            param_const: 0.0,
            ranges,
            failure: FailureCapture::default(),
        })
    }
    /// Three variables; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        3
    }
    /// Three equations; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        3
    }
    /// The patch parameters `(u1, v1, u2, v2)` of the free variables `x`.
    fn parameters(&self, x: &Vector<'_>) -> [f64; 4] {
        let mut params = [self.param_const; 4];
        for (k, index) in free_indices(self.chx_iso).into_iter().enumerate() {
            params[index - 1] = x[k as i32 + 1];
        }
        params
    }
    fn store_difference(&mut self, f: &mut Vector<'_>) {
        let (p1, p2) = (self.pntsol1, self.pntsol2);
        self.f = [p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z()];
        (f[1], f[2], f[3]) = (self.f[0], self.f[1], self.f[2]);
    }
    /// `Value(X, F)`: the difference of the two patch points; `function.value(&x, &mut f)?`.
    pub fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let [u1, v1, u2, v2] = self.parameters(x);
        self.pntsol1 = T::value(&self.surf1, u1, v1)?;
        self.pntsol2 = T::value(&self.surf2, u2, v2)?;
        self.store_difference(f);
        Ok(true)
    }
    /// The first derivatives of both patches, and the Jacobian columns of the free
    /// variables (the second patch's derivatives negated).
    fn jacobian(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<(), Failure> {
        let [u1, v1, u2, v2] = self.parameters(x);
        let first = T::d1(&self.surf1, u1, v1)?;
        (self.pntsol1, self.dpuv[0], self.dpuv[1]) = (first.point, first.d1u, first.d1v);
        let second = T::d1(&self.surf2, u2, v2)?;
        (self.pntsol2, self.dpuv[2], self.dpuv[3]) = (second.point, second.d1u, second.d1v);
        for (column, index) in free_indices(self.chx_iso).into_iter().enumerate() {
            let derivative: Vector3 = self.dpuv[index - 1];
            let sign: f64 = if index <= 2 { 1.0 } else { -1.0 };
            let c: i32 = column as i32 + 1;
            (d[(1, c)], d[(2, c)], d[(3, c)]) = if sign > 0.0 {
                (derivative.x(), derivative.y(), derivative.z())
            } else {
                (-derivative.x(), -derivative.y(), -derivative.z())
            };
        }
        Ok(())
    }
    /// `Derivatives(X, D)`; `function.derivatives(&x, &mut d)?`.
    pub fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        self.jacobian(x, d)?;
        Ok(true)
    }
    /// `Values(X, F, D)`; `function.values(&x, &mut f, &mut d)?`.
    pub fn values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        self.jacobian(x, d)?;
        self.store_difference(f);
        Ok(true)
    }
    /// `ComputeParameters`: fixes the parameter of `choix_iso` from `param` (1-based
    /// `u1 v1 u2 v2`) and writes the start, the bounds widened by 1% and the
    /// tolerances of the three free ones.
    pub fn compute_parameters(
        &mut self,
        choix_iso: IntImpConstIsoparametric,
        param: &Array1<'_, f64>,
        uvap: &mut Vector<'_>,
        [born_inf, born_sup, tolerance]: [&mut Vector<'_>; 3],
    ) {
        self.chx_iso = choix_iso;
        self.param_const = param[const_index(choix_iso) as i32];
        for (k, index) in free_indices(choix_iso).into_iter().enumerate() {
            let (first, last, resolution) = self.ranges[index - 1];
            let i: i32 = k as i32 + 1;
            (uvap[i], born_inf[i], born_sup[i], tolerance[i]) =
                (param[index as i32], first, last, resolution);
        }
        for i in 1..=3 {
            let incr: f64 = (born_sup[i] - born_inf[i]) * 0.01;
            born_inf[i] -= incr;
            born_sup[i] += incr;
        }
    }
    /// `IsTangent(UVap, Param, BestChoix)`: writes the full parameters of the
    /// solution `uvap` into `param`, then whether the patches are tangent there
    /// (from the derivatives of the last `Derivatives`/`Values`); otherwise the best
    /// isoparametric becomes the current one and is returned in `best_choix`.
    pub fn is_tangent(
        &mut self,
        uvap: &Vector<'_>,
        param: &mut Array1<'_, f64>,
        best_choix: &mut IntImpConstIsoparametric,
    ) -> bool {
        let params = self.parameters(uvap);
        for (index, value) in params.into_iter().enumerate() {
            param[index as i32 + 1] = value;
        }
        let eps_uv: [f64; 4] = self.ranges.map(|(_, _, resolution)| resolution);
        let mut tab_iso = [IntImpConstIsoparametric::default(); 4];
        self.tangent = compute_tangence(&self.dpuv, &eps_uv, &mut self.tgduv, &mut tab_iso);
        if !self.tangent {
            self.chx_iso = tab_iso[0];
        }
        *best_choix = self.chx_iso;
        self.tangent
    }
    /// `Root()`: the squared distance of the last points; `function.root()`.
    pub fn root(&self) -> f64 {
        self.f[0] * self.f[0] + self.f[1] * self.f[1] + self.f[2] * self.f[2]
    }
    /// `Point()`: the middle of the last points; `function.point()`.
    pub fn point(&self) -> Point3 {
        Point3::from((self.pntsol1.xyz() + self.pntsol2.xyz()) / 2.0)
    }
    fn undefined(&self, member: &str) -> Result<(), Failure> {
        if self.tangent {
            return Err(Failure::with_message(
                "StdFail_UndefinedDerivative",
                &format!("IntImp_ZerParFunc::{member}: the patches are tangent"),
            ));
        }
        Ok(())
    }
    /// `Direction()`: the 3d tangent of the intersection; `StdFail_UndefinedDerivative`
    /// on tangent patches; `function.direction()?`.
    pub fn direction(&self) -> Result<Direction3, Failure> {
        self.undefined("Direction")?;
        let (a, b) = (*self.dpuv[0].xyz(), *self.dpuv[1].xyz());
        Direction3::from_xyz(self.tgduv[0] * a + self.tgduv[1] * b)
    }
    /// `DirectionOnS1()`: the tangent in the parameters of the first patch.
    pub fn direction_on_s1(&self) -> Result<Direction2, Failure> {
        self.undefined("DirectionOnS1")?;
        Direction2::new(self.tgduv[0], self.tgduv[1])
    }
    /// `DirectionOnS2()`: the tangent in the parameters of the second patch.
    pub fn direction_on_s2(&self) -> Result<Direction2, Failure> {
        self.undefined("DirectionOnS2")?;
        Direction2::new(self.tgduv[2], self.tgduv[3])
    }
    /// `AuxillarSurface1()`; `function.auxillar_surface1()`.
    pub fn auxillar_surface1(&self) -> &T::Surface {
        &self.surf1
    }
    /// `AuxillarSurface2()`; `function.auxillar_surface2()`.
    pub fn auxillar_surface2(&self) -> &T::Surface {
        &self.surf2
    }
    /// The first failure raised by a solver callback since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

/// The bounds and resolutions (`Precision::Confusion()`) of the four patch
/// parameters, queried in the native order: the U and V bounds of the first patch,
/// then of the second, then the four resolutions. Shared with `IntImp_Int2S`.
pub(super) fn patch_ranges<T: ThePSurfaceTool>(
    s1: &T::Surface,
    s2: &T::Surface,
) -> Result<[(f64, f64, f64); 4], Failure> {
    let bounds = |s: &T::Surface| -> Result<[f64; 4], Failure> {
        let (u0, v0) = (T::first_u_parameter(s)?, T::first_v_parameter(s)?);
        Ok([u0, v0, T::last_u_parameter(s)?, T::last_v_parameter(s)?])
    };
    let ([ua0, va0, ua1, va1], [ub0, vb0, ub1, vb1]) = (bounds(s1)?, bounds(s2)?);
    let conf: f64 = precision::confusion();
    let (ures1, vres1) = (T::u_resolution(s1, conf)?, T::v_resolution(s1, conf)?);
    let (ures2, vres2) = (T::u_resolution(s2, conf)?, T::v_resolution(s2, conf)?);
    Ok([
        (ua0, ua1, ures1),
        (va0, va1, vres1),
        (ub0, ub1, ures2),
        (vb0, vb1, vres2),
    ])
}

captured_function_set!([T: ThePSurfaceTool] IntImpZerParFunc<T>);
