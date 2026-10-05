# NCollection component operator dispatch

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe uses an element whose binary operators and compound assignment
operators deliberately return distinct tags. OCCT `Vec2` value `+`, `-`, `*`,
and `/` apply the element's binary operator directly. `Vec3` and `Vec4` value
operators copy the left vector and apply the matching compound assignment.
All dimensions' compound vector operators call the element compound assignment.
The captured result is `tkernel_vector_component_operator_paths.expected`.

Build and replay against the pinned generated headers:

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_vector_component_operator_paths.cxx \
  -o /tmp/vector-component-operator-paths
/tmp/vector-component-operator-paths > /tmp/vector-component-operator-paths.actual
diff -u tools/oracle/tkernel_vector_component_operator_paths.expected \
  /tmp/vector-component-operator-paths.actual
PATH=/home/vmiguel/.cargo/bin:$PATH cargo test -p rustycad-tkernel \
  --test ncollection_vector_operator_paths
```

Rust mirrors this distinction in its trait bounds and dispatch. This is
supplemental template behavior evidence; it does not add or claim legacy test
declarations, and it does not cover undefined/overflowing custom element
operations. Scalar division, including Vec3/Vec4 reciprocal specializations, is
outside this change.
