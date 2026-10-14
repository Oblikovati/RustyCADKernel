# Completed GUI framebuffer captures after desktop restart

Source: `f4f714f7aea6b060b081833a290900f952fb9d70`, Linux x86_64,
Rust 1.99.0. This records the framebuffer acceptance decision in
[ADR 0016](../../adr/0016-framebuffer-images-for-rendering-acceptance.md).
There is no renderer behavior change, new expected baseline, or OCCT comparison.
The previous diagnostic files are retained without replacing their failures.

`raw.tar.gz` contains both `current/` (the earlier gate at `3cf4ed814`) and
`restarted/` (this run). `receipt.json` records the source, executable hashes,
validation manifest, commands, outcomes, independent expectations and exclusions.
Raw RGBA, little-endian float depth, PPM images, logs and invocation failures
are retained. These artifact directories had no files overwritten by this run.

## Current functional evidence

Five separate AMD Radeon 8060S/RADV rapid processes and two separate lavapipe
rapid processes passed on the restarted Xwayland `:1` desktop, with pinned
validation enabled. Each process checks the full 1440×900 framebuffer at eight
stages: blue, green, four completed GUI frames, clear and restoration. A GUI
packet mutation invalidates the previous capture; removal rejects capture.
The first GUI frame is checked before later redraws can conceal a stale frame.

`check_framebuffers.py` independently constructs the constant backgrounds,
300×110 rectangle at `[8,308) × [8,118)`, depth `1.0` and P6 image bytes. All
seven processes match those expected bytes at all stages. This also establishes
cross-process and cross-driver identity for these workloads. Substituting the
green image for the first GUI frame makes that checker fail at the exact file;
`restarted/independent-stale-rejection.log` retains that negative check.

The full native GUI acceptance executable passed on both drivers: replacement,
clear, resize, alpha/UV, font retirement, selection identity, model blend modes
and model alpha scope. The captured overlay matches across drivers in RGBA,
depth and PPM. The targeted suites passed per driver: 7 library, 19 toolkit and
4 filtered example tests. All 10 rapid example helper tests passed separately;
the 8 release-verification-plan tests also passed.

Reproduce the image check after extracting `raw.tar.gz` into a temporary directory:

```sh
python3 check_framebuffers.py /path/to/extracted/restarted
```

The runtime command is `target/debug/examples/native_gui_rapid OUTPUT_DIRECTORY`.
Use `DISPLAY=:1` and the validation environment in `receipt.json`, selecting
the recorded RADV or lavapipe ICD explicitly. Build with
`cargo +1.99.0 build -p martensite-vulkan --example native_gui_rapid
--example native_gui_acceptance --locked`. Run GUI suites with
`cargo +1.99.0 test -p martensite-vulkan --lib --test toolkit
--example native_gui_rapid --locked gui_`, using the same driver environment.
Run Python regressions with `PYTHONPATH=tools python3 -m unittest discover
-s tools/tests -p test_release_verification_plan.py` from the repository root.

## Unresolved lifecycle failure and exclusions

The earlier first AMD rapid invocation exited 1 after only the blue and green
captures. Xlib reported `BadWindow`, request `X_GetWindowAttributes`, resource
`0x1000001`, request serial 81/current serial 82. Its log and partial captures
remain at `current/amd/rapid-a/`. Later successful processes do not explain or
fix this failure. Five later invocations failed device enumeration with
`ERROR_INITIALIZATION_FAILED` after the desktop disappeared; those logs and
all run exits remain at `current/amd/repetition-*` and `current/runs.json`.

The diagnosing-bugs feedback command reaches that same rapid workload and
retains a failing process exit. Five consecutive current AMD processes passed,
so the original lifecycle failure could not be reproduced or minimized on this
restarted desktop. No cause has been established and no lifecycle fix is claimed.
Further diagnosis requires a reproducing session or a captured X11 lifecycle
trace around the failing request; framebuffer success alone cannot provide it.

No performance measurement was made: concurrent cargo/rustc/llvm-cov work
prevented a quiet host reservation. Invocation wall times in the receipt describe
test execution only. Earlier performance receipts remain historical and do not
establish performance for this source. Multi-thread determinism, thread sanitizer,
other platforms, extracted packages and application acceptance were not evaluated
by this run. This evidence does not close those gates or the lifecycle failure.
