# Checked GUI surface outcomes v2 evidence

Measured source: `89d2c0a1e8b82bbad69de4c2ea62c5031bc24b32`, Rust 1.99.0,
Linux x86_64 kernel `7.1.5-76070105-generic`, XWayland `DISPLAY=:1`.
The source behavior is opt-in; existing constructors select LegacyV1.
See [ADR0028](../../adr/0028-typed-gui-surface-outcomes.md) and the
[machine summary](summary.json).

## Provenance and executed gates

- Owned pre-change fixtures: `a4694ac6c`; structural extraction: `9aea1015c`.
  The committed legacy baseline and structural logs remain revision-bound.
- Final full renderer command on each driver: `CARGO_BUILD_JOBS=2 cargo +1.99.0
  test -p martensite-vulkan --all-targets --all-features --no-fail-fast --
  --test-threads=1`: **836 passed, zero failed, six ignored per driver**.
  The new ignored native phase gate was then explicitly executed on each driver;
  the other five existing ignores remain excluded.
- Twelve native cases per driver: Acquire, BeforeSubmit, Present and Publication,
  each with resize, resize plus injected primary fault, and same-extent injected
  fault. Every failure retires the chain; the same presenter then recovers with
  exact full green RGBA/depth and no validation errors. These injected faults do
  not claim actual device-loss generation or arbitrary concurrent operation safety.
- Existing native window and display lifecycle examples pass unchanged on both
  drivers. Full GUI acceptance passes on both, including additive CheckedV2
  public capture-before-redraw refusal, observed away/back invalidation, fresh
  packet recovery and mapped/unmapped routing cases.
- Five AMD rapid processes (Legacy indices 0,2; Checked indices 1,3,4) and two
  lavapipe processes (Legacy 0; Checked 1) pass the unchanged independent checker:
  eight exact analytic RGBA/depth/PPM stages per process. Full GUI overlay bytes
  match across drivers. This is completed framebuffer evidence under ADR0016,
  not compositor/display visibility evidence.
- Two separate host-test processes produce identical canonical bytes; each
  compares independent 8-thread × 16-operation results with its single-thread
  fixture. Native operation concurrency is not inferred from these CPU tests.
- Final all-target/all-feature Clippy with `-D warnings`, documentation with
  `RUSTDOCFLAGS='-D warnings'`, formatting, and eight release-plan tests pass.
  The doctest command succeeds with **zero executable doctests**.
- Both full source bookends match; the supplemental source bookends match;
  the five native example binaries are unchanged before/after native checks and
  timing. Sources include all crate/build files and release-plan/ADR notes.

The pinned validation library SHA-256 is
`89e618526d6d1532096dbbd62ee1dc710dda647061cd3b7b66b542dc7dbce104`.
Exact ICD/library hashes, renderer device identities, validation paths, commands,
source snapshots, samples and process memory are in `raw.tar.gz`.
Both `VK_DRIVER_FILES` and `VK_ICD_FILENAMES` select the recorded driver.
No new OCCT oracle process was run; existing retained renderer assets are replayed.

## Preserved failures and limits

The first final checker invocation failed **before reading any image**: its
`*-rapid-*` glob also selected seven sibling `.log` files. That exit-1 log is
retained. A checker-only symlink input directory fixes the invocation; the checker
and captures are unchanged. `capture-bookends.json` records all nine exact link
targets and SHA-256 for **174 capture files before and after** a passing checker
run. All hashes match. The archive excludes the ephemeral absolute symlink view;
its targets and mapping suffice to recreate it. The corrected reusable runner
creates relative links and preserves the same oracle.

Development compile/import/lint failures and pre-final native runs are retained
with their own logs/source snapshots. They are not relabelled as final evidence.
[ThreadSanitizer remains incomplete](TSAN.md): instrumented startup control built
but crashed before main; the GUI sanitizer suite was neither built nor executed.

Application command-loop acceptance (Oblikovati#3630), controlled previous-release
performance, physical visibility, unreported native mutation races, display reset
outside admitted lifetimes, and the historical intermittent BadWindow remain
outside this acceptance. No release, default-policy change or migration deletion
is claimed. Main integration and coordinator replay remain required.

## Paired timing observations

Same frozen debug/unoptimized executable, existing seven workloads, 20 warmups
and 100 samples each, validation disabled during timing. Every operation completed
successfully; timing followed functional acceptance. Values are median milliseconds:

| Workload | AMD Legacy | AMD Checked | lavapipe Legacy | lavapipe Checked |
|---|---:|---:|---:|---:|
| alpha-order | 0.873 | 1.011 | 1.133 | 1.187 |
| clip | 0.796 | 1.055 | 1.007 | 1.063 |
| font-retire | 0.846 | 1.024 | 0.996 | 1.011 |
| model | 0.679 | 0.700 | 0.958 | 1.181 |
| replace | 0.806 | 0.958 | 1.128 | 1.501 |
| resize | 1.800 | 1.935 | 7.296 | 7.794 |
| static | 0.803 | 0.960 | 1.451 | 2.428 |

Peak process RSS (KiB): AMD Legacy **68,988**, Checked **68,732**; lavapipe Legacy
**108,520**, Checked **107,368**. RSS is for the complete seven-case process,
not a per-frame allocation measurement. Raw `/usr/bin/time -v` output is retained.
One-minute host load during these pairs was approximately 8.6–8.8; longer load
averages retained earlier shared-host contention. The debug profile, shared host,
fixed run ordering and absence of an independently built previous-release pair
preclude controlled release performance conclusions. These observations expose
costs; they do not establish a speedup, performance acceptance or regression fix.

## Reproduction

With the coordinator's exclusive GPU lease, from a checked source checkout:

```sh
python3 docs/evidence/gui-surface-outcomes-v2-2026-10-14/replay.py   target/gui-surface-outcomes-replay
```

The output directory must be new. The runner uses Rust 1.99.0/jobs 2, the recorded
host ICDs and pinned validation layer path, builds before running, hashes current
sources, runs both driver sequences, and gates timing on successful checks.
The original runner and corrected-input supplement are retained in the archive;
`replay.py` incorporates only the diagnosed input-directory correction and a fresh
output-directory argument. It is a runner for the recorded checks, not a new oracle.
