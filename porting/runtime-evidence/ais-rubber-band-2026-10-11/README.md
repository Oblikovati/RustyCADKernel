# Martensite rubber-band runtime evidence

Pinned OCCT 3d097a0 compared against source revision b0474326b.
162,208 exact numeric comparisons include 2,520 rubber-band requests.
Both device profiles pass all 96 visibility/filling roles; their 48 strict image
comparisons remain failed with zero depth differences. Boundary line coverage
and boundary blending remain raw failures under the deferred pixel-row policy.
No strict renderer parity or accepted performance baseline is claimed.
All 33 timing workloads, raw samples and device fingerprints are retained.
The initial default-layer-only fixture mistake is retained separately; the
corrected fixtures capture TopOSD through the full-view capture path.

The tar contains captures only. No checkout or build cache is archived.

`performance-repeat.json` compares all 33 identical device/workload configurations
against the initial measurement with median ratio 1.25 and p95 ratio 1.5. The new
rubber-band workload passes both budgets on both devices. Host passes all budgets;
lavapipe retains a window-resize-present median failure. These are diagnostic repeats,
not an accepted overall renderer baseline.
