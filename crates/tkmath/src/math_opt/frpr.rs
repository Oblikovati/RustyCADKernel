//! MathOpt_FRPR.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use super::*;
use crate::math_utils::{ARMIJO_C1, dot_product, wolfe_search};
/// Native beta formula, including invalid integer values; `ConjugateGradientFormula::PolakRibiere`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConjugateGradientFormula {
    /// Squared gradient ratio; `ConjugateGradientFormula::FletcherReeves`.
    FletcherReeves,
    /// Difference projection with restart; `ConjugateGradientFormula::PolakRibiere`.
    PolakRibiere,
    /// Direction projection denominator; `ConjugateGradientFormula::HestenesStiefel`.
    HestenesStiefel,
    /// Squared norm over direction projection; `ConjugateGradientFormula::DaiYuan`.
    DaiYuan,
    /// Preserves native invalid-enum handling; `ConjugateGradientFormula::Invalid(42)`.
    Invalid(i32),
}
/// Native conjugate-gradient options; `FrprConfig::default()`.
#[derive(Clone, Debug)]
pub struct FrprConfig {
    /// Shared scalar tolerances; `config.base.f_tolerance`.
    pub base: Config,
    /// Beta update rule; `config.formula`.
    pub formula: ConjugateGradientFormula,
    /// Zero selects dimension; `config.restart_interval=2`.
    pub restart_interval: u32,
}
impl Default for FrprConfig {
    fn default() -> Self {
        Self {
            base: Config::default(),
            formula: ConjugateGradientFormula::PolakRibiere,
            restart_interval: 0,
        }
    }
}
struct FrprState {
    x: Vector<'static>,
    g: Vector<'static>,
    dir: Vector<'static>,
    next: Vector<'static>,
    next_g: Vector<'static>,
    diff: Vector<'static>,
    fx: f64,
    norm_sq: f64,
    restarts: usize,
    iterations: u32,
}
impl FrprState {
    fn initialize(
        f: &mut impl MultipleVarFunctionWithGradient,
        start: &Vector<'_>,
    ) -> Result<Self, Status> {
        let x: Vector<'static> = zero_based(start);
        let n: usize = x.length() as usize;
        let mut fx: f64 = 0.0;
        let status: Status = value_status(f, &x, &mut fx);
        if status != Status::Ok {
            return Err(status);
        }
        let mut g: Vector<'static> = Vector::with_size(n);
        let status: Status = gradient_status(f, &x, &mut g);
        if status != Status::Ok {
            return Err(status);
        }
        let norm_sq: f64 = dot_product(&g, &g);
        let mut dir: Vector<'static> = Vector::with_size(n);
        for i in 0..n as i32 {
            dir[i] = -g[i];
        }
        Ok(Self {
            x,
            g,
            dir,
            next: Vector::with_size(n),
            next_g: Vector::with_size(n),
            diff: Vector::with_size(n),
            fx,
            norm_sq,
            restarts: 0,
            iterations: 0,
        })
    }
    fn finish(self, status: Status) -> VectorResult {
        VectorResult {
            status,
            iterations: self.iterations,
            solution: Some(self.x),
            value: Some(self.fx),
            gradient: Some(self.g),
            ..Default::default()
        }
    }
    fn line(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        cfg: &Config,
    ) -> LineSearchResult {
        let mut result: LineSearchResult = wolfe_search(
            f,
            &self.x,
            &self.dir,
            &self.g,
            self.fx,
            [1.0, ARMIJO_C1, FRPR_WOLFE_CURVATURE],
            20,
        );
        if result.alpha < cfg.step_min {
            result.is_valid = false;
        }
        result
    }
    fn search(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        cfg: &Config,
    ) -> Result<LineSearchResult, Status> {
        let first: LineSearchResult = self.line(f, cfg);
        if first.is_valid {
            return Ok(first);
        }
        for i in 0..self.x.length() {
            self.dir[i] = -self.g[i];
        }
        let last: LineSearchResult = self.line(f, cfg);
        if !last.is_valid {
            return Err(failure_after_fallback(first.status, last.status));
        }
        self.restarts = 0;
        Ok(last)
    }
    fn accept(&mut self, fx: f64, norm_sq: f64) {
        self.x.initialized(&self.next).unwrap();
        self.g.initialized(&self.next_g).unwrap();
        self.fx = fx;
        self.norm_sq = norm_sq;
    }
    fn next_norm(&self) -> f64 {
        let mut sum: f64 = 0.0;
        for i in 0..self.x.length() {
            sum += self.next_g[i] * self.next_g[i];
        }
        sum
    }
    fn beta(&mut self, cfg: &FrprConfig, new_norm: f64) -> Result<f64, Status> {
        self.restarts += 1;
        let interval: usize = if cfg.restart_interval > 0 {
            cfg.restart_interval as usize
        } else {
            self.x.length() as usize
        };
        if self.restarts >= interval {
            self.restarts = 0;
            return Ok(0.0);
        }
        let (num, den): (f64, f64) = self.projections();
        let mut beta: f64 = match cfg.formula {
            ConjugateGradientFormula::FletcherReeves => {
                if self.norm_sq > ZERO_TOL {
                    new_norm / self.norm_sq
                } else {
                    0.0
                }
            }
            ConjugateGradientFormula::PolakRibiere => {
                if self.norm_sq > ZERO_TOL {
                    num / self.norm_sq
                } else {
                    0.0
                }
            }
            ConjugateGradientFormula::HestenesStiefel => {
                if den.abs() > ZERO_TOL {
                    num / den
                } else {
                    0.0
                }
            }
            ConjugateGradientFormula::DaiYuan => {
                if den.abs() > ZERO_TOL {
                    new_norm / den
                } else {
                    0.0
                }
            }
            ConjugateGradientFormula::Invalid(_) => return Err(Status::InvalidInput),
        };
        let nonnegative: bool = matches!(
            cfg.formula,
            ConjugateGradientFormula::PolakRibiere | ConjugateGradientFormula::HestenesStiefel
        );
        if !beta.is_finite() || (nonnegative && beta < 0.0) {
            beta = 0.0;
            self.restarts = 0;
        }
        Ok(beta)
    }
    fn projections(&self) -> (f64, f64) {
        let mut num: f64 = 0.0;
        let mut den: f64 = 0.0;
        for i in 0..self.x.length() {
            num += self.next_g[i] * self.diff[i];
            den += self.dir[i] * self.diff[i];
        }
        (num, den)
    }
    fn direction(&mut self, beta: f64) {
        for i in 0..self.x.length() {
            self.dir[i] = -self.next_g[i] + beta * self.dir[i];
        }
        let mut derivative: f64 = 0.0;
        for i in 0..self.x.length() {
            derivative += self.next_g[i] * self.dir[i];
        }
        if !derivative.is_finite() || derivative >= 0.0 {
            for i in 0..self.x.length() {
                self.dir[i] = -self.next_g[i];
            }
            self.restarts = 0;
        }
    }
    fn iteration(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        cfg: &FrprConfig,
    ) -> Result<bool, Status> {
        let line: LineSearchResult = self.search(f, &cfg.base)?;
        for i in 0..self.x.length() {
            self.next[i] = self.x[i] + line.alpha * self.dir[i];
        }
        let status: Status = gradient_status(f, &self.next, &mut self.next_g);
        if status != Status::Ok {
            return Err(status);
        }
        let norm_sq: f64 = self.next_norm();
        if norm_sq.sqrt() < cfg.base.f_tolerance {
            self.accept(line.f_new, norm_sq);
            return Ok(true);
        }
        for i in 0..self.x.length() {
            self.diff[i] = self.next_g[i] - self.g[i];
        }
        let beta: f64 = self.beta(cfg, norm_sq)?;
        self.direction(beta);
        self.accept(line.f_new, norm_sq);
        Ok(false)
    }
}
/// Minimizes with native conjugate-gradient restarts; `frpr(&mut f,&start,&FrprConfig::default())`.
pub fn frpr(
    f: &mut impl MultipleVarFunctionWithGradient,
    start: &Vector<'_>,
    cfg: &FrprConfig,
) -> VectorResult {
    if !is_valid_config(&cfg.base) || !is_finite_vector(start) {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut state: FrprState = match FrprState::initialize(f, start) {
        Ok(s) => s,
        Err(status) => {
            return VectorResult {
                status,
                ..Default::default()
            };
        }
    };
    if state.norm_sq.sqrt() < cfg.base.f_tolerance {
        return state.finish(Status::Ok);
    }
    for iteration in 1..=cfg.base.max_iterations {
        state.iterations = iteration;
        match state.iteration(f, cfg) {
            Ok(true) => return state.finish(Status::Ok),
            Err(status) => return state.finish(status),
            Ok(false) => {}
        }
    }
    state.finish(Status::MaxIterations)
}
/// Uses native adaptive central differences; `frpr_numerical(&mut f,&start,1e-8,&FrprConfig::default())`.
pub fn frpr_numerical(
    f: &mut impl MultipleVarFunction,
    start: &Vector<'_>,
    step: f64,
    cfg: &FrprConfig,
) -> VectorResult {
    if !step.is_finite() || step <= 0.0 {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    frpr(
        &mut super::bfgs::Numerical { function: f, step },
        start,
        cfg,
    )
}
impl FrprConfig {
    /// Uses the base tolerance constructor; `FrprConfig::new(1e-8,100)`.
    pub fn new(tolerance: f64, max_iterations: u32) -> Self {
        Self {
            base: Config::new(tolerance, max_iterations),
            ..Default::default()
        }
    }
}
