# Martensite AIS mouse integration evidence

This slice implements four captured AIS_ViewController declarations: the live
mouse gesture map and its mutable accessor, UpdateMouseButtons and
UpdateMousePosition. Class/constructor claims remain partial. Camera/window
application, redraw, XR and 3D mouse integration remain open. No exclusions or
scope denominator changes were added.

The oracle is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. Two local
native runs reproduced 1,220 mouse transitions byte for byte, covering every
gesture, emulated tolerances (including zero, negative, infinity and NaN), policy
gates, button priority, strict click thresholds, double-click counters/timers,
selection completion, modifier changes, idle-walk rotation, pan accumulation,
zoom coalescing and queued drag/highlight requests. Provenance records source,
generator and fixture hashes. Rust tests use a frozen injected clock and require
no OCCT installation.

Six new Rust regressions replay every owned state/queued request snapshot and
cover map isolation, exact double-click interval boundaries, NaN interval
semantics, failed animation-pause ordering, hover suppression and coalesced zoom
return values. All 550 TKV3d tests pass on the feature checkout and integrated
main. Clippy with warnings denied, formatting, canonical toolkit discovery and
13 workflow/suite regression tests pass locally. The renderer is unchanged.
Rendering functional/capture and performance validation remains available on
lavapipe and the native host. Windows/macOS validation is deferred.

The feature commit is integrated directly into main. The clean completed worktree
and branch were deleted after ancestry and process-reference checks, reclaiming
2.66 GiB. Nothing was archived. The receipt and compressed logs preserve the
integration checks. The prior Linux-only CI run
[38164188237](https://github.com/Oblikovati/RustyCADKernel/actions/runs/38164188237)
completed successfully, including Rust, coverage, Vulkan, tools, inventory and
dependency checks. See [implementation notes](../../../docs/ais-view-controller-mouse.md).

Fresh compiler inventory uniquely binds all 99 controller claims: 97 implemented
and two partial record/constructor claims. Port-scope done APIs rise from 52,288
to 52,292; open APIs fall from 22,776 to 22,772, with 977 partial APIs unchanged.
TKV3d has 2,671 done declarations out of 5,172 (51.6%), 2,497 open and four partial.
There are zero unresolved ledger rows. Explicit compiler configuration/identity/
mapping proof gaps still prevent full completion.
