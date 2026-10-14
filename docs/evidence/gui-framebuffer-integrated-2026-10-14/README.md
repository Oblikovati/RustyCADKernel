# Integrated completed-framebuffer GUI checks

Measured source: `c47191e0492c715e24410b999e547ef500854750` on Linux x86_64,
Rust 1.99.0. The [receipt](receipt.json) records commands, environment, raw
artifact hashes and source snapshots. All 15,526 tracked crate/build inputs and
both example binaries remained unchanged throughout the native runs.

Five AMD and two lavapipe rapid processes passed exact independent full-frame
RGBA/depth/PPM comparisons across eight stages. The full native GUI example
also passed on both drivers. Twelve framebuffer-helper tests passed on that
revision. Eight release-plan tests passed at `cb0b7464c`; their initial invocation
without `PYTHONPATH=tools` failed during import and is retained separately.

Extract `raw.tar.gz` to recover the `gui-integrated-c47191e04` directory,
including the runner, source snapshots, all captures and command output.
The archived checker invocation is reproducible with the committed
`../gui-framebuffer-restart-2026-10-14/check_framebuffers.py`.

These are completed renderer framebuffer checks under ADR 0016. They do not
establish compositor behavior, resolve the preserved intermittent BadWindow
failure, supply current performance measurements, or complete the consuming
application gate. No new OCCT oracle was executed. Earlier failures retain
their original evidence and source revisions.
