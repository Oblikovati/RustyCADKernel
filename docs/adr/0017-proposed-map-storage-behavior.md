# Proposed versioned Map storage for callback traversal

Status: **proposed, unaccepted**. Recorded 2026-10-14. This document authorizes no
production storage change, default switch, or baseline replacement. Martensite
`main` at `80457d007` retains its existing vector-backed Map behavior.

## Capability and dependency boundary

The experimental controller work needs a key to keep its allocated address while
callbacks insert or remove other keys, a cursor to follow the original bucket
allocation after ownership exchange, and a clear operation that releases caller
field guards before running key destructors and allocator callbacks. It also
needs resource selection after key destruction and rejection of stale clear
traversals. These are concrete lifetime capabilities; an upstream API's behavior
alone does not establish that Martensite must change its storage policy.

The experiment is preserved on `porting/martensite-controller-frame-events` at
`5b6d414f4`, in `../Martensite-controller-frame-events`. Its provenance is
`0b4e0fbe2` (stable original Map nodes and tables), `6bec3c193` (clear failure
metadata and bounded key capture), and `5b6d414f4` (scoped retirement, allocator
selection, and stale traversal rejection).

A two-endpoint audit against `80457d007` found 38 changed TKernel files. Five
standard value-reference files are independent of Map. The remaining 33 files
form the Map storage, cursor, clear, and test closure; they are **not imported** by
this decision proposal. Main had no TKernel changes after the two branches'
merge-base. Neither controller nor service code is part of this proposal.

The experiment's shared `release_node_using` helper has only one consumer on
current main: Sequence's existing current-resource release implementation.
Extracting that unchanged body alone provides no production consolidation, so
this proposal leaves it in place.

## Observable behavior that would change

| Operation | Current vector Map | Experimental allocated-node Map |
| --- | --- | --- |
| Add a unique key | Vector owns the key; injected allocator is retained as identity | Allocates a typed node through the injected allocator; virtual Allocate can execute |
| Remove, clear, or destroy a key | Drops vector-owned payload without virtual node Free | Destroys payload and calls the selected allocator's Free; callbacks and failures become observable |
| Reset an injected incremental arena while Map keys are live | Succeeds because Map keys hold no typed arena lease | Refuses while actual node leases remain; succeeds after nodes are cleared |
| Mutate unrelated entries | Existing vector movement rules apply | Existing node addresses stay stable across vector-index changes |
| Interrupt a callback-driven clear | Existing public Map contract applies | New progress API exposes partial traversal, preserves failure causes, and rejects tokens from earlier traversals |

The earlier reset-success baseline is valid. Main stores keys in a `Vec<K>` and
does not allocate their storage from the injected arena. Reset therefore cannot
invalidate those keys. Calling the experimental refusal a main-branch soundness
fix would misstate the ownership model. The experiment does need refusal for its
own typed node allocations, but selecting that allocation policy is a behavior
change. Virtual Allocate/Free side effects are another supported observable
change, including new allocation failure and callback reentrancy paths.

The existing test
`map_canonical_storage_contracts::panic_cleanup::replacement_allocator_clears_actual_old_nodes_first`
retains its reset-success assertion unchanged. No native fixture was generated or
updated for this proposal.

## Proposed decision requiring acceptance

Preserve the current vector policy as behavior version 1. If the callback
capability is accepted, introduce allocated stable nodes as an explicitly chosen
behavior version 2, with a documented constructor or policy setting. The final
API and version identifier require review. Record the selected behavior in replay
inputs and retain version 1 until a recorded migration; absence of a recorded
choice must continue to select the existing behavior.

An opt-in API can be released as an additive feature with an appropriate release
version and changelog. Changing the default or removing version 1 requires a
breaking release (proposed next minor series `0.3.0` while the crate is pre-1.0),
explicit migration notes, and a reviewed baseline update. No such version bump
or migration is approved by this document.

Before accepting version 2, assign a replacement/deletion task and a migration
milestone with at most one milestone of overlap. Prove a complete callback
operation, retain independently expected lifetime and failure fixtures, and
measure allocation count, copy cost, lock cost, runtime, and peak memory on the
same accepted workloads. Current Miri results do not establish performance,
lock-free published results, or full controller acceptance. Published operation
results must satisfy Martensite's immutable ownership contract independently of
these mutable container mechanics.

## Independent Rust evidence

The owned tests in
[`allocator_policy.rs`](../../crates/tkernel/tests/map_canonical_storage_contracts/allocator_policy.rs)
pin the existing policy without an OCCT oracle:

- A Map containing an owned String permits arena reset and retains the key.
  A separate typed arena value blocks reset; dropping that value permits reset
  again while the Map key remains live. This distinguishes actual leases from
  allocator identity using public Rust operations.
- A named fake allocator counts virtual calls over insert, duplicate insert,
  replacement, resize, remove, retained clear, reinsertion, and destruction.
  Current main performs zero virtual node Allocate and Free calls. A control test
  allocates and drops a real typed value through the fake and observes one
  Allocate and one Free, proving the instrumentation is active.

The same test source was replayed unchanged against experimental TKernel
`5b6d414f4` from a temporary external Cargo harness. Both tests failed as expected:
reset refused with a live Map key, and the second test observed four allocations
where the existing policy requires zero. This is evidence of a policy difference,
not an acceptance failure to hide by editing the baseline.

Validation uses Linux x86_64, stable Rust 1.99.0, and Miri
`nightly-2026-09-25` (`f7575a9da8`, default checks, no disabled alias or leak checks).
Commands and final results are recorded below. No C++ capture, GPU run,
ThreadSanitizer run, or controller/API parity claim is included.

| Revision and check | Command | Result |
| --- | --- | --- |
| Main `80457d007`, before adding evidence | `cargo +1.99.0 test -p rustycad-tkernel --test toolkit map_canonical_storage_contracts --locked` | 26 passed; original reset baseline unchanged |
| Main plus the three owned policy tests | `cargo +1.99.0 test -p rustycad-tkernel --locked` | 5,323 passed, including 4,443 toolkit tests and doctests |
| Main plus owned policy tests, Miri | `cargo +nightly-2026-09-25 miri test -p rustycad-tkernel --test toolkit map_canonical_storage_contracts::allocator_policy --locked` | 3 passed, including the fake allocation/free control |
| Experiment `5b6d414f4`, clear ownership | `cargo +nightly-2026-09-25 miri test -p rustycad-tkernel --lib ncollection::map::clear_ --locked` | 7 passed |
| Experiment `5b6d414f4`, raw key access and thread migration | `cargo +nightly-2026-09-25 miri test -p rustycad-tkernel --lib ncollection::slot_chains::stable_access --locked` | 5 passed |
| Identical owned policy source against experiment | External harness with a path dependency on experiment TKernel; `cargo +1.99.0 test --manifest-path target/map-policy-replay/Cargo.toml --offline` | Fake control passed; both policy expectations failed, as described above |

The identical policy test files in the integration candidate and experimental
replay had SHA-256
`923d627376d8cf642b2c3546a072ad1b005cd2c761a7faad3982c217fd34ef6f`.
The replay harness is temporary build evidence, not a production fallback. It
contains the unchanged `allocator_policy.rs` as its only integration test and
sets `rustycad-tkernel` to the preserved experimental crate by path. Its purpose
is to reveal the supported behavior difference; it must not replace the main
policy gate with an expected failure.
