# Martensite live subview sequence references

`V3dView::subviews_ref()` names the actual owned subview sequence. Its length,
one-based values, endpoints and node cursor observe subsequent public
`add_subview` / `remove_subview` changes without acquiring view ownership.
For example, retain `let alias = parent.subviews_ref()`, append a child, then
read `alias.value_ref(1)?.observe()`. Each callback runs after the brief state
read guard is released. `subviews()` and `snapshot()` are explicit owning copies.

The pinned OCCT CPU oracle records five public mutation phases, including
empty bounds and exact intrusive counts. Two executions are byte-identical;
Rust replays the committed assets without OCCT. A separate six-row callback
oracle observes removing another node and appending during traversal. Its
current-node const_cast replacement case is auxiliary container stress;
`mySubviews` is private upstream and no private-field claim is made.

Removing the retained current node makes its native reference dangling and is
outside defined upstream use. Martensite reports a typed failure instead.
Declared V3d add/remove and SetWindow reparenting remove then append nodes;
no declared V3d API transfers an existing node between sequences. Deep generic
container transfer contracts are separate from this getter proof.

This checkpoint claims only the public `V3d_View::Subviews` reference contract.
The view class and controller frame family remain partial. No rendering,
headset behavior or full controller orchestration credit follows from this CPU
container proof. Native pin and source/executable hashes are in the fixture
provenance files.
