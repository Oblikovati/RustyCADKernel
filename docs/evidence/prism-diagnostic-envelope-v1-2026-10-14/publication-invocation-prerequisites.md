# Sealed invocation adapter prerequisites

The coordinator authorized dependency `6ed91a0be` after reviewing the actual
packet issuer in `a3b345b7e`: private fields, actual publication result, same
invocation request/plan snapshots and actual borrowed binding storage. This
initial authorization was scoped to packet provenance; broader semantic-map
review was handled separately. Merge `6f80dd01a` preserved both module-registration
sets and passed all-target/all-feature typechecking, formatting and 137
owned-request tests.

`61a240222` makes the separately authorized actual invocation dispatch choose the
reviewed V2 producer only for retained request version 2. All other requests use
the unchanged V1 admission. Thirty resolver tests and strict all-target/all-feature
Clippy passed. The first run is retained: a new test incorrectly expected the
child class Cancelled. Existing publication/source.rs maps the first nested
Embedding cancellation to Incomplete; the corrected fixture now asserts that
unchanged class, the full Embedding(CurrentBinding)->Binding(Cancelled) cause,
and the distinct outer native-callback Failed class. No production classification
was changed to satisfy this expectation. Actual successful V2 still issues the
same 13-member/26-use semantic domain. No detached packet constructor was added.

The naming owner authorized `d5fe68290`, a separate shared-codec extraction.
The full authored-member encoder is renamed/reexported without changing its body;
existing diagnostic native/class/phase/callback encoders are exposed/reused.
Fifteen diagnostic codec tests and strict Clippy pass. The semantic-map replay is
exactly its prior 10,571 bytes (`ffbdcd9461a1acb2349d514610a6f24b6cdadd03b460bf9394a65240b88247ff`),
and the full typed preparation diagnostic remains its integrated 5,807 bytes
(`a27a9465306496b6d8fa5c732e80bc39e7a2976aa6a074ae67612bddfad196a8`).

These are incremental prerequisite checks, not a relabeling of the earlier
source-bookended producer gate. Full adapter and final combined source gates
remain pending. The archive includes the failed probe, all focused logs and
exact before/after payloads. TSan remains incomplete under the recorded host
startup failure; no unchanged-host repeat, native OCCT or GPU operation was run.
