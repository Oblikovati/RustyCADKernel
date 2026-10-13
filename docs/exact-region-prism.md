# Constructed cubic region and prism evidence

This is a diagnostic for [kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).
It joins the exact wire classifier from ADR 0010 to actual constructed pcurves.
It does not add a production region contract or change mapped OCCT behavior.

## Complete representative path

The source is one closed polynomial cubic with control points `(0,0)`, `(4,0)`,
`(0,4)`, `(0,0)` on the XY plane. The test-only nonempty builder hook retains
filler errors and uses the existing General Fuse stages. Both directed edge
occurrences enter the existing planar face builder. No sampled face classifier
is used by the test to select its resulting bounded component.

The test reads the actual pcurve, parameter range, and edge direction from each
constructed face. It checks the polynomial representation and every control
coordinate before passing them to the exact wire classifier. Both resulting
wires contain `(1,1)`: their winding numbers are `+1` and `-1`. For this known
simple cubic, the positive orientation identifies the bounded component.
Nonzero winding alone would accept both. This fixture-specific proof is not a
general region assembly policy for holes, intersections, or arbitrary planes.

The selected region produces a solid through the existing prism builder with
displacement `(0,0,2)`. Its three faces match independent OCCT 8.0.0 measurements
for area, centroid, surface kind, wire count, edge count, and mapped geometric
validity. Source-edge, start-cap, and end-cap history cover each face exactly
once. The solid has two vertices, three edges, three faces, and one shell;
its Euler characteristic is two. Volume 4.8 is an additional check.

The exact seed corpus has two inside, three outside, and two boundary points.
The latter include the closed curve's shared vertex, which the native face
classifier misclassifies in the [wire evidence](exact-bezier-wire.md).
Generated checks repeat construction, classification, history, and per-face
measurements for 17 power-of-two scales from `2^-8` to `2^8`, with both input
edge directions: 34 variants. Native metrics are normalized by the corresponding
length, area, or volume scale. The comparison bound remains `1e-8` in normalized
fixture units; it is not an achieved modelling tolerance. These variants do not
prove arbitrary near-degenerate input handling.

## Ownership result

Non-destructive General Fuse preserves this unchanged source edge by sharing
it. Adding the region pcurve then changes the caller's original edge: its curve
representation count increases from one to two. The retained Rust and OCCT
receipts agree. This is an expected mapped ownership behavior, not evidence of
immutable extension inputs.

A second test copies topology and geometry with the existing copy builder before
arrangement. Its copy history maps the original edge to exactly one copied edge.
Construction leaves the original representation count at one; the copied edge
has two. Changing the original cubic's second pole to `(8,0,0)` does not change
the copied region: exact seed tests, geometric validity, and the complete prism
checks still pass. The native copy probe independently confirms isolated curve
representations and pole mutation. This supports an explicit copy boundary for
this workload; it is not a proof of all reachable caches, meshes, failures, or
cancellation paths, nor a storage redesign or performance claim.

## Remaining acceptance work

`BRepCheckAnalyzer` is mapped validity evidence, not the ordered strict
extension report. Self-intersection and tolerance-consistency coverage,
measured achieved error, complete edge/vertex history, persistent split names,
source isolation on failure and cancellation, and an owning extension ADR
remain required. General Fuse and the face builder's internal numerical policy
have not been certified by the exact post-construction query.

Arbitrary trims can end at algebraic intersections. Rounded trim parameters
must not be treated as exact shared endpoints. Analytic conic extraction and
multi-wire assembly also remain open. The application still pins v0.2.1 and
cannot consume these local exact predicates. No release or dependency change is
part of this evidence. Production recognizers, tolerance constants, fallback
sites, and type assertions each have net change zero.

## Reproduction

Source: local Martensite base `359939144`; native OCCT source `3d097a0328`.
Platform: local Linux x86-64. Native code is only an independent test generator.

```sh
cargo test -p rustycad-tkbo --test toolkit exact_region_prism
g++ -std=c++17 -I../occt-install/include/opencascade \
  crates/tkbo/tests/exact_region_prism/occt_oracle.cpp \
  -L../occt-install/lib -Wl,-rpath-link,../occt-install/lib \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/region-prism-occt
LD_LIBRARY_PATH=../occt-install/lib /tmp/region-prism-occt
```

The retained CSV includes both signed region reports. The unbounded complement
fails the mapped geometric checker; the bounded component and all prism faces
pass. Test role matching uses operation history, not face order or proximity.

## Local checks

The full TKBO suite passes: 39 unit tests, 646 integration tests, and 35
documentation tests, with no skipped tests. The five new integration tests also
pass after the final source changes. Clippy with all targets and features,
strict Rustdoc, Rust formatting, Markdown lint, SPDX/file-size checks, and
`git diff --check` pass. The native generator reproduces its CSV byte-for-byte.

The instrumented five-test run covers 371 of 372 lines in the four new Rust
test modules (99.73%); the remaining line is the failed interior-seed assertion.
This measures the new test bridge, not whole-kernel production coverage.
Duplication across the new fixture sources is zero. All production source is
unchanged. No application gate, live application test, remote push, or release
was performed for this kernel test-only change.
