# Pcurve import diagnosis

Base: `77e631344`. Owned V1 baseline: `adcb2c311`. Structural plumbing:
`031b1e132`. No production OCCT execution.

1. The frozen projection's original built binary failed in 0.09 s with
   `GraphAudit`, CoEdge index 5, `CoEdgeDef has no Curve2D representation`.
2. An independent TKBO test imported the actual retained prism directly and
   reproduced that issue. Retained `PrismImageRole::Last` identifies the issue's
   owning face. The last cap alone fails at CoEdge index 0.
3. Removing region construction, copying, sweeping, cubic geometry and seam
   topology leaves one authored line edge/pcurve on a planar face. Translation
   plus `create_auto_product=false` is sufficient. Default auto-product import
   normalizes the root placement and does not reproduce.
4. Ranked hypotheses were relayed before probes: missing ancestor placement;
   transformed surface-handle mismatch; extraction-to-registration loss;
   erroneous reversed seam selection.
5. The source pcurve matches the original surface handle at identity, but V1
   expects the face translation relative to the local identity edge. A single
   variable probe applied the face placement to the lookup edge. Both minimized
   and original prism/cap audit regressions passed. This confirms the first
   hypothesis; the other three are not needed to explain the defect.
6. The probe was reverted. The owned V1 baseline was committed and passed before
   structural plumbing. New V2 tests were run red with legacy lookup, then green
   after cumulative face/wire/edge lookup. No debug instrumentation remains.

The original reproducer can now be run on V2 with:

```sh
CARGO_BUILD_JOBS=2 cargo +1.99.0 test -p rustycad-tkbo --lib sketch_region_prism::graph_import_tests
CARGO_BUILD_JOBS=2 cargo +1.99.0 test -p rustycad-tkbrep --test toolkit graph_pcurve_placement
```

The compound repeated-occurrence test initially expected two face definitions.
The documented importer instead strips child placement into two compound
references sharing one definition. The final independent expectations preserve
that existing contract; the shell case separately requires two located face
definitions. No topology-sharing production change was made.

See ADR0026 for the additive behavior contract, explicit rebuild requirement,
exclusions and migration/deletion gate. Raw diagnosis logs are retained in the
verification archive; timing above describes these small diagnostic runs only.
