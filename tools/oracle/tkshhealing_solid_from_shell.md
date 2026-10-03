# Solid repair oracle

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`tkshhealing_solid_from_shell.cxx` emits eight construction/state rows:
ordinary, reversed, empty and open shells, plus retained status after Init.
`tkshhealing_solid_perform.cxx` emits fourteen orchestration rows covering
shell repair modes, orientation modes, nested shells, disconnected components,
empty and open shells, shared-face compsolid creation, and cancellation.

The Rust comparisons include output shape kinds and counts, shell orientations
and closed flags, status bits, context creation, stored Solid versus Shape,
and signed volume (absolute tolerance 1e-9). Native island containment subtracts
both nested interior shells in the supplied case; that current behavior is
preserved. SolidFromShell catches classification failures and returns the solid
already constructed, as OCCT does. The native header needs an explicit prior
Message_ProgressRange include in the standalone oracle source.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_solid --test shape_fix_solid_perform`.
Build the C++ fixtures against the pinned modeling build with TKShHealing,
TKPrim, TKTopAlgo, TKBRep, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.

These are additional differential cases, not new legacy mappings. General
curved-shell containment, classification failures, context replacement domains
and all cancellation points require broader evidence before behavioral parity
can be claimed.
