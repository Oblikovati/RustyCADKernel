//! MathOpt_GlobOpt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math_utils::Config;
/// Native global method selection; `GlobalStrategy::PSOHybrid`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlobalStrategy {
    /// Native swarm-only strategy.
    PSO,
    /// Native repeated Powell strategy.
    MultiStart,
    /// Native swarm followed by Powell.
    #[default]
    PSOHybrid,
    /// Native differential evolution strategy.
    DifferentialEvolution,
}
/// Native global optimizer configuration; `GlobalConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct GlobalConfig {
    /// Native solver field; `value.solver`.
    pub solver: Config,
    /// Native use_bounds field; `value.use_bounds`.
    pub use_bounds: bool,
    /// Native strategy field; `value.strategy`.
    pub strategy: GlobalStrategy,
    /// Native population field; `value.population`.
    pub population: usize,

    /// Native starts field; `value.starts`.
    pub starts: usize,
    /// Native mutation_scale field; `value.mutation_scale`.
    pub mutation_scale: f64,
    /// Native crossover_prob field; `value.crossover_prob`.
    pub crossover_prob: f64,
    /// Native seed field; `value.seed`.
    pub seed: u32,
    /// Native polish_budget_per_dim field; `value.polish_budget_per_dim`.
    pub polish_budget_per_dim: u32,
}
impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new(GlobalStrategy::PSOHybrid, 200)
    }
}
impl GlobalConfig {
    /// Uses native tolerances and population defaults; `GlobalConfig::new(GlobalStrategy::PSO,200)`.
    pub fn new(strategy: GlobalStrategy, limit: u32) -> Self {
        Self {
            solver: Config::new(1e-8, limit),
            use_bounds: true,
            strategy,
            population: 40,
            starts: 10,
            mutation_scale: 0.8,
            crossover_prob: 0.9,
            seed: 6,
            polish_budget_per_dim: 50,
        }
    }
    pub(super) fn local(&self, iterations: u32) -> Config {
        Config {
            tolerance: self.solver.tolerance,
            x_tolerance: self.solver.tolerance,
            f_tolerance: self.solver.tolerance,
            max_iterations: iterations,
            ..Default::default()
        }
    }
}
