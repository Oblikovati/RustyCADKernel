# Martensite live clipping transport

The clipping collection stores a real `NCollection_Sequence` of distinct native
handle fields. Its protected `clip_plane_items` is the actual item sequence;
`clip_override_global` is the actual override flag. Iterators retain native node
links and physical field observations without retaining clipping planes.
`first_ref` and `value_ref` return the original handle field, so existing aliases
observe protected-field assignment. The copied `first`, `value`, and `items`
conveniences are explicitly owning snapshots and are not native reference mappings.

Insertion borrows the original caller slot, checks current identities, allocates
outside collection guards, and constructs the payload after allocator callbacks.
Removal publishes native metadata and iterator advancement before destruction.
Clear saves Next before each callback, keeps First registered during the first
payload destructor, preserves the old size through current-resource Free, and
resets metadata after traversal. The valid native last-callback direct append loses
its node ownership; that native behavior is retained by the controlled traversal.
The earlier native dangling-node diagnostic remains outside successful parity credit.

CView `clip_planes_ref` captures the actual selected slot once. Its weak field
alias retains neither CView nor sequence ownership and does not repeat the virtual
getter on alias reads. `set_clip_planes` borrows its actual caller reference through
native assignment. V3d's Add/Remove bodies retain the intentional native local
sequence handle, then reread protected `driver_view_slot` for the final virtual
setter after allocation/destructor/getter callbacks. Concrete copied clipping
getters remain unclaimed conveniences.

Rendering receives its actual retained clipping slot explicitly. Native
`OpenGl_View` reads `myClipPlanes`, not an override of public `ClipPlanes`; the
snapshot occurs after culling, matching the pinned Reset ordering. Camera/light
acquisition and cap equations/material policies are preserved. The layer-input
helpers only separate responsibilities and do not adjust comparison gates.

Rust-only regressions replay all 44 existing family native records: six source
protocol/equation/material records, ten live references, thirteen valid retirement
records, five allocator/caller-retarget records, and ten protected-myView records.
The separate real-node allocator retirement prerequisite replays six further rows.
The callback fixture's external anchors match C++ lifetime/count ordering; retained
Camera aliases stay on the old field while retained View aliases follow myView.

Six further native cases exercise getter failure after protected-myView replacement,
setter failure before assignment, and setter failure after assignment for both
inherited Add and Remove. Typed Rust failures preserve the same receiver, plane and
sequence counts and prior edits: getter failure does not edit the collection, setter
failure keeps the old collection edits, and completed assignment is not rolled back.
The native generator runs twice with identical 26 records and retained strong anchors.

The full collection/view transport checkpoint now includes the native default-setter
retirement fixture, live Dump callback visitation, derived iterator initialization,
and allocator active-owner-loss proof. The allocator invocation copies immutable
code and weak identity; bounded callback-free field capture ends before invocation.
It retains physical backing independently of logical receiver destruction and adds
no intrusive ownership. Native Allocate retires the last owner before returning,
retargets the actual incoming plane slot, and the appended payload observes that
replacement. Focused Miri and borrow-escape checks cover this required boundary.
See [allocator-reference-free-invocation.md](allocator-reference-free-invocation.md).

Final source `e4e51d28e` was captured for all 31 workspace libraries. Four scoped
fragments contain 24 unique visible declarations: 22 implemented methods/actual
fields/parent alias, and two partial class records. The class and Iterator records
remain partial; basic descriptor target tests and the real parent type alias do
not establish every inherited/RTTI reference contract. Clip-plane chain-reference
closure is separately unproved. Existing copied conveniences receive no credit.
Explicitly deprecated/obsolete upstream APIs remain excluded by the project policy.

Three real retained scenes per device exercise inherited clipping, derived plane
and collection sources, and an actual returned-field alias clear. Native captures
repeat exactly; all six unchanged strict comparisons pass with zero RGBA mismatch.
The software maximum depth error is 1.4901161193847656e-8; AMD depth is exact.
Earlier-source captures remain separately attributed. Current final-source buffers,
commands, hashes and device receipts are under `derived-captures-freeze-source/`.

The frozen optimized executable measures eight workloads per device, including
actual live append, retirement and slot self-assignment. Each uses 20 warmups and
100 completed redraw/readback samples with validation disabled. All 1,600 samples
are finite and positive. Whole-process peak RSS is 100,292 KiB on default256
lavapipe and 63,836 KiB on AMD; GPU memory was not measured. Source/full-input,
external ICD/layer and executable hashes pass independent root bookends. Timing
is descriptive and no baseline was accepted. Raw vectors, process observations,
peak-memory output and prior evidence are preserved in `performance-freeze-source/`.

The final durable receipt is
`porting/runtime-evidence/clipping-live-transport-2026-10-12/final-handoff-receipt.json`.
No global ledger or generated inventory is edited on this branch. Weak alias
expiration remains safe-null outside C++'s valid reference lifetime; it receives
no native dangling-reference parity claim.
