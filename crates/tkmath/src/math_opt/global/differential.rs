// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! MathOpt_GlobOpt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
struct Population {
    points: Vec<Vector<'static>>,
    fitness: Vec<Option<f64>>,
    best: Option<usize>,
    best_value: Option<f64>,
    rng: RandomGenerator,
}
impl Population {
    fn new(
        function: &mut impl MultipleVarFunction,
        bounds: &BoxBounds<'_>,
        config: &GlobalConfig,
    ) -> Self {
        let mut result: Self = Self {
            points: Vec::new(),
            fitness: Vec::new(),
            best: None,
            best_value: None,
            rng: RandomGenerator::new(config.seed as u64),
        };
        for _ in 0..config.population {
            let point: Vector<'static> = random_point(bounds, &mut result.rng);
            let mut value: f64 = 0.0;
            let fitness: Option<f64> = function.value(&point, &mut value).then_some(value);
            result.points.push(point);
            result.fitness.push(fitness);
        }
        for index in 0..config.population {
            result.improve(index);
        }
        result
    }
    fn improve(&mut self, index: usize) {
        if let Some(value) =
            self.fitness[index].filter(|v| self.best_value.is_none_or(|old| *v < old))
        {
            self.best_value = Some(value);
            self.best = Some(index);
        }
    }
    fn indices(&mut self, current: usize) -> [usize; 3] {
        let mut chosen: Vec<usize> = vec![current];
        for _ in 0..3 {
            let index: usize = loop {
                let index: usize = (self.rng.next_real() * self.points.len() as f64) as usize;
                if !chosen.contains(&index) {
                    break index;
                }
            };
            chosen.push(index);
        }
        [chosen[1], chosen[2], chosen[3]]
    }
    fn candidate(
        &mut self,
        index: usize,
        bounds: &BoxBounds<'_>,
        config: &GlobalConfig,
        trial: &mut Vector<'_>,
    ) {
        let [a, b, c]: [usize; 3] = self.indices(index);
        let dimension: usize = trial.length() as usize;
        let force: usize = (self.rng.next_real() * dimension as f64) as usize;
        for i in 0..dimension {
            *trial.change_at(i).unwrap() =
                if self.rng.next_real() < config.crossover_prob || i == force {
                    let mutated: f64 = self.points[a].at(i).unwrap()
                        + config.mutation_scale
                            * (self.points[b].at(i).unwrap() - self.points[c].at(i).unwrap());
                    clamp(
                        mutated,
                        *bounds.lower.at(i).unwrap(),
                        *bounds.upper.at(i).unwrap(),
                    )
                } else {
                    *self.points[index].at(i).unwrap()
                };
        }
    }
    fn evolve(
        &mut self,
        function: &mut impl MultipleVarFunction,
        bounds: &BoxBounds<'_>,
        config: &GlobalConfig,
        trial: &mut Vector<'_>,
    ) {
        for index in 0..self.points.len() {
            self.candidate(index, bounds, config, trial);
            let mut value: f64 = 0.0;
            if function.value(trial, &mut value)
                && self.fitness[index].is_none_or(|old| value <= old)
            {
                self.points[index].initialized(trial).unwrap();
                self.fitness[index] = Some(value);
                self.improve(index);
            }
        }
    }
    fn converged(&self, tolerance: f64) -> bool {
        let mut max_diff: f64 = 0.0;
        let mut valid: usize = 0;
        for value in self.fitness.iter().flatten() {
            valid += 1;
            max_diff = max_diff.max((*value - self.best_value.unwrap()).abs());
        }
        valid == self.points.len() && max_diff < tolerance
    }
}
/// Native DE/rand/1 with immediate selection; `differential_evolution(&mut f,&lower,&upper,&config)`.
pub fn differential_evolution(
    function: &mut impl MultipleVarFunction,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    config: &GlobalConfig,
) -> VectorResult {
    let bounds: BoxBounds<'_> = BoxBounds { lower, upper };
    if !valid(&bounds, config)
        || !config.mutation_scale.is_finite()
        || config.mutation_scale < 0.0
        || !config.crossover_prob.is_finite()
        || config.crossover_prob < 0.0
        || config.crossover_prob > 1.0
        || config.population < 4
        || config
            .population
            .checked_mul(lower.length() as usize)
            .is_none()
    {
        return failure_result(Status::InvalidInput);
    }
    let mut checked: Checked<'_, _> = Checked {
        function,
        failure: Status::Ok,
    };
    let mut population: Population = Population::new(&mut checked, &bounds, config);
    if population.best.is_none() {
        return failure_result(if checked.failure != Status::Ok {
            checked.failure
        } else {
            Status::NumericalError
        });
    }
    iterate_population(&mut checked, &mut population, &bounds, config)
}
fn iterate_population(
    function: &mut impl MultipleVarFunction,
    population: &mut Population,
    bounds: &BoxBounds<'_>,
    config: &GlobalConfig,
) -> VectorResult {
    let mut trial: Vector<'static> = Vector::with_size(bounds.lower.length() as usize);
    let mut result: VectorResult = failure_result(Status::MaxIterations);
    for iteration in 0..config.solver.max_iterations {
        result.iterations = iteration + 1;
        population.evolve(function, bounds, config, &mut trial);
        if population.converged(config.solver.tolerance) {
            result.status = Status::Ok;
            break;
        }
    }
    let mut best: Vector<'static> = population.points[population.best.unwrap()].clone();
    let mut value: f64 = population.best_value.unwrap();
    if config.polish_budget_per_dim > 0 {
        polish_coordinate_wise(
            function,
            &mut best,
            &mut value,
            bounds.lower,
            bounds.upper,
            config.solver.tolerance,
            config.polish_budget_per_dim as usize * trial.length() as usize,
            &mut 0,
        );
    }
    result.solution = Some(best);
    result.value = Some(value);
    result
}
