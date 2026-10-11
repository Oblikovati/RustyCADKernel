# Martensite AIS keyboard navigation integration evidence

This slice implements 16 captured AIS_ViewController declarations: keyboard
forwarding and shared cache exports, six-axis navigation fetch, event timing,
next-frame requests, focus/reset and the upstream empty window callbacks.
The class and constructor remain partial. Mouse/touch/3D-mouse dispatch,
window-listener integration, animation, camera/render dispatch and XR stay open.
No API exclusion or denominator change was introduced for this slice.

The source reference is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Two local native runs reproduced all 206 frozen-time snapshots byte-for-byte.
Committed fixture provenance records source, generator and asset hashes. Sixteen
new Rust regressions replay the snapshots using named injected clocks; CI needs
no OCCT headers, compiler or runtime. Tests cover all axes, opposite keys,
pressure/duration, jump/crouch/run precedence, thrust activation/release,
negative and nonfinite values, signed zeros, focus and click-timer preservation.

All 531 TKV3d tests pass on the feature checkout and integrated main. Clippy with
warnings denied, canonical toolkit discovery and Rust formatting pass. The
renderer is unchanged; its paired-device capture/performance gates remain in CI.
See [implementation notes](../../../docs/ais-view-controller-keyboard.md).

The feature commit is integrated directly into main. Its clean completed checkout
and branch were deleted after ancestry, source equality and process-reference
checks, reclaiming 2.71 GiB. No archive was made. The integration receipt and
compressed logs retain the checks and fresh compiler-backed inventory results.

The refreshed crosswalk uniquely binds all 84 controller claims (82 implemented
and two partial class/constructor claims), with zero unresolved ledger rows.
Port-scope completed APIs rise from 52,261 to 52,277; open APIs fall from 22,803
to 22,787, with 977 partial APIs unchanged. TKV3d has 2,656 done declarations
out of 5,172 (51.4%), 2,512 open and four partial. Compiler configuration, identity
and mapping proof gaps still prevent a full-completion claim.
