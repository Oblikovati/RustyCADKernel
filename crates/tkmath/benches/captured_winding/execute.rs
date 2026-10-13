// SPDX-License-Identifier: GPL-2.0-only
use super::cases::{OwnedWindingSegment, WindingReuseWorkload};
use rustycad_tkmath::predicates::roots::bezier_line::*;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindingWorkLedger {
    pub new_bisections: u32,
    pub reused: usize,
    pub historical_bisections: u64,
    pub effective_budgets: [u16; 4],
    pub capture_budget_count: usize,
}
pub fn run_baseline(workload: &WindingReuseWorkload) -> WindingWorkLedger {
    let mut work = 0u32;
    for reverse in [false, true] {
        let segments = borrowed_occurrences(workload, reverse);
        let query = BezierWireQuery {
            segments: &segments,
            line: workload.line,
            options: workload.options,
        };
        let result = classify_bezier_wire(query, || false).unwrap();
        assert_eq!(result.location(), &workload.expected[usize::from(reverse)]);
        work += u32::from(result.bisections());
    }
    baseline_ledger(work)
}

pub fn run_captured(workload: &WindingReuseWorkload) -> WindingWorkLedger {
    let mut store = CapturedBezierWireStore::new(workload.line, workload.options);
    let mut historical = 0;
    let mut reused = 0;
    let mut actual = 0;
    for reverse in [false, true] {
        let occurrences = owned_occurrences(workload, reverse);
        let result = store.classify(&occurrences, || false).unwrap();
        assert_eq!(result.location(), &workload.expected[usize::from(reverse)]);
        historical += result.work().certificate_bisections;
        reused += result.work().reused_certificates;
        actual = result.work().operation_bisections;
    }
    captured_ledger(&store, actual, historical, reused)
}

fn captured_budgets(store: &CapturedBezierWireStore) -> [u16; 4] {
    let mut budgets = [0; 4];
    for (index, proof) in store.proofs().iter().enumerate() {
        budgets[index] = proof.effective_policy().max_bisections;
    }
    budgets
}
fn traversal(workload: &WindingReuseWorkload, reverse: bool) -> Vec<&OwnedWindingSegment> {
    let mut segments: Vec<_> = workload.segments.iter().collect();
    if reverse {
        segments.reverse();
    }
    segments
}
fn borrowed_occurrences(
    workload: &WindingReuseWorkload,
    reverse: bool,
) -> Vec<BezierWireSegment<'_>> {
    traversal(workload, reverse)
        .into_iter()
        .enumerate()
        .map(|(index, source)| BezierWireSegment {
            id: index as u64 + if reverse { 100 } else { 0 },
            controls: &source.controls,
            weights: source.weights.as_deref(),
            interval: [0., 1.],
            reversed: reverse,
        })
        .collect()
}
fn owned_occurrences(
    workload: &WindingReuseWorkload,
    reverse: bool,
) -> Vec<CapturedBezierWireOccurrence> {
    traversal(workload, reverse)
        .into_iter()
        .enumerate()
        .map(|(index, source)| CapturedBezierWireOccurrence {
            id: index as u64 + if reverse { 100 } else { 0 },
            reversed: reverse,
            input: CapturedBezierWireInput::capture(
                source.physical,
                &source.controls,
                source.weights.as_deref(),
                [0., 1.],
            ),
        })
        .collect()
}

fn baseline_ledger(work: u32) -> WindingWorkLedger {
    WindingWorkLedger {
        new_bisections: work,
        reused: 0,
        historical_bisections: u64::from(work),
        effective_budgets: [0; 4],
        capture_budget_count: 0,
    }
}
fn captured_ledger(
    store: &CapturedBezierWireStore,
    actual: u16,
    historical: u64,
    reused: usize,
) -> WindingWorkLedger {
    let effective_budgets = captured_budgets(store);
    WindingWorkLedger {
        new_bisections: u32::from(actual),
        reused,
        historical_bisections: historical,
        effective_budgets,
        capture_budget_count: store.proofs().len(),
    }
}
