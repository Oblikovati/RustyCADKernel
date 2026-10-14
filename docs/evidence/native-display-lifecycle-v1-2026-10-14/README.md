# Retained native display lifecycle v1 receipt

## Source and decision

Final source: **cc310d9745c17769dec11452ce305f5b2bd90a04**, based on integrated
main `4757e8b35`. Owned unleased fixtures were committed first (`da38095c0`),
then the native-call injection/refactor (`24fd13951`), then the lifecycle change
(`a823a26c5`). The final commit removes its obsolete renderer metadata-trait
import. [ADR0019](../../adr/0019-retained-native-display-connections.md) records
behavior v1, the legacy adapter migration/deletion task and scope.

Reset admission and consumer acquisition share the connection lock. Owned/wrapped
windows and Vulkan runtimes retain explicit leases. XwWindow native operations,
deletion-atom lookup and renderer admission use the original binding despite
replacement of display metadata. Explicit expose overrides retain their argument
through the send callback. Unleased fixtures preserve prior behavior and hazards.

## Accepted checks

Executed on Linux x86-64, Rust 1.99.0, with `CARGO_BUILD_JOBS=2`. Native X11 display
`:1` was available. Driver identities in every native run identify AMD Radeon
8060S (RADV STRIX_HALO) and llvmpipe (LLVM 20.1.2), Mesa 26.1.6. The runner records
exact validation-layer/library paths and ICD environment per process.

| Gate | Result |
| --- | --- |
| TKService all targets/all features | 333 passed, zero failed/ignored |
| Vulkan all targets/all features, AMD | 749 passed, zero failed, five existing ignored |
| Vulkan all targets/all features, lavapipe | 749 passed, zero failed, five existing ignored |
| Typed renderer error tests | Three passed; also included in each complete Vulkan suite |
| Named fake display admission/ownership under Miri | 12 passed |
| Python release-plan tests | Eight passed |
| TKService/Vulkan doctests | One compile-fail doctest passed; Vulkan has zero |
| Clippy, selected libraries and new native example | Passed with `-D warnings` |
| Workspace formatting check | Passed |
| Native replay | All 21 processes exited zero |
| Exact independent framebuffer checker | Passed |

The 21 native cases comprise the unchanged prior 15 GUI/window cases and three
new display-lifetime processes per driver. Each new process checks owned/wrapped
window counts, invalid constructor inputs, reset refusal with exact unchanged
view captures, optional GUI composition, same-window rebind with substituted
metadata, runtime retention after view removal, and successful reset after runtime
destruction. Validation error lists must be empty. This is bounded kernel evidence.

The unchanged framebuffer checker compares seven rapid processes against exact
analytic RGBA/depth/PPM expectations at eight stages, and compares the full GUI
overlay byte for byte across drivers. No baseline or tolerance was changed.

All **47,871 tracked file hashes** and all **four native executable hashes** match
before/after the native sequence and after the complete Vulkan matrix. Snapshots
include the native-window default seam, Xw event handling and release-plan files.
The subsequent evidence commit adds documentation/artifacts only.

## Reproduction commands

From the repository root, build the four examples before replay:

```sh
CARGO_BUILD_JOBS=2 cargo +1.99.0 build -p martensite-vulkan --locked \
  --example native_gui_rapid --example native_gui_acceptance \
  --example native_window_lifecycle --example native_display_lifecycle
python3 docs/evidence/native-display-lifecycle-v1-2026-10-14/run_native_checks.py NEW_OUTPUT_DIR
python3 docs/evidence/gui-framebuffer-restart-2026-10-14/check_framebuffers.py NEW_OUTPUT_DIR/native
CARGO_BUILD_JOBS=2 cargo +1.99.0 test -p rustycad-tkservice --all-targets --all-features --locked
CARGO_BUILD_JOBS=2 cargo +1.99.0 clippy -p rustycad-tkservice -p martensite-vulkan --lib --example native_display_lifecycle --locked -- -D warnings
CARGO_BUILD_JOBS=2 cargo +1.99.0 test -p rustycad-tkservice -p martensite-vulkan --doc --locked
cargo +1.99.0 fmt --all --check
CARGO_BUILD_JOBS=2 cargo +nightly-2026-09-25 miri test -p rustycad-tkservice --lib --locked -- display_connection:: --skip an_unreachable_display
PYTHONPATH=tools python3 -m unittest discover -s tools/tests -p test_release_verification_plan.py
```

For each driver's recorded environment, the complete renderer command is:

```sh
CARGO_BUILD_JOBS=2 cargo +1.99.0 test -p martensite-vulkan --all-targets --all-features --locked
```

`vulkan-runs.json` records the two exact commands, environment and exit codes.
Miri selected the fake display boundary and ordinary host-thread admission tests;
the native unreachable-display test was exercised in the normal TKService suite,
not Miri. `LD_PRELOAD`, `MIRIFLAGS` and `RUSTFLAGS` were unset. Two inherited
TKMath future-name-collision warnings remain in the Miri log. Miri/host-thread
results do not certify concurrent native view, Xlib or Vulkan operations; ThreadSanitizer
was not run, and concurrent-native acceptance is not claimed.

## Preserved diagnostics and artifacts

- `accepted-cc310d974.tar.gz`: exact images, native logs/commands, source/executable
  bookends, independent checker result, complete renderer suites and final CPU logs.
- `development-diagnostics.tar.gz`: prior structural fixture runs, red/green
  admission and metadata tests, initial native runs, unsuccessful invocations and
  compiler findings. These runs occurred during development; they are not relabeled
  as execution of the final source.
- `archive-sha256.json`: archive integrity hashes.
- `run_native_checks.py`: reusable 21-case runner; uses the existing 15-case runner
  without modifying it and refuses an already-existing output directory.

The admission regression first failed because reset returned success with a lease.
The metadata regression first observed native address 55 instead of retained 41.
Both signals were preserved and corrected. Also retained: a malformed test-filter
invocation; initial Clippy rejection of an oversized unboxed native error payload;
and Clippy's obsolete import finding at `a823a26c5`. Earlier two-driver native
passes preceded the metadata correction and are diagnostic only. They have no
independent frozen source revision and are not final acceptance evidence.

The historical display-reset baseline remains unchanged in
`../native-window-lifecycle-2026-10-14/baseline-diagnostics.tar.gz`. This bounded
correction addresses retained connection reset and metadata substitution; it does
not attribute or establish a cure for the original intermittent BadWindow.

Excluded: external close/reset of borrowed or independently aliased raw displays,
other protected native metadata mutation (including XID/visual/configuration),
unreported window destruction, explicit-override same-server XID validity, event
delivery guarantees, and native/application concurrency. Existing unleased
self-alias and failed-reset hazards remain explicit in ADR0019. Shared-host elapsed
times are diagnostic only. No performance, current full application acceptance,
OCCT compatibility/oracle, workspace version change or publication is claimed.
