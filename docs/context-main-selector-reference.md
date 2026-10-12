# Martensite MainSelector reference

`AisInteractiveContext::main_selector()` now returns `AisMainSelectorRef` rather
than a lock-bearing `AisContextHandle`. The new return value keeps a noncounting
storage lease of the manager selected at the getter call and refers to that
manager's actual private selector slot. Replacing the context manager leaves the
original reference bound to its original slot, as OCCT's `mgrSelector->Selector()`
requires. Direct selector member calls remain available through `Deref`.

The shared `HandleReference` interface identifies the actual slot and observes
its current target without increasing native ownership. Use `clone_handle()`
when an owning selector snapshot is needed. Every context read guard is released
before returning the alias or invoking selector operations.

The pinned CPU oracle retains two real selection managers and probes context
replacement, nullification and restoration. Four committed rows include exact
slot/object identity and manager/selector intrusive counts; two native runs are
byte identical. Rust replays those rows and verifies that a context write lease
is available while the selector observation remains alive. This checkpoint also
adds `SharedState::try_write`, tested against active reader/writer leases and
poisoned-state recovery, so the callback regression fails immediately if a
container read lock is accidentally retained.

The native pin, source/executable hashes and schema are recorded in
`crates/tkv3d/tests/fixtures/context-main-selector-reference-provenance.json`.
The test replay needs no OCCT libraries or graphics device. Native dangling
manager-reference behavior is excluded; the oracle retains both real managers.
This bounded reference correction adds no context-class or controller-frame
completion claim.
