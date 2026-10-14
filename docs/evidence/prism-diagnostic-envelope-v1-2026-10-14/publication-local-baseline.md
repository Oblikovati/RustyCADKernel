# Publication local-failure baseline for ADR0048

Base: `bd5bffc35b91a07d70594ddb9b4536c2e97e6399`. Only test registration and
`publication/diagnostic_baseline_tests.rs` were added. Rust 1.99, jobs 2,
ordinary host, no native OCCT or GPU.

Three new independently expected tests pin callback precedence for all original
child classes, exact callback phase/ordinal and native failure retention, and
unchanged child headers when no callback stopped. V1 `finish_child` drops a
child's phase/class while preserving its cause; the new projection must not
retroactively infer those absent fields. Existing publication suite: 33 passed.

A fresh full publication replay remains 2,202,990 bytes, SHA-256
`accba93e30c455d9f888a96c5b70f684a1dbacbbf3afec9076de3544c7a13107`, identical
to the integrated V1 receipt. The archive retains commands/logs, exact replay,
and matching before/after hashes for 12,856 source/build files. Baseline tests
ran before any producer or diagnostic behavior change.

ThreadSanitizer remains incomplete under the already recorded host startup
failure. This test-only prefix does not repeat that unchanged-host control or
claim a new sanitizer, release-performance or application acceptance gate.
