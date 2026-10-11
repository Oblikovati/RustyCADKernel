# Martensite light-set iteration and cached state

`Graphic3dLightSetIterator` now reads the set's current indexed membership.
The earlier Rust implementation copied membership when constructing a cursor;
that missed OCCT's append and swap-last removal behavior. `Graphic3dLightSet::lights()`
remains an explicit snapshot convenience. This correction follows OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`'s `Graphic3d_LightSet` and
`NCollection_BaseMap::IndexedIterator`.

```rust
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{Graphic3dCLight, Graphic3dLightSet,
    Graphic3dLightSetIterator, Graphic3dLightSetIterationFilter as Filter,
    Graphic3dTypeOfLightSource as Kind};
let set = Graphic3dLightSet::default();
let light = Handle::new(Graphic3dCLight::new(Kind::Directional));
set.add(&light)?;
let mut cursor = Graphic3dLightSetIterator::new(&set, Filter::ExcludeDisabled);
while cursor.more() {
    let current = cursor.value()?;
    cursor.next();
}
Ok::<(), rustycad_tkernel::standard::Failure>(())
```

A cursor retains its numeric index and reads the map at that index each time.
Removing a member moves the last member into its slot. Appending can make an
exhausted cursor valid again when the new extent reaches its index. Cursor
clones have independent indices and share membership. Construction and `next()`
check enabled, ambient and shadow filters against live light properties.
`more()` and `value()` do not recheck a current light whose flags changed;
`skip_filtered()` exposes the native protected recheck hook explicitly.
An exhausted, default or null-set cursor returns `Standard_NoSuchObject` from
`value()`, through `Result<Handle<Graphic3dCLight>, Failure>`.
The numeric probe now unwraps that result only inside a successful `more()` loop.

Martensite retains the shared map storage so a cursor remains memory safe if its
set wrapper is dropped. Native cursors borrow a map and require its lifetime to
cover iteration. The lifetime safety regression is recorded separately from
native behavioral parity. Returned handles own their light reference rather
than borrowing a native map entry. Concurrent compound mutations are not a
native oracle claim; the fixture exercises sequential operations.

Raw membership/type counts and structural revisions change immediately.
Enabled counts, ambient accumulation and shader keys remain cached until
`update_revision()`. Several source changes coalesce into one set revision;
structural changes already dirtying the cache prevent an extra revision bump.
Disabled sources also refresh their cached light revision. Ambient channels
are unclamped float accumulation, with alpha fixed to one after an update.
This behavior was already present and is now covered by committed Rust replay
rather than relying on the existing local numeric OCCT harness.

The CPU fixture contains 576 transitions across all six named filters. It covers
both initialized constructors, empty/default/null cursors, duplicate/null adds,
missing removals, bounds including `i32::MIN`, swap-last mutation, independently
advanced copied cursors, current/future flag changes, the protected filter hook,
stale reads, colored ambient float bits and update/no-op revision coalescing.
The generator and provenance are committed beside the Rust test; ordinary tests
require no OCCT libraries, compiler or runner generation:

```sh
cargo test -p rustycad-tkservice --all-targets
```

`porting/ledger-fragments/light-set-live-iteration.json` pins 25 existing corrected
operations and keeps both class records partial. Native protected raw map/state
fields and a complete subclass layout remain open. The fragment does not claim
private helpers, expand the denominator, or promote getters as newly ported work.
Its two `Default` constructor bindings use captured impl-context selectors; all
other bindings resolve to unique exported inherent methods or class records.
RTTI, unrelated light behavior, renderer submission, GPU performance and
Windows/macOS validation are outside this bounded correction.
