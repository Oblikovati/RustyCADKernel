# Area wrapper direct dispatch regression

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The native fixture subdivides a planar 2 by 2 face with fixed 2 by 3 counts.
It records changed-result, face count, and DONE status: `1 6 1`.

The Rust regression deliberately omits the `ShapeDivider` trait import. Before
an inherent forwarding method was added, dereferencing the area wrapper selected
the base `perform` implementation and returned `0 1 0`. The forwarding method
retains the derived criterion for ordinary direct calls. This is additional
oracle evidence, not a legacy-test mapping or full-class parity claim.
