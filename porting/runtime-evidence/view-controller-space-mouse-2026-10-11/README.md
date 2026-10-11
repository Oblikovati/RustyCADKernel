# Martensite AIS SpaceMouse integration evidence

This slice implements the full captured AIS_ViewController::Update3dMouse API,
sharing native translation, rotation and button routing through Martensite's
narrow fallible keyboard interface. Existing AspectWindowInputListener methods
retain their signatures and callbacks through a blanket adapter; the controller
uses the same dispatch directly without incomplete window callbacks.

The oracle is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. Two local
native runs reproduced 288 snapshots byte for byte. They record report bytes,
return values, decoder range, modifier state, every cached HID button and every
virtual key's pressed/free/held status, duration and pressure. Scenarios cover
combined/separate packets, six device products, idle and unknown reports, repeated
presses and releases, quadric/linear pressure, rotation policy gates, skipped and
reversed axes, and zero/negative/infinite/NaN acceleration.

Eight new Rust regressions verify the complete committed native trace, callback
bypass for translation, rotation error propagation, skipped axes, button-cache
mutation before failure, repeated presses, one-time releases, release errors and
shared routing through a window-listener trait object. Named callback targets and
frozen/injected event time require no external I/O, sleeps or native OCCT runtime.
All 795 local toolkit tests pass: 244 TKService and 551 TKV3d. Clippy with warnings
denied, formatting, canonical suite discovery and 13 workflow/suite tests pass.

The feature commit is integrated into local main and verified again. After clean
checkout, ancestry and process-reference checks, the merged worktree and branch
were deleted, reclaiming 2.68 GiB. No archive was made. This slice was not pushed;
the user's 2026-10-11 instruction keeps further porting local. Windows/macOS
validation is deferred. Renderer code is unchanged; local lavapipe and native-host
functional/capture/performance harnesses remain available for rendering changes.

The full controller class/constructor, camera/window application, redraw and XR
remain partial/open. No scope exclusions were added. See the
[implementation notes](../../../docs/ais-view-controller-space-mouse.md) and receipt.

Fresh compiler inventory uniquely binds all 100 controller claims: 98 implemented
and two partial record/constructor claims. Port-scope done APIs rise from 52,292
to 52,293; open APIs fall from 22,772 to 22,771, with 977 partial APIs unchanged.
TKV3d has 2,672 done declarations out of 5,172 (51.7%), 2,496 open and four partial.
There are zero unresolved ledger rows; explicit compiler configuration/identity/
mapping proof gaps still prevent full completion. The previously pushed mouse
slice's [Linux CI run](https://github.com/Oblikovati/RustyCADKernel/actions/runs/38165936898)
completed successfully in all six jobs. This newer SpaceMouse slice remains local.
