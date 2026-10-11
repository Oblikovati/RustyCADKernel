# Martensite ViewCube animation reference

OCCT `AIS_ViewCube::ViewAnimation()` returns a const reference to the animation
**handle slot**. Retaining that reference observes subsequent setter assignments,
including null and direct protected-field replacement. Copying the current
handle observes the old animation instead. The original implementation's copied
getter therefore did not complete the native reference contract.

`AisViewCubeMembers::view_animation_ref()` now returns
`AisViewCubeAnimationRef`, a borrowed live view of that actual retained slot:

```rust,ignore
let alias = cube.view_animation_ref();
cube.set_view_animation(&next);
assert_eq!(alias.object_address(), next.object_address());
cube.set_view_animation(&Handle::default());
assert!(alias.is_null());
```

The view holds neither a field lock nor an owning animation reference. Two views
of the same widget alias the same storage, and each access reads its current
assignment. It cannot outlive the widget. Explicit `clone_handle()` creates an
owning snapshot; existing `view_animation()` remains a copied convenience and
is not the implementation binding for the native reference getter.

`with_animation` temporarily retains the current object, releases the field
lock, then calls user code. This prevents concurrent replacement from
invalidating the callback's borrow and permits setters inside that callback.
Returning an owning snapshot is explicit; no stale target is retained by the
live view itself.

The native probe at OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`
retains two actual getter references and original owning handles. Seven states
cover setter replacement, null, restoration and protected-field replacements;
both aliases, actual getter-slot identity, copied-handle identity, null state and
three reference counts are replayed exactly in Rust. Two native executions are
byte-identical. Original owning handles keep all referenced objects alive, so
the proof excludes dangling-reference undefined behavior.

Only the existing `AIS_ViewCube::ViewAnimation` fragment row changes its target
and status. Whole-class coverage, other appearance getters and the global
ledger remain outside this correction. The historical gap receipt records why
the original claim was downgraded; new native provenance and compiler binding
proof are in `porting/runtime-evidence/view-cube-animation-reference/`.

Run `cargo test --locked -p rustycad-tkv3d --all-targets` for Rust-only replay.
