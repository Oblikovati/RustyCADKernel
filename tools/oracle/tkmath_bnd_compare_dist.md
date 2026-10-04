# Bnd_B2/B3 protected distance hooks

`Bnd_B2` and `Bnd_B3` expose protected static `compareDist` and `compareDistD`
hooks to derived classes. At OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`,
both use strict `abs(distance[i]) > halfSize[i]`; equality and NaN comparisons
therefore return false. The Rust compact boxes previously embedded this logic
inside their box queries without a callable equivalent. The new associated
functions expose the protected behavior; Bnd_B2 segment checks now reuse the
double-coordinate form.

The probe derives tiny classes only to make the protected functions callable.
It was built and replayed against the pinned install headers with:

```sh
g++ -std=c++17 -O0 -g0 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkmath_bnd_compare_dist.cxx -o /tmp/tkmath_bnd_compare_dist
/tmp/tkmath_bnd_compare_dist > /tmp/tkmath_bnd_compare_dist.out
cmp tools/oracle/tkmath_bnd_compare_dist.expected /tmp/tkmath_bnd_compare_dist.out
```

Pinned source SHA-256 values:

| File | SHA-256 |
| --- | --- |
| `Bnd_B2.hxx` | `4c84ee209f2d0ba66be7148fb897047a43e80bd256d78bd909365d950332d418` |
| `Bnd_B3.hxx` | `fa515b61a58d232c5f6e5631bdd48b3716d22aa2c93d66a81b0984de47800fad` |
| Probe source | `dbe42823ec81c17038c2f7c89b94386e1f599176f50e7bb537f44b885fe54582` |
| Captured stdout | `bf58cd74bde2998c25a42fcc66ef47d81bf49de0592f4de1b12e23b3f3434fb7` |

The Rust tests cover f32/f64 scalar storage and f64 vector coordinates, exact
boundary values, outside values, and NaNs. They test the protected helper
contracts, not C++ inheritance or access control.
