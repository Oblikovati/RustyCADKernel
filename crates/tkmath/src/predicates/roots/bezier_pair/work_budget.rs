// SPDX-License-Identifier: GPL-2.0-only
//! One execution owns its split counter; failed and cancelled stages still spend completed work.
use super::super::RootIsolationOptions;
use std::cell::Cell;
pub(super) struct PairWorkBudget {
    requested: u16,
    completed: Cell<u16>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PairWorkReceipt {
    pub requested: u16,
    pub completed: u16,
}
impl PairWorkBudget {
    pub fn new(requested: u16) -> Self {
        Self {
            requested,
            completed: Cell::new(0),
        }
    }
    pub fn effective(&self, mut options: RootIsolationOptions) -> RootIsolationOptions {
        options.max_bisections = self.requested - self.completed.get();
        options
    }
    pub fn completed_split(&self) {
        let previous = self.completed.get();
        assert!(
            previous < self.requested,
            "completed splits {previous}; expected below requested {}",
            self.requested
        );
        self.completed.set(previous + 1);
    }
    pub fn receipt(&self) -> PairWorkReceipt {
        PairWorkReceipt {
            requested: self.requested,
            completed: self.completed.get(),
        }
    }
}
