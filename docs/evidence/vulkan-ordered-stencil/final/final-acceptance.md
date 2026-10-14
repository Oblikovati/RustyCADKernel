# Martensite ordered stencil handoff

The full original 87-case renderer family is ready for coordinator review and
integration. This is renderer behavior evidence, without new API/class credits.
Source adopts checked main `026ed7ad7`; the final compiler source is preserved by
`af6732821`, with later commits containing additional evidence only.

All 174 final native/Rust comparisons pass across genuine Mesa 25.2.8 software
and AMD profiles, with two runs per backend/profile: 696 successful processes.
RGBA and all eight final state fields match exactly. Maximum depth difference is
`1.4901161193847656e-8`, within the unchanged finite absolute `1e-6` gate.
All repeated buffers/state and native traces match byte-for-byte. Actual storage,
mask-bit, point/line/text, filtering, capping/layer, and view/group lifecycle
proofs remain in the attributed raw archive and independent Rust fixtures.

Fresh extracted packages pass 717 Vulkan tests across 75 executables on each
already-admitted Mesa 26 profile; five inherited exclusions are unchanged.
The stencil-specific extracted suite passes 18 tests per genuine Mesa 25 profile,
including all 87 native fixtures, physical rejection and lifecycle repair. These
Rust runs never use OCCT. Existing FFP/OIT tests reject Ubuntu2 Mesa25 before
rendering because their older admitted25 cohort is Ubuntu4: those rejection logs
are preserved, and no identity predicate was weakened.

Deterministic archive mtimes exposed a stale reused benchmark executable. The
exact archive comparison proved that its CLI was the only changed source member;
all dependency crate sources were identical. The extracted Vulkan package was
explicitly cleaned and rebuilt. The accepted fresh run demonstrably executes
seven benchmark tests, including the new `--verify-only` argument regression.
Earlier six-test runs are retained and excluded from final-source acceptance.

All-target/all-feature Clippy with warnings denied, shared-plan shader variants
and controls, 129 Vulkan tool tests, included-leaf formatting and discovery pass.
The final32 package archives plus support, source equality and VCS identities are
retained separately. Native compiler/library/source/executable hashes, the valid
Erase→Remove lifecycle, and earlier invalid teardown/text diagnostics remain
attributed. No failure was hidden or tolerance widened.

The independently audited optimized executable verifies all 87 inputs without
measuring, then runs exactly 20 warmup and 100 measured redraws per input/profile.
Both measurement processes exit0; all 17,400 samples are finite and positive,
with strict native buffer/state checks before and after. Whole-process peak RSS
is 186,576 KiB on lavapipe and 72,116 KiB on AMD. GPU memory is unmeasured.
All 47,513 source and 123 frozen artifact hashes match before and after.

These measurements are the first contextual desktop baseline. Root observed
background kernel I/O pressure near60% before authorizing the run; CPU/memory
pressure and competing compilation were absent. `baseline_accepted=false` means
there is no prior comparative or exclusive bare-metal performance acceptance.
The raw vectors, GNU time/RSS, identities, exact commands and host counters are
preserved. Samples measure steady redraw/readback wall time; setup, native
comparison and teardown are outside samples but included in process peak RSS.
No performance improvement or comparative device equivalence is claimed.

Windows/macOS, application acceptance and inherited undefined/raster/text/depth
exclusions remain separate. The frozen bundle is outside this removable checkout:
`/tmp/martensite-ordered-stencil-performance-af6732821`. Root owns final merge,
combined checks and physical checkout/branch deletion.
