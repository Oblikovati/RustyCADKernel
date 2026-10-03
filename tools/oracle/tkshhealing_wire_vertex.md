# Wire vertex analysis oracle

`tkshhealing_wire_vertex.cxx` executes OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871` without modifying its sources.
It records geometric statuses and the native state quirks: ignored Init precision,
SetStart writing the following parameter slot, shared arrays on copy, retained
records when curves are missing, retained state on empty initialization, and the
criterion 1 predicate excluding status 0 despite the header description.

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_wire_vertex.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_wire_vertex
/tmp/tkshhealing_wire_vertex > /tmp/tkshhealing_wire_vertex.actual
diff -u tools/oracle/tkshhealing_wire_vertex.expected /tmp/tkshhealing_wire_vertex.actual
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_vertex
```

The fixture supplies finite line segments. It is evidence for these cases, not
proof over every curve family. Native unfinished intersection/prolongation search
remains unfinished in this translation; unsupported junctions remain disjoined.
