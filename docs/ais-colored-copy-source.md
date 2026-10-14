# Martensite ColoredShape copy source references

At pinned OCCT `3d097a0`, `AIS_ColoredShape.cxx:75–101` repeatedly
accesses the original `const Handle<AIS_Shape>&` constructor argument.
A virtual Material, Color or Transparency getter can replace that actual
caller slot. Geometry was already copied from its initial target; later
appearance predicates and getters must access its current target.

`AisColoredShape::from_shape` therefore accepts
`&dyn HandleReference<dyn AisShapeSource>`. An ordinary `&Handle` remains
usable, while `HandleFieldRef` forwards a genuinely mutable caller slot.
Each predicate and getter observes that slot separately without adding
native ownership. Topology acquisition retains an owning shape at the
same constructor boundary as upstream.

The local native oracle `martensite_colored_copy_source.cxx` runs 18
scenarios: three getter stages, full/default incoming appearance, and
success/typed/foreign exceptions. Two native runs produce identical 108
rows. Rust independently replays topology, appearance, map independence,
original/incoming ownership and failure followed by repair. Both source
objects remain externally owned during getter callbacks; native dangling
object execution and null source dereferences are outside this proof.

The existing copy and failure fixtures remain active. A named application
subclass also exercises inherited real presentation and selection
orchestration. The TKV3d suite runs with:

```sh
cargo test -p rustycad-tkv3d --all-targets --all-features
```

The receipt and one-declaration constructor fragment live under
`porting/runtime-evidence/colored-copy-source-2026-10-12/copy`.
Its pinned compiler preflight proves this constructor; it grants no
inherited AIS_Shape HLR or whole-class completion.

## Current Martensite contract audit (2026-10-14)

The constructor and its source-reference tests were integrated in
`9b1a19383`. The historical OCCT rows above remain unchanged regression
fixtures; they do not define new kernel acceptance criteria. Recovery at
`d7a7fb9c4` adds independent assertions in `colored_shape_copy_tests.rs`
using explicit colors, widths, topology identity and ownership invariants.
No live OCCT executable or fixture regeneration is involved.

The five `copy_contract_*` tests establish these bounded properties:

- Replacing the source's shape field and changing source color/custom
  appearance leaves the copy's original shape, color and empty custom map.
- Changing copy color, width and custom appearance leaves source color,
  width and custom map intact.
- Releasing the source leaves the copy's topology and appearance usable.
- Topology payload is shared: changing its `checked` flag through either
  presentation is visible through the other. This is a recorded limitation,
  not proof of deep geometry, cache or mesh isolation.
- Material, color and transparency getter failures release the partial
  copy's topology ownership and permit subsequent successful construction.
  An unrelated panic retains its original payload and also permits reuse.
  These tests use the existing named application subclass fixture.

`from_shape` returns a mutable presentation object and `Failure`, with no
completion/validation/history/error-bound envelope. It has no cancellation
parameter. Shape and appearance reads use shared-state locks. No immutable,
lock-free kernel result, full geometry isolation, cancellation behavior,
concurrent source-mutation consistency or thread-safety acceptance is claimed.
Null source dereferences and releasing the last external source owner inside
a getter remain outside this audit. The existing slot-replacement fixture
covers callback-directed replacement; it is not a concurrent mutation test.

The new tests depend only on the already integrated constructor, existing
copy/expansion fixtures and existing shape flag APIs. They require none of
the unfinished HLR changes in this checkout. The branch's merge base with
main is `8062f0f23`; its seven original unique commits end at `d7a7fb9c4`
and include HLR, adaptor retirement, location references, presentation
forwarding, downstream consumers and Vulkan assets. Those commits remain
subject to `porting/runtime-evidence/shape-hlr-2026-10-13/WORK-IN-PROGRESS.md`.
Integrating this audit alone must not imply acceptance of that series.

Checks run on Linux x86_64 with the pinned Rust 1.99.0 toolchain, all targets,
all features and the lockfile. The initial unchanged TKV3d run passed 951
tests; the final run passed 956 (400 unit, one fatal-path and 555 integration),
with zero failures or ignored tests. The new transparency assertion uses
the exact `f64::from(0.4_f32)` value stored by the existing material API.
`cargo clippy -p rustycad-tkv3d --all-targets --all-features --locked -- -D warnings`
and formatting of the edited Rust file also passed. The toolchain revision
is `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, on Linux 7.1.5.
The shell's default Rust 1.95.0 fails the workspace MSRV requirement;
put the pinned toolchain's `bin` directory first in `PATH` before running the
command above. GPU captures, performance/peak-memory comparisons, Miri,
ThreadSanitizer and other platforms are excluded from this bounded audit.
