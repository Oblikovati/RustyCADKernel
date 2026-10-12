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

This is a source checkpoint, not full-family completion. No audited partial claims
or class/constructor markers are promoted. Dedicated derived render capture proof,
final scoped compiler bindings and coordinated optimized performance still remain. Ordinary both-device tests are tracked separately from
native image-capture parity. Weak alias expiration is safe-null outside C++'s valid
reference lifetime, rather than a claim about native dangling-reference behavior.
