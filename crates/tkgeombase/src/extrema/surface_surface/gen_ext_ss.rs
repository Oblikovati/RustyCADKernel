// Rust translation of Extrema_GenExtSS.hxx/.cxx (with the file-local
// Extrema_FuncDistSS) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GenExtSS`: the nearest and farthest pairs of the samples of two
//! surfaces, the nearest refined by math_BFGS on the square distance (by
//! math_FunctionSetRoot on [`FuncExtSS`] when BFGS fails), the farthest by
//! math_FunctionSetRoot.
use super::func_ext_ss::FuncExtSS;
use crate::extrema::failures::{extremum_index, not_done, null_object};
use crate::extrema::pon::POnSurf;
use crate::failure_capture::FailureCapture;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{
    Bfgs, FunctionSetRoot, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector,
};

/// `Extrema_FuncDistSS`: `|S1(u1,v1) - S2(u2,v2)|²` with its half gradient.
struct FuncDistSS<'a> {
    s1: &'a dyn Adaptor3dSurface,
    s2: &'a dyn Adaptor3dSurface,
    failure: FailureCapture,
}

impl FuncDistSS<'_> {
    fn distance(&self, x: &Vector<'_>) -> Result<f64, Failure> {
        Ok(self
            .s1
            .value(x[1], x[2])?
            .square_distance(&self.s2.value(x[3], x[4])?))
    }
    fn half_gradient(&self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<(), Failure> {
        let a = self.s1.d1(x[1], x[2])?;
        let b = self.s2.d1(x[3], x[4])?;
        let p1p2: Vector3 = Vector3::between(b.point, a.point);
        g[1] = p1p2.dot(a.d1u);
        g[2] = p1p2.dot(a.d1v);
        g[3] = -p1p2.dot(b.d1u);
        g[4] = -p1p2.dot(b.d1v);
        Ok(())
    }
}

impl MultipleVarFunction for FuncDistSS<'_> {
    fn variables(&self) -> i32 {
        4
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let result = self.distance(point).map(|d| *value = d);
        self.failure.keep(result).is_some()
    }
}

impl MultipleVarFunctionWithGradient for FuncDistSS<'_> {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let result = self.half_gradient(point, gradient);
        self.failure.keep(result).is_some()
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        let result = self
            .distance(point)
            .map(|d| *value = d)
            .and_then(|()| self.half_gradient(point, gradient));
        self.failure.keep(result).is_some()
    }
}

/// The sampling of one surface: `n` parameters from `min + (max - min) / n / 200`
/// by `(max - min - (max - min) / n / 100) / (n - 1)` (the start and the step).
fn sampling(min: f64, max: f64, n: i32) -> (f64, f64) {
    let mut step: f64 = max - min;
    let mut start: f64 = step / f64::from(n) / 100.;
    step = (step - start) / f64::from(n - 1);
    start = min + start / 2.;
    (start, step)
}

/// The parameters `start + i step` accumulated, 1-based for GeomGridEval.
fn accumulated((start, step): (f64, f64), n: i32) -> Result<Array1<'static, f64>, Failure> {
    let mut array = Array1::with_bounds(1, n)
        .map_err(|error| array_failure(error, "Extrema_GenExtSS parameters"))?;
    let mut value: f64 = start;
    for index in 1..=n {
        array[index] = value;
        value += step;
    }
    Ok(array)
}

/// The grid points of a surface, `points[i - 1][j - 1]` at `(u_i, v_j)`.
fn grid_points(
    surface: &dyn Adaptor3dSurface,
    u: (f64, f64),
    v: (f64, f64),
    (nu, nv): (i32, i32),
) -> Result<Vec<Vec<Point3>>, Failure> {
    let grid = GeomGridEvalSurface::from_adaptor(surface)?
        .evaluate_grid(&accumulated(u, nu)?, &accumulated(v, nv)?)?;
    let mut points: Vec<Vec<Point3>> = Vec::new();
    for i in 1..=nu {
        let mut row: Vec<Point3> = Vec::new();
        for j in 1..=nv {
            row.push(
                *grid
                    .value(i, j)
                    .map_err(|error| array_failure(error, "NCollection_Array2::Value"))?,
            );
        }
        points.push(row);
    }
    Ok(points)
}

/// The sample indices (from 1) of the nearest and the farthest pairs.
#[derive(Clone, Copy, Default)]
struct SamplePair {
    n1u: i32,
    n1v: i32,
    n2u: i32,
    n2v: i32,
}

/// `Extrema_GenExtSS`; `GenExtSS::with_surfaces(&s1, &s2, 20, 20, 1e-9, 1e-9)?`.
#[derive(Default)]
pub struct GenExtSS<'a> {
    done: bool,
    u1min: f64,
    u1sup: f64,
    v1min: f64,
    v1sup: f64,
    u2min: f64,
    u2sup: f64,
    v2min: f64,
    v2sup: f64,
    usample: i32,
    vsample: i32,
    points2: Vec<Vec<Point3>>,
    tol1: f64,
    tol2: f64,
    f: FuncExtSS<'a>,
    s2: Option<&'a dyn Adaptor3dSurface>,
}

impl<'a> GenExtSS<'a> {
    /// `Extrema_GenExtSS()`; `GenExtSS::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GenExtSS(S1, S2, NbU, NbV, Tol1, Tol2)`: on the surface bounds;
    /// `GenExtSS::with_surfaces(&s1, &s2, 20, 20, 1e-9, 1e-9)?`.
    pub fn with_surfaces(
        s1: &'a dyn Adaptor3dSurface,
        s2: &'a dyn Adaptor3dSurface,
        nb_u: i32,
        nb_v: i32,
        tol1: f64,
        tol2: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self {
            f: FuncExtSS::with_surfaces(s1, s2),
            ..Self::default()
        };
        extrema.initialize(s2, nb_u, nb_v, tol2)?;
        extrema.perform(s1, tol1)?;
        Ok(extrema)
    }
    /// `Extrema_GenExtSS(S1, S2, NbU, NbV, U1min, U1sup, V1min, V1sup, U2min, U2sup,
    /// V2min, V2sup, Tol1, Tol2)`; `GenExtSS::with_ranges(&s1, &s2, (20, 20),
    /// [0., 1., 0., 1.], [0., 1., 0., 1.], (1e-9, 1e-9))?`.
    pub fn with_ranges(
        s1: &'a dyn Adaptor3dSurface,
        s2: &'a dyn Adaptor3dSurface,
        (nb_u, nb_v): (i32, i32),
        [u1min, u1sup, v1min, v1sup]: [f64; 4],
        [u2min, u2sup, v2min, v2sup]: [f64; 4],
        (tol1, tol2): (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self {
            f: FuncExtSS::with_surfaces(s1, s2),
            ..Self::default()
        };
        extrema.initialize_range(s2, (nb_u, nb_v), (u2min, u2sup), (v2min, v2sup), tol2)?;
        extrema.perform_range(s1, (u1min, u1sup), (v1min, v1sup), tol1)?;
        Ok(extrema)
    }
    /// `Initialize(S2, NbU, NbV, Tol2)`: on the bounds of S2;
    /// `extrema.initialize(&s2, 20, 20, 1e-9)?`.
    pub fn initialize(
        &mut self,
        s2: &'a dyn Adaptor3dSurface,
        nb_u: i32,
        nb_v: i32,
        tol2: f64,
    ) -> Result<(), Failure> {
        let u = (s2.first_u_parameter()?, s2.last_u_parameter()?);
        let v = (s2.first_v_parameter()?, s2.last_v_parameter()?);
        self.initialize_range(s2, (nb_u, nb_v), u, v, tol2)
    }
    /// `Initialize(S2, NbU, NbV, U2min, U2sup, V2min, V2sup, Tol2)`: the samples of S2;
    /// `extrema.initialize_range(&s2, (20, 20), (0., 1.), (0., 1.), 1e-9)?`.
    pub fn initialize_range(
        &mut self,
        s2: &'a dyn Adaptor3dSurface,
        (nb_u, nb_v): (i32, i32),
        (u2min, u2sup): (f64, f64),
        (v2min, v2sup): (f64, f64),
        tol2: f64,
    ) -> Result<(), Failure> {
        self.s2 = Some(s2);
        (self.usample, self.vsample) = (nb_u, nb_v);
        (self.u2min, self.u2sup, self.v2min, self.v2sup) = (u2min, u2sup, v2min, v2sup);
        self.tol2 = tol2;
        let u = sampling(u2min, u2sup, nb_u);
        let v = sampling(v2min, v2sup, nb_v);
        self.points2 = grid_points(s2, u, v, (nb_u, nb_v))?;
        Ok(())
    }
    /// `Perform(S1, Tol1)`: on the bounds of S1; `extrema.perform(&s1, 1e-9)?`.
    pub fn perform(&mut self, s1: &'a dyn Adaptor3dSurface, tol1: f64) -> Result<(), Failure> {
        let u = (s1.first_u_parameter()?, s1.last_u_parameter()?);
        let v = (s1.first_v_parameter()?, s1.last_v_parameter()?);
        self.perform_range(s1, u, v, tol1)
    }
    /// `Perform(S1, U1min, U1sup, V1min, V1sup, Tol1)`: the minimum and the maximum
    /// (done in any case);
    /// `extrema.perform_range(&s1, (0., 1.), (0., 1.), 1e-9)?`.
    pub fn perform_range(
        &mut self,
        s1: &'a dyn Adaptor3dSurface,
        (u1min, u1sup): (f64, f64),
        (v1min, v1sup): (f64, f64),
        tol1: f64,
    ) -> Result<(), Failure> {
        let s2 = self
            .s2
            .ok_or_else(|| null_object("Extrema_GenExtSS::Perform()"))?;
        self.f.initialize(s1, s2);
        (self.u1min, self.u1sup, self.v1min, self.v1sup) = (u1min, u1sup, v1min, v1sup);
        self.tol1 = tol1;
        let (nu, nv) = (self.usample, self.vsample);
        let s1_u = sampling(u1min, u1sup, nu);
        let s1_v = sampling(v1min, v1sup, nv);
        let s2_u = sampling(self.u2min, self.u2sup, nu);
        let s2_v = sampling(self.v2min, self.v2sup, nv);
        let points1: Vec<Vec<Point3>> = grid_points(s1, s1_u, s1_v, (nu, nv))?;
        let (min_pair, max_pair) = self.extreme_pairs(&points1);
        let start = |pair: SamplePair| -> Result<Vector<'static>, Failure> {
            let mut uv: Vector<'static> = Vector::with_bounds(1, 4)?;
            uv[1] = s1_u.0 + f64::from(pair.n1u - 1) * s1_u.1;
            uv[2] = s1_v.0 + f64::from(pair.n1v - 1) * s1_v.1;
            uv[3] = s2_u.0 + f64::from(pair.n2u - 1) * s2_u.1;
            uv[4] = s2_v.0 + f64::from(pair.n2v - 1) * s2_v.1;
            Ok(uv)
        };
        self.refine_minimum(s1, s2, start(min_pair)?)?;
        // The maximum cannot be computed in the same manner (the distance functional
        // is badly defined for it): the old approach.
        self.solve_roots(&start(max_pair)?)?;
        self.done = true;
        Ok(())
    }

    // The nearest and the farthest pairs of samples (the first of equal ones).
    fn extreme_pairs(&self, points1: &[Vec<Point3>]) -> (SamplePair, SamplePair) {
        let (mut distmin, mut distmax) = (f64::MAX, 0.0);
        let (mut min_pair, mut max_pair) = (SamplePair::default(), SamplePair::default());
        for (i1, row1) in points1.iter().enumerate() {
            for (j1, p1) in row1.iter().enumerate() {
                for (i2, row2) in self.points2.iter().enumerate() {
                    for (j2, p2) in row2.iter().enumerate() {
                        let dist: f64 = p1.square_distance(p2);
                        let pair = SamplePair {
                            n1u: i1 as i32 + 1,
                            n1v: j1 as i32 + 1,
                            n2u: i2 as i32 + 1,
                            n2v: j2 as i32 + 1,
                        };
                        if dist < distmin {
                            (distmin, min_pair) = (dist, pair);
                        }
                        if dist > distmax {
                            (distmax, max_pair) = (dist, pair);
                        }
                    }
                }
            }
        }
        (min_pair, max_pair)
    }

    // The minimum by math_BFGS from `uv` (stored in the function by Value and
    // GetStateNumber), or by math_FunctionSetRoot when BFGS fails.
    fn refine_minimum(
        &mut self,
        s1: &dyn Adaptor3dSurface,
        s2: &dyn Adaptor3dSurface,
        uv: Vector<'static>,
    ) -> Result<(), Failure> {
        let mut dist = FuncDistSS {
            s1,
            s2,
            failure: FailureCapture::default(),
        };
        let mut solver = Bfgs::new(4)?;
        solver.perform(&mut dist, &uv)?;
        dist.failure.take()?;
        if !solver.is_done() {
            // Restore the initial point: the old approach.
            return self.solve_roots(&uv);
        }
        let location: Vector<'static> = solver.location()?.clone();
        let mut scratch: Vector<'static> = Vector::with_bounds(1, 4)?;
        self.f.value(&location, &mut scratch)?;
        self.f.get_state_number()?;
        Ok(())
    }

    // math_FunctionSetRoot on the function from `uv` within the bounds.
    fn solve_roots(&mut self, uv: &Vector<'_>) -> Result<(), Failure> {
        let mut tol: Vector<'static> = Vector::with_bounds(1, 4)?;
        (tol[1], tol[2], tol[3], tol[4]) = (self.tol1, self.tol1, self.tol2, self.tol2);
        let mut inf: Vector<'static> = Vector::with_bounds(1, 4)?;
        (inf[1], inf[2], inf[3], inf[4]) = (self.u1min, self.v1min, self.u2min, self.v2min);
        let mut sup: Vector<'static> = Vector::with_bounds(1, 4)?;
        (sup[1], sup[2], sup[3], sup[4]) = (self.u1sup, self.v1sup, self.u2sup, self.v2sup);
        let mut solver = FunctionSetRoot::new(&self.f, &tol, 100)?;
        solver.perform_bounded(&mut self.f, uv, &inf, &sup, false)?;
        self.f.take_failure()
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GenExtSS::NbExt()"));
        }
        Ok(self.f.nb_ext())
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        extremum_index("Extrema_GenExtSS::SquareDistance()", n, || self.nb_ext())?;
        self.f.square_distance(n)
    }
    /// `PointOnS1(N)`; `extrema.point_on_s1(1)?`.
    pub fn point_on_s1(&self, n: i32) -> Result<POnSurf, Failure> {
        extremum_index("Extrema_GenExtSS::PointOnS1()", n, || self.nb_ext())?;
        self.f.point_on_s1(n)
    }
    /// `PointOnS2(N)`; `extrema.point_on_s2(1)?`.
    pub fn point_on_s2(&self, n: i32) -> Result<POnSurf, Failure> {
        extremum_index("Extrema_GenExtSS::PointOnS2()", n, || self.nb_ext())?;
        self.f.point_on_s2(n)
    }
}
