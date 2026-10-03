# ShapeFix_Face small-area wire removal

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Seven native cases cover a large contour, complete removal enabled/disabled,
mixed large/small contours, reversed face orientation, missing context, and an
existing context face replacement. Rust compares result boolean, retained wire
count, working-face identity/orientation, root shape identity, result identity,
and context removal.

OCCT returns false when all wires are small, even when it removes the face from
context. It retains the working face in that branch. With mixed contours, it
replaces only the working face and context; the root shape and Result remain
unchanged. The initial context application updates the working face immediately.
Warnings and the source's unusual non-wire eligibility conjunction are preserved.
These are additional oracle tests, not legacy declaration mappings.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -O2 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_face_small_area.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_face_small_area
/tmp/tkshhealing_face_small_area
cargo test -p rustycad-tkshhealing --test shape_fix_face_small_area
```
