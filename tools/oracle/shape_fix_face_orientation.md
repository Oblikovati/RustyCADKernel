# `ShapeFix_Face` orientation and natural-bound oracle

This probe is built against the pinned OCCT source revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It records orientation, child
preservation, return values, and `MapWires` sizes for single, nested, disjoint,
reversed-face, tiny-wire, non-orientable-child, periodic-cylinder, and sphere-natural-bound inputs. It
also records finite-empty and periodic-wire `FixAddNaturalBound` behavior. The
periodic fixture records classifiers on the shifted wire and the `MSG5`/`MSG11`
warning pair, including the native post-conflict wire-state handling.

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/shape_fix_face_orientation.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/rustycad-shape-fix-face-oracle
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib:\
/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/rustycad-shape-fix-face-oracle
```

Pinned output:

```text
single_ccw changed=0 map=0 wires=1 forward=1 reversed=0 internal=0 mapped_children=0
single_cw changed=1 map=0 wires=1 forward=1 reversed=0 internal=0 mapped_children=0
nested changed=1 map=1 wires=3 forward=3 reversed=0 internal=0 mapped_children=2
disjoint changed=0 map=2 wires=2 forward=2 reversed=0 internal=0 mapped_children=0
reversed_face changed=0 map=0 wires=1 forward=0 reversed=1 internal=0 mapped_children=0
tiny_wire_removed changed=1 map=0 wires=1 forward=1 reversed=0 internal=0 mapped_children=0
empty_wire_removed changed=1 map=0 wires=1 forward=1 reversed=0 internal=0 mapped_children=0
internal_child_preserved changed=1 map=0 wires=2 forward=1 reversed=0 internal=1 mapped_children=0
periodic_edge_counts=1:4,4
periodic_states infinite=1 0.1,2.5=1 0.3,1=1 0.5,2.5=1 0.3,4=1
periodic_shifted_states infinite=1 5.8,2.5=1 6.1,1=1 6.4,2.5=1 6.1,4=1
periodic_reverse_states infinite=1 5.8,2.5=1 6.1,1=1 6.4,2.5=1 6.1,4=1
periodic_messages Unknown message invoked with the keyword FixAdvFace.FixOrientation.MSG11:L=1:R=0 Unknown message invoked with the keyword FixAdvFace.FixOrientation.MSG5:L=0:R=0
periodic_shift changed=1 pcurve_mid_before=0.1 pcurve_mid_after=6.38319 map=0 forward=2 reversed=0 warning_shapes=2 edge_forward=4 edge_reversed=4
shared_seam_reversal changed=1 p=6.28319 p=6.28319
empty_sphere_natural_bound changed=1 wires=1 orientation=0
empty_cylinder_natural_bound changed=0 wires=0
uninitialized_natural_bound changed=0
sphere_wire_natural_bound changed=0 wires=1
 torus_p=6.48319,6.88319 torus_p=3.64159,6.78319
torus_reversed_wire_natural_bound changed=1 wires=2
context changed=1 result_matches_context=1 result_forward=1
sphere_natural_bound changed=0 wire_orientation=1
```

The Rust and C++ tiny one-edge fixtures both build a valid unit line edge and
then change its 3D curve range to `[0, 1e-8]` with `BRep_Builder::Range`. This
keeps edge construction valid while making the sampled curve length lower
than `Precision::Confusion()`.

The `MapShape` warning records are hash-map ordered, so the `MSG5` and `MSG11`
entries can appear in either order.

The periodic result also checks that the candidate right pcurve is translated by
`2π` while its source wire records the native `MSG5` warning. The companion wire
records `MSG11`: OCCT sets an unknown aggregate state on mixed edge classifications,
then still applies the final candidate-state comparison before selecting the
warning/reversal behavior. The Rust port preserves that control flow.

`shared_seam_reversal` adds the exact same periodic-cylinder wire twice, once reversed,
so the first pass exercises a seam pcurve shared by both source wires. Native reverses
the first wire before classifying the second; the resulting shared pcurve midpoint is
`2π` on both rebuilt wires. Rust now classifies and applies each first-pass result in
source order before proceeding to the next wire, then performs the separate nested-wire
pass.

`empty_sphere_natural_bound` calls `FixAddNaturalBound` on a newly initialized empty
sphere face. OCCT creates one forward natural-bound wire and reports a change.
`empty_cylinder_natural_bound` confirms the no-wire branch declines an unbounded
cylindrical surface.
`uninitialized_natural_bound` records the null-support early return.
`sphere_wire_natural_bound` starts with one small spherical patch wire; the existing
outer-bound test keeps `FixAddNaturalBound` unchanged.
`torus_reversed_wire_natural_bound` uses a reversed wire on a doubly periodic torus.
The fixture checks the original wire's `+2π` p-curve translation and the new natural
boundary's selected UV interval shift.
