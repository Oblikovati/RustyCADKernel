# Martensite AIS click/touch integration evidence

This slice implements 11 more captured AIS_ViewController declarations: camera
and object animation handles, camera abort, live mouse-selection bindings,
mouse-click dispatch and touch add/update/remove. Controller record/constructor
claims remain partial. Mouse-button/position, 3D-mouse, window-listener interface,
camera/redraw and XR dispatch remain open. No exclusions or denominator changes
were added to claim class completion.

The reference is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Two local native runs reproduced 163 click/animation and 254 touch snapshots
byte-for-byte. Committed provenance records header, implementation, generator
and fixture hashes. Rust tests require no OCCT compiler, headers or runtime.

Thirteen new regressions cover default animation ownership and handle identity,
null and polymorphic timelines, running/paused/stopped camera abort, pause-before-
selection and error propagation, every selection scheme and default modifier
binding, double-click flag behavior, retained duplicate picks, independent queues,
duplicate and unknown touch IDs, virtual clear-before/unknown-update transitions,
pinch-to-one/two-finger origins, unclamped tolerance, NaN comparison branches,
exact double-tap boundaries and callback failure ordering during removal.

The NaN snapshots exposed an incorrect comparison before integration. The fix
preserves NCollection_Vec2's exact x > y ? x : y maximum and x < y ? x : y minimum.
Duplicate touch IDs keep their stored origin/current position, while controller
click/rotation origins restart from the newly supplied point. The fixtures keep
both details independently observable.

All 544 TKV3d tests pass on the feature checkout and integrated main. Clippy with
warnings denied, canonical suite discovery, Rust formatting and 13 workflow/suite
regressions pass. The renderer is unchanged; paired-device functional/capture and
performance gates remain in CI. See [implementation notes](../../../docs/ais-view-controller-pointer.md).

The feature commits are integrated directly into main. The clean completed
worktree and branch were deleted after ancestry, source equality and process-
reference checks, reclaiming 2.71 GiB. No archive was made. The integration receipt
and compressed logs retain these checks and the fresh compiler inventory.

Fresh compiler inventory uniquely binds all 95 controller claims: 93 implemented
and two partial record/constructor claims. Port-scope completed APIs rise from
52,277 to 52,288; open APIs fall from 22,787 to 22,776, with 977 partial APIs
unchanged. TKV3d has 2,667 done declarations out of 5,172 (51.6%), 2,501 open
and four partial. There are zero unresolved ledger rows; the explicit compiler
configuration/identity/mapping proof gaps still prevent full completion.
