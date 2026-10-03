# Wire-to-shape endpoint connectivity

The native fixture links pinned OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871` and records the four pairing
choices, equidistant tie behavior, tolerance rejection, null/unsupported shapes,
and forward/reversed wire endpoints. Rust compares every numeric field exactly.

Normal rows encode `found DONE1 DONE2 DONE3 DONE4 DONE5 DONE6 FAIL1 FAIL2
 tailhead tailtail headtail headhead minimum3d maximum3d`. The four mutable
output values start at distinct sentinels, verifying preservation on early exits.

An eleventh case deliberately builds a wire with distinct coincident vertices
instead of shared connectivity. Native endpoint discovery yields null vertices
and raises Standard_NullObject. The Rust result carries the same exception type,
keeps all output sentinels, and preserves the already assigned DONE1 status.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_shape_connect
```

Use the native compile command from
[the singular-edge fixture](tkshhealing_wire_singular_checks.md), replacing its
source with `tkshhealing_wire_shape_connect.cxx`. These branch regressions do not
establish full wire or toolkit parity.
