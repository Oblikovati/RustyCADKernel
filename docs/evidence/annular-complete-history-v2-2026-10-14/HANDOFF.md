# Annular history structural checkpoint and deferred proof

## Status at requested wrap-up

**Complete annular history is UNIMPLEMENTED.** This candidate adds two actual
annular native-census fixtures and shares existing V1 history projection helpers.
It adds no V2 history issuer, no successful annular history result, no exhaustive
annular original-continuation/deletion proof, and no body-validity authority.
The coordinator explicitly closed this work at its coherent structural boundary
when the user asked current workers to finish and hand off after pausing the goal.

The additive private changes do not change the workspace version. V1 entry points,
classifications, callback order, native causes, and complete replay payloads remain
unchanged. A future behavior implementation must remain separate from this prefix
and pass its own independent corpus and integration gate. ADR0058 describes that
planned V2 contract; it is not evidence of implemented acceptance.

## Commits and ownership

- `fb49a31ce`: eight owned baseline payloads and the proposed ADR0058 contract.
- `b3f7a8763`: two actual retained annular native-census tests.
- `bd41ee8d5`: shared native input view, scalar/context/budget/sweep helpers;
  reviewed by the coordinator without an ordering or behavior finding.
- `154dca444`: shared candidate/generated/partition metadata and native diagnostic
  helpers, with original V1 wrappers preserved.
- `b6dc60301`: small responsibility-based helper/test splits; no new behavior.

Source branch: `kernel/prism-annular-history`. The coordinator owns main merge,
final integration checks and worktree/branch deletion. Do not use a deleted
checkout or depend on its target artifacts when resuming.

## Independently pinned actual topology/history census

The existing general pipeline constructs two closed polynomial source edges,
selected annulus and two rejected one-wire domains. The fixture asserts:

- Two original/input-copy receipts, each with three directed contexts and one
  actual native copier image per query.
- Six native original partition member queries; Modified and Generated are
  explicitly empty, Deleted explicitly false, and history is requested/available.
- Native source-definition count four, two source rows and two physical vertices.
- Candidate copier has seven source/copied/query contexts and seven member links.
- Three generated candidate context sizes are 5, 5 and 9; selected receipt retains
  nine relations and the receipt retains two rejected histories.
- Nine native sweep queries, seven false retirements, two actual retired Wire
  queries and two corresponding native retirement rows.

These are observations and combinatorial expectations. In particular, the empty
input-deletion vector has not become a certified annular deletion conclusion;
intermediate wire retirement has not been converted into original deletion.

## Reusable implementation seams

All paths below are under `crates/tkbo/src/sketch_region_prism/frozen/`.
The `complete_history` helpers expose narrow borrowed operation-local inputs.
Those references do not prove isolation or Send/Sync of native owners. There is
no new public owner-bearing API or alternate history engine.

- `native_inputs.rs`: `NativeHistoryInputs` borrows bridge, receipt and explicit
  publication bindings. It carries no proof authority.
- `contexts.rs`, `context_names.rs`, `copied_contexts.rs`: actual full directed
  ancestry, authored member/use bijection, and native copier correspondence.
- `parent_rows.rs`: original keys and relation classes preserve parent multiplicity.
- `budget.rs`: checked native operand accounting, including nested observations;
  a future caller still needs to charge its new scope/name and current-census rows.
- `candidate_copy.rs`: existing physical copied-tree and member-link projection.
- `generated.rs`, `relations.rs`: complete generated-member/descendant projection
  and exact candidate relation comparisons over supplied candidate operands.
- `partition_metadata.rs`: unchanged source metadata and native endpoint-role
  checks; the V1 wrapper retains its one-source admission.
- `partition_queries.rs`: native error preservation and unchanged partition query
  checks; V1 orchestration still selects its single source.
- `sweep.rs`: complete native query projection with existing exact multiset
  correspondence, including false retirement and empty answer observations.
- `diagnostics.rs`: original native completion/cause order and separate borrowed
  local diagnostics remain available when owner-bearing causes cannot be published.

The final-source test-size audit counts physical executable body lines excluding
multiline signatures and exterior braces. All 24 changed Rust files comply with
20-line bodies; the largest file is 365 lines. No comments/provenance were removed.

## Exact next behavior implementation (not started)

1. Define a private bounded V2 request over the **same** retained
   `TwoLoopRegionRequest`, `PrismReceipt`, explicit publication/copy/intermediate
   names and sealed `AnnularFrozenDomain`. Keep input diagnostic lifetimes separate
   from the borrowed join lifetime. Reuse `PrismHistoryLimits` with checked aggregate
   admission before any new allocation. Do not widen existing freeze limits.
2. Own source-specific Original/InputCopy scopes, operation-level Partition and
   CandidateCopy scopes, authored-candidate-specific selected/rejected scopes and
   SweepBase. Require exact scope/member/use bijections, including unused-extra
   refusal. Source IDs and candidate IDs are authored lookup bindings, never
   recovered from parent sets, coordinates, addresses or returned order.
3. Reproject the actual initial-copy and partition query chain for both sources.
   Prove each query targets its own native copied image; bind continuation to that
   exact directed context. V1 `relations::validate_partition` filters by shape kind
   and cannot be reused unchanged for plural sources: it would mix outer and inner
   same-kind parents. Match actual source/context continuation instead.
4. Bind all three actual candidates and four wire dispositions against the sealed
   region proof and actual batch, including selected annulus, rejected inner domain
   and rejected outer complement. Reuse actual generated snapshots, copied tree,
   all member relations, ancestor paths and exact class multiplicities. Preserve
   distinct intermediates even when their original-parent sets are equal.
5. Compose existing selected-copy/sweep lineage with exhaustive native sweep
   answers. Require multiset bijections for every answer, not equal total counts.
   Bind class-specific parents and the **same full source/output/role row** to
   `AnnularPrismBinding.parents().captured`; preserve its flat original order.
6. Prove all six original directed-use continuations/deletions through the complete
   candidate disposition and sweep chain. Record the two intermediate Wire
   retirements separately. Source-key the native closure witnesses from the two-loop
   proof; the annular join has no singleton contact identity. Do not invent one.
7. Retain complete joined bytes plus all new owned historical operands, settings,
   measured units and conclusions in deterministic replay. Expose borrowed views
   only; do not expose a raw successful-history constructor. After the last external
   callback, perform the shared live annular join rebind exactly once, followed by
   callback-free history-only reprojection and complete byte comparison. The latter
   must recapture extra rejected/intermediate/query observations absent from join.
8. Add independent actual-operation and adversarial fixtures before acceptance:
   duplicate/missing scopes and same-cardinality query omissions; same-parent
   distinct intermediates; source/endpoint-parent swaps; native empty/false/error
   distinctions; exact/one-short/overflow budgets; final callback/source mutations;
   success/failure/cancellation isolation and reuse; Send+Sync and thread/process
   equality. Run full source-frozen gates and independent review, then let the
   coordinator integrate. Ordered publication remains a separate downstream gate.

The singleton restrictions remain deliberately in V1: seven stage enumeration,
`sources[0]`/`input_contexts[0]`, one-source cardinalities, exactly two generated
areas, one complementary region and singleton closure contact. Simply widening
those counts cannot implement the planned proof.

## Verification limits

No live OCCT execution, GPU run, new sanitizer probe, host setting change or
thread-safety acceptance claim was made. Existing [ordinary-host TSan startup limitations](../frozen-prism-projection-v1-2026-10-14/README.md#ordinary-host-threadsanitizer)
remain unresolved; successful deterministic replay/stress and Rust
Send/Sync checks are separate evidence. There is no new storage redesign or
allocation/performance acceptance claim. Application/release acceptance is not
provided by this structural candidate.
