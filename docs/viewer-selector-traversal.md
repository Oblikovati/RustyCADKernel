# Viewer selection traversal and public picking

The port follows `SelectMgr_ViewerSelector.cxx` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. `viewer_selector_overlap.rs`
accumulates actual sensitive matches by owner. `viewer_selector_object.rs`
walks each object's sensitive BVH. `viewer_selector_traversal.rs` walks all
five object subsets and invokes virtual sorting only after successful traversal.
`viewer_selector_pick.rs` supplies the four public Pick overloads.

The retained thread-pool sentry covers clearing, traversal and final sorting.
A failing callback leaves completed matches unsorted; a subsequent traversal
can recover. Entity callbacks execute outside selector field/BVH guards.
Window Size callbacks also execute outside the selection-volume guard.

Matching preserves OCCT's live owner-slot timing: the selectable is captured
before Matches; the owner is read afterward; insertion rereads the slot after
point processing. Existing-owner counts advance before point processing, and
box repeats retain the original criterion. Only exact 2D persistence combined
with presentations enables display-priority preference; AIS presentation lookup
supplies its priority, while other selectable types retain the invalid sentinel.

Traversal preserves left-first BVH order, active wrappers, view affinity,
object and entity persistence, model-aware flipping, and native matrix mutation
across subset order. Persistent entities and flippers bypass sensitive bounds
as in OCCT. Global box clipping still runs after an object overrides global
planes. Non-overlap area selection removes newly inserted owners when fewer
entities matched than the sensitive set records for that owner.

Point Pick sets the sensitivity tolerance. Box Pick reuses the rectangular
volume without resetting that tolerance. Polyline Pick updates its overlap
flag through initialization. Axis Pick assigns neither the camera nor window
size, and a newly created axis volume therefore has no camera. All overloads
refresh layer order, build geometry, assign view clipping, then traverse with
the actual view identification. Missing windows produce a checked NullObject
after the same initialized prefix, rather than dereferencing null.

Validation includes 192 native overlap traces, 1,042 native traversal cases,
and 174 repeated public-pick traces. Fourteen direct tests cover these seams,
including failure recovery and callback inspection without a held selector lock.
The native null driver's SetClipPlanes now retains its argument, matching the
Rust null driver and making these clipping comparisons meaningful. The native
polyline builder assumes one-based input while calculating polygon orientation;
valid traversal fixtures use that indexing. Existing frustum tests retain the
separate invalid-input behavior coverage.

The subsequent selection-image and sensitive-display ports complete the declared
non-boilerplate ViewerSelector implementation mappings. Arbitrary graph mutation that invalidates OCCT's borrowed references or
BVH iteration is not claimed as a supported concurrent operation. Root traversal
retains the original intersector's camera slot and reads its current handle at
persistence and subset boundaries. A Matches callback can replace that handle;
108 native comparisons cover point, box and polygon selection, both projection
types, entity persistence and mixed object subsets. The minimized regression
previously picked two owners where OCCT picked one. Matrix values still follow
native snapshot/subset timing. Replacing the active manager does not redirect the
retained camera slot, and no camera guard spans a client callback. Clearing a
camera during a traversal can make OCCT dereference null while rebuilding a
frustum; that invalid native path is not counted as parity evidence.
