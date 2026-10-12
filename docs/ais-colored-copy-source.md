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
