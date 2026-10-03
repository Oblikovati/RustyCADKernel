# ComposeShell sequence SplitByLine oracle

OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`tkshhealing_split_sequence.cxx` exercises the protected sequence overload through
a derived probe. Seven cases cover vertical and horizontal cuts, an outside cut,
a boundary tangency, closed mode, an open wire with FAIL4, and two disjoint loops.
The expected rows include segment orientation, edge count, patch indices,
p-curve ranges and endpoint coordinates, and 3D-curve/SameParameter/SameRange flags.
Rust compares every recorded field in `shape_fix_compose_shell_behavior/split_sequence.rs`.

The implementation combines all wire events, preserves native stable ordering,
merges tangential event pairs, tracks crossing and tangency parity, creates
EXTERNAL seam segments, and applies reshape replacements. Private tests also
exercise equal and NaN ordering, identity-based tangency merging, and retained
half-parity state. These are additional tests, not mapped legacy declarations.

An empty compound replacement remains an explicit parity limitation: native
ApplyContext returns zero and the enclosing sequence loop cannot advance. Rust
reports Standard_NotImplemented. A removed edge produces a null replacement;
native dereferences it in ShapeType, whereas Rust reports Standard_NullObject. This is not an approved behavioral exception
or evidence of complete API parity. The seven native fixtures do not exercise it.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -O2 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_split_sequence.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_split_sequence
/tmp/tkshhealing_split_sequence
cargo test -p rustycad-tkshhealing --test shape_fix_compose_shell_behavior
cargo test -p rustycad-tkshhealing --lib split_line_sequence
```
