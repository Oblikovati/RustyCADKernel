# gp_Ax2 standard-direction protected hooks

At OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, `gp_Ax2` declares
protected constexpr `getPerpendicularXDir`, `getPerpendicularYDir`, and
`crossStandardDir`. These are used by its axis-aligned frame constructors and
remain callable to subclasses. Rust already used these exact finite maps inside
frame construction but restricted them to the `gp` module. This change makes
the existing `StandardDirection3` methods public, avoiding duplicate logic.

The derived native probe calls the protected hooks for all six `gp_Dir::D`
values and all 36 cross-product pairs. The output retains OCCT's fallback for
parallel axes, which is not a geometric cross product. Build and replay with:

```sh
g++ -std=c++17 -O0 -g0 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkmath_gp_standard_direction_hooks.cxx \
  -o /tmp/tkmath_gp_standard_direction_hooks
/tmp/tkmath_gp_standard_direction_hooks > /tmp/tkmath_gp_standard_direction_hooks.out
cmp tools/oracle/tkmath_gp_standard_direction_hooks.expected \
  /tmp/tkmath_gp_standard_direction_hooks.out
```

Pinned source hashes:

| File | SHA-256 |
| --- | --- |
| `gp_Ax2.hxx` | `1254b1d47dbd585e6e8f20edb2595f28e9f6199d1cd4f7e01d2f9bc2bedc7ac1` |
| `gp_Dir.hxx` | `a732ec1495f2f1b8482d08041b978b3c3e077de1ecf024ab8abeafda7c10d2ff` |
| Probe source | `db93a0fcce5fb11d4b398b848c0f4399f90d12b5c8656d907a04ba0959db4c2d` |
| Captured output | `302f0eb737467ac164a87edb8ec56d64427884bd8da09dc9ecfcaad91b00ce17` |

The Rust tests cover the same finite domain. They establish hook outputs, not
C++ inheritance or access-control equivalence.
