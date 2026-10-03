# Shape subdivision policy wrappers

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkshhealing_shape_divide_policies.cxx` against that build with `-std=c++17
-O2` and `TKShHealing TKTopAlgo TKBRep TKG3d TKG2d TKMath TKernel`. Native output is
committed as `.expected`. Rust reproduction:

```sh
cargo test -p rustycad-tkshhealing --test shape_upgrade_shape_policies
```

Four angle rows verify constructor configuration, replacement on SetMaxAngle,
incompatible surface criteria, and null face-tool getter fallback. Four closed-edge
rows verify that positive, zero and negative requested split counts all recreate
the tools and still produce two edges, exactly as native does. An additional Rust
regression checks that InitTool discards a previously installed wire policy.

Angle InitTool on a null face tool returns an explicit Rust failure instead of the
native null dereference; no exceptional-domain equivalence is claimed. The inherited
Perform retains its documented partial status. These cases do not add legacy
mapping declarations or establish full class/toolkit completion.
