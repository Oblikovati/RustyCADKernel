# Martensite readback diagnosis

The original failed acceptance receipt remains `../vulkan-surfaces-main-2026-10-10.json`.
No baseline or timing limit was changed. The six before runs include one AIS
median failure; all six uninstrumented after runs pass the original limits.

`backend-phase-results.json` and `backend-phase-after.json` contain five phase
wall times per frame, raw samples, whole-cycle summaries, selected memory types,
driver identity and binary/patch hashes. Phase medians are separate distributions,
not additive components of a selected sample. Logs are outside the measured
readback phase, but instrumentation can affect total latency. They are diagnostic,
not acceptance receipts. `regression-red.log` records the preference test failing
under the old policy; `regression-green.log` records all six policy tests passing.

To reproduce the before probe, use an isolated checkout of `31a0abccd`, copy this
folder's files to `target/diagnostics`, apply `backend-phase-probe.patch`, and copy
`readback_phase_probe.rs` to `crates/vulkan/examples/readback_phase_probe.rs`.
Build with `cargo build --release -p martensite-vulkan --example readback_phase_probe`
using the checkout's own default target directory. Then run
`python3 target/diagnostics/measure-backend-phases.py` from the checkout root.

For the fixed revision, apply `instrument-fixed.patch` instead, build the same
example, and run `python3 target/diagnostics/measure-backend-after.py`.
`backend-phase-after.patch` retains the exact tracked working diff hashed by the
historical after receipt; it includes the allocator call-site changes relative
to the pre-fix revision and is not the standalone instrumentation patch.

For the uninstrumented loop, restore the production files, build the same example
without either instrumentation patch, and run
`python3 target/diagnostics/run-isolation-after.py`. This command fails if any of
six runs exceeds the original AIS median/p95 limits. Both Rust source copies are
retained because formatting changed their bytes; the receipt's source hash refers
to `ais_timing_isolation.rs`, and `readback_phase_probe.rs` is the formatted build
input with the same behavior. Remove the temporary example and instrumentation
after measurement. Production source contains no diagnostic logging.
