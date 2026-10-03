# 3D curve continuity splitter oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
This repository-owned probe is distinct from the legacy test inventory.

```sh
build=/path/to/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_split_curve3d_continuity.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKG3d -lTKMath -lTKernel \
  -o /tmp/rustycad-split-continuity-oracle
/tmp/rustycad-split-continuity-oracle
cargo test -p rustycad-tkshhealing --test shape_upgrade_split_curve3d_continuity
```

Each row records the case name, DONE status bits, retained split-sequence identity,
resulting direct B-spline knot count (-1 for another dynamic type), then split
parameters. The wrapper cases preserve native copied-basis recursion: corrections
to the nested copy do not replace the outer curve's basis. Existing split values
cause DONE1 to overwrite earlier DONE2/DONE3. The final case uses a restricted
range of a longer straight spline to check consecutive removal/index handling.

The Rust tests also check input-copy isolation, base-class state projection,
virtual Compute/Build dispatch, default construction and built result access.
These finite fixtures are not exhaustive behavioral proof.
