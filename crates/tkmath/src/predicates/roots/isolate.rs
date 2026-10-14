// SPDX-License-Identifier: GPL-2.0-only
use super::{DyadicBound, RootBracket, RootIsolation, RootIsolationError, RootIsolationOptions};
use super::{
    chain::{Endpoint, SturmChain, root_count},
    error::poll,
};

struct RootCell {
    lower: Endpoint,
    upper: Endpoint,
    count: usize,
}
impl RootCell {
    fn new(lower: Endpoint, upper: Endpoint) -> Result<Self, RootIsolationError> {
        let count = root_count(&lower, &upper)?;
        Ok(Self {
            lower,
            upper,
            count,
        })
    }
    fn finished(&self, width: &DyadicBound, previous: Option<&RootBracket>) -> bool {
        self.count == 1
            && (self.upper.root
                || (!self.lower.root
                    && previous.is_none_or(|root| root.upper < self.lower.bound)
                    && self.upper.bound.difference(&self.lower.bound) <= *width))
    }
}
pub(super) fn isolate(
    tower: &[SturmChain],
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    isolate_accounted(tower, bounds, width, options, cancelled, &mut || {})
}
// Captured operations charge real completed splits even if a later poll fails.
// The compatibility entry point above intentionally retains its original result.
pub(super) fn isolate_accounted(
    tower: &[SturmChain],
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<RootIsolation, RootIsolationError> {
    let mut work = SubdivisionWork::new(tower, width, options.max_bisections);
    work.seed(bounds, cancelled)?;
    work.run(cancelled, completed_split)
}
struct SubdivisionWork<'a> {
    tower: &'a [SturmChain],
    width: DyadicBound,
    limit: u16,
    pending: Vec<RootCell>,
    result: RootIsolation,
}
impl<'a> SubdivisionWork<'a> {
    fn new(tower: &'a [SturmChain], width: DyadicBound, limit: u16) -> Self {
        Self {
            tower,
            width,
            limit,
            pending: Vec::new(),
            result: RootIsolation {
                roots: Vec::new(),
                bisections: 0,
            },
        }
    }
    fn seed(
        &mut self,
        bounds: [DyadicBound; 2],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), RootIsolationError> {
        let first = initial_cell(&self.tower[0], bounds, cancelled)?;
        if first.lower.root {
            self.result
                .push(self.tower, &first.lower, &first.lower, cancelled)?;
        }
        self.pending.push(first);
        Ok(())
    }
    fn run(
        mut self,
        cancelled: &mut impl FnMut() -> bool,
        completed_split: &mut impl FnMut(),
    ) -> Result<RootIsolation, RootIsolationError> {
        while let Some(cell) = self.pending.pop() {
            poll(cancelled)?;
            if cell.count == 0 {
                continue;
            }
            if cell.finished(&self.width, self.result.roots.last()) {
                self.result
                    .push(self.tower, &cell.lower, &cell.upper, cancelled)?;
                continue;
            }
            self.split(cell, cancelled)?;
            completed_split();
        }
        Ok(self.result)
    }
    fn split(
        &mut self,
        cell: RootCell,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), RootIsolationError> {
        if self.result.bisections == self.limit {
            return Err(RootIsolationError::SubdivisionLimit { limit: self.limit });
        }
        let middle =
            self.tower[0].endpoint(cell.lower.bound.midpoint(&cell.upper.bound), cancelled)?;
        // Right goes on the stack first: output order follows exact parameter order.
        self.store_cell(middle.clone(), cell.upper)?;
        self.store_cell(cell.lower, middle)?;
        self.result.bisections += 1;
        Ok(())
    }
    fn store_cell(&mut self, lower: Endpoint, upper: Endpoint) -> Result<(), RootIsolationError> {
        let cell = RootCell::new(lower, upper)?;
        if cell.count > 0 {
            self.pending.push(cell);
        }
        Ok(())
    }
}
fn initial_cell(
    chain: &SturmChain,
    bounds: [DyadicBound; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootCell, RootIsolationError> {
    let lower = chain.endpoint(bounds[0].clone(), cancelled)?;
    let upper = if bounds[0] == bounds[1] {
        lower.clone()
    } else {
        chain.endpoint(bounds[1].clone(), cancelled)?
    };
    RootCell::new(lower, upper)
}
impl RootIsolation {
    fn push(
        &mut self,
        tower: &[SturmChain],
        lower: &Endpoint,
        upper: &Endpoint,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), RootIsolationError> {
        let lower_bound = if upper.root {
            &upper.bound
        } else {
            &lower.bound
        };
        let multiplicity = root_multiplicity(tower, lower_bound, &upper.bound, cancelled)?;
        self.roots.push(RootBracket {
            lower: lower_bound.clone(),
            upper: upper.bound.clone(),
            multiplicity,
        });
        Ok(())
    }
}
fn root_multiplicity(
    tower: &[SturmChain],
    lower: &DyadicBound,
    upper: &DyadicBound,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<usize, RootIsolationError> {
    let mut multiplicity = 1;
    for chain in &tower[1..] {
        if !contains_root(chain, lower, upper, cancelled)? {
            break;
        }
        multiplicity += 1;
    }
    Ok(multiplicity)
}

fn contains_root(
    chain: &SturmChain,
    lower: &DyadicBound,
    upper: &DyadicBound,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<bool, RootIsolationError> {
    let first = chain.endpoint(lower.clone(), cancelled)?;
    if lower == upper {
        return Ok(first.root);
    }
    Ok(root_count(&first, &chain.endpoint(upper.clone(), cancelled)?)? != 0)
}

#[cfg(test)]
#[path = "isolate_accounting_tests.rs"]
mod accounting_tests;
