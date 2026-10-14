# Delivered native destruction and explicit rebind, behavior v1

Source **0849fbaaf**, [decision and migration](../../adr/0018-delivered-native-window-destruction.md).
The [earlier baseline](../native-window-lifecycle-2026-10-14/README.md) preserves
fatal `X_GetWindowAttributes` evidence separately from the production correction.
No result here attributes or claims to cure the original intermittent `BadWindow`.

## Accepted bounded behavior

A delivered destruction event retires the destroyed XID (not its parent), records
original identity/cause/version, and sends the close callback once. Duplicate
events and removal are idempotent. Closed targets refuse capture, redraw and GUI
operations with typed operation-specific diagnostics, including an offscreen
attempt. Public resize and destruction no longer request the retired XID.

Explicit rebind invalidates the presenter, capture and prior GUI packet even with
the same handle and extent. It always advances the publication generation, also
without GUI. A named injected backend drives actual `set_window` and publication
in a deterministic ordering test: an old pending result is refused, and a new
frame succeeds. This is not native/view concurrency coverage.

## Final checks

- 15 native processes: five AMD rapid, two lavapipe rapid, full native GUI
  acceptance on each driver, and three lifecycle processes per driver.
- Unchanged independent checker: exact analytic RGBA, depth and PPM bytes for
  all eight rapid stages in all seven processes; full GUI overlay bytes match
  across drivers.
- `gui_` tests on each driver: 9 library, 21 toolkit and 4 example tests passed.
- TKService closure tests 3, event tests 4, typed lifecycle tests 2; renderer error
  tests 2; release verification plan tests 8. Format, clippy and example builds pass.
- All 15,539 tracked source/build hashes and all three executable hashes match
  before and after. Full commands, selected ICDs and device identities are retained.

## Retained failures and archives

[receipt.json](receipt.json) identifies revisions, checks, exclusions and archive
hashes. Extract archives with `tar -xzf ARCHIVE -C DESTINATION`.

- `accepted-images.tar.gz`: final `gui-lifecycle-0849fbaaf` images, logs, commands,
  comparison and hash bookends.
- `pre-fix-images.tar.gz`: `gui-lifecycle-6e50a8f76` outputs. These native checks
  passed, but review then found the pending model-only publication defect. Their
  status explicitly excludes acceptance of that rebind contract.
- `diagnostic-iterations.tar.gz`: baseline diagnostics, first implementation logs,
  initial missing-import build failure, release-plan count failure, wrong test
  target invocation, and the decisive publication red/green regression. None is
  replaced by the later passing runs.

The native check sequence can be rerun from the repository root after building
`native_window_lifecycle`, `native_gui_rapid`, and `native_gui_acceptance`:

```sh
python3 docs/evidence/native-window-lifecycle-v1-2026-10-14/run_native_checks.py target/NEW-INTEGRATED-RECEIPT
python3 docs/evidence/gui-framebuffer-restart-2026-10-14/check_framebuffers.py target/NEW-INTEGRATED-RECEIPT
```

The runner requires a new output directory and does not overwrite an earlier
attempt. It pins the same display, validation-layer installation and ICDs used
here. The archive retains the exact original runner; this review copy only adds
the output-directory argument. Record integrated source/build hash bookends
separately; these frozen source captures must not be relabelled as main evidence.

## Exclusions

Unreported external destruction, shared-display reinitialization and historical
intermittent `BadWindow` remain open. This receipt establishes no current
performance, downstream application, native/view concurrency or release gate.
The host was shared and busy; elapsed values are timeout context, not performance
measurements. No new OCCT oracle or global Xlib error handler was introduced.
