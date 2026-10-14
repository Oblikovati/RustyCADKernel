// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use bounds::{IntegralBounds, interval_bounds};
use restriction::Interval;
use std::collections::BinaryHeap;

pub(super) struct Execution {
    pub options: PolynomialSideOptions,
    pub admitted: bool,
    pub work: PolynomialSideWork,
    pub diagnostic: Option<[f64; 2]>,
}
impl Execution {
    pub fn new(options: PolynomialSideOptions) -> Self {
        Self {
            options,
            admitted: false,
            work: PolynomialSideWork::default(),
            diagnostic: None,
        }
    }
    pub fn failure(&self, cause: PolynomialSideFailure) -> PolynomialSideError {
        PolynomialSideError {
            requested_options: self.options,
            effective_options: self.admitted.then_some(self.options),
            work: self.work,
            diagnostic_enclosure: self.diagnostic,
            cause: Box::new(cause),
        }
    }
    pub fn integrate(
        &mut self,
        controls: Vec<Vector>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<CertifiedPolynomialSideIntegral, PolynomialSideFailure> {
        let root = self.initial(controls, cancelled)?;
        let mut frontier = Frontier::new(root);
        loop {
            checkpoint(cancelled)?;
            if let Some(report) = self.completion(&frontier.tight, cancelled)? {
                return Ok(report);
            }
            frontier.refine(self, cancelled)?;
        }
    }
    fn initial(
        &mut self,
        controls: Vec<Vector>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Interval, PolynomialSideFailure> {
        self.interval(
            controls,
            ExactRatio::from_finite(0.),
            ExactRatio::from_finite(1.),
            cancelled,
        )
    }

    fn interval(
        &mut self,
        controls: Vec<Vector>,
        start: ExactRatio,
        width: ExactRatio,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Interval, PolynomialSideFailure> {
        let bounds = interval_bounds(&controls, &width, cancelled)?;
        self.work.interval_bounds += 1;
        let gap = bounds.upper.subtract(&bounds.lower);
        Ok(Interval {
            controls,
            start,
            width,
            bounds,
            gap,
        })
    }
    fn children(
        &mut self,
        parent: &Interval,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<[Interval; 2], PolynomialSideFailure> {
        let [first, second] = restriction::bisect(&parent.controls, cancelled)?;
        let width = parent.width.multiply(&ExactRatio::from_finite(0.5));
        let start = parent.start.add(&width);
        Ok([
            self.interval(first, parent.start.clone(), width.clone(), cancelled)?,
            self.interval(second, start, width, cancelled)?,
        ])
    }
    fn completion(
        &mut self,
        bounds: &IntegralBounds,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Option<CertifiedPolynomialSideIntegral>, PolynomialSideFailure> {
        let (enclosure, error) = bounds.publish()?;
        self.diagnostic = Some(enclosure);
        if error <= self.options.absolute_error_limit {
            checkpoint(cancelled)?;
            return Ok(Some(self.published(enclosure, error)));
        }
        self.require_remaining_work(error)?;
        Ok(None)
    }
    fn published(&self, enclosure: [f64; 2], error: f64) -> CertifiedPolynomialSideIntegral {
        CertifiedPolynomialSideIntegral {
            enclosure,
            error,
            options: self.options,
            work: self.work,
        }
    }
    fn require_remaining_work(&self, error: f64) -> Result<(), PolynomialSideFailure> {
        if self.work.subdivisions == self.options.max_subdivisions {
            return Err(PolynomialSideFailure::WorkLimit {
                requested: self.options.max_subdivisions,
                achieved_error: error,
                error_limit: self.options.absolute_error_limit,
            });
        }
        Ok(())
    }
}

struct Frontier {
    total: IntegralBounds,
    tight: IntegralBounds,
    queue: BinaryHeap<Interval>,
}
impl Frontier {
    fn new(root: Interval) -> Self {
        Self {
            total: root.bounds.clone(),
            tight: root.bounds.clone(),
            queue: BinaryHeap::from([root]),
        }
    }
    fn refine(
        &mut self,
        execution: &mut Execution,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), PolynomialSideFailure> {
        let parent = self
            .queue
            .pop()
            .expect("one complete interval covers the domain");
        let [first, second] = execution.children(&parent, cancelled)?;
        self.total = self
            .total
            .replace(&parent.bounds, &first.bounds, &second.bounds);
        self.tight = self.tight.intersect(&self.total);
        execution.work.subdivisions += 1;
        self.queue.extend([first, second]);
        Ok(())
    }
}
