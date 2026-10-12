# Martensite temporary frame assignment

`Graphic3dFrameStatsDataTmp::assign_data` copies a completed statistics frame's
rates, counters and accumulated timer values. It preserves the temporary frame's
previous samples and actual OSD timer objects. Resetting those extra fields on
assignment would change the next sample's delta or stop a running measurement.

The local OCCT probe at revision `3d097a0328e71b826377d4814ab05ec3c3d23871`
captures six rows: the running destination, first assignment, next delta,
second assignment, final averaged sample, and reset. It compares the actual
timer object's identity and running state without asserting wall-clock elapsed
time. Two executions produced byte-identical assets. Rust replays all recorded
values with exact floating-point bit comparisons.

The final average retains a previous sample of 11, so the next accumulated
value 16 contributes a delta of 5. Assignment must not reset that history.
The existing Rust implementation passes this behavior; the new replay supplies
native evidence for its inventory binding rather than replacing that body.

Run the replay and related toolkit checks with:

```sh
cargo test -p rustycad-tkservice --all-features --all-targets
```

The oracle and provenance are under `tools/oracle` and
`porting/runtime-evidence/frame-stats-assignment`. OCCT is only needed to
regenerate assets. This does not prove move assignment, generic vector storage
aliases, GPU performance or whole-class completion. Safe Rust's unique mutable
borrow prevents passing the destination itself as an aliased source; no native
exception or alternate self-alias API is claimed.
