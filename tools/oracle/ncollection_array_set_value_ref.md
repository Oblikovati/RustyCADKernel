# Borrowed Array SetValue

The native `const_reference` overload validates its index before invoking copy assignment. This fixture also checks that neither setter replaces the element object and that rejected indices do not assign.

```sh
c++ -std=c++17 -O2 \
  -I target/occt-tkernel-full/include/opencascade \
  tools/oracle/ncollection_array_set_value_ref.cxx \
  -L /home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib \
  -lTKernel -o /tmp/ncollection_array_set_value_ref
/tmp/ncollection_array_set_value_ref | diff -u tools/oracle/ncollection_array_set_value_ref.expected -
cargo test -p rustycad-tkernel --test ncollection_array_set_value_ref
```

Rust's `set_value_ref` uses `Clone::clone_from` after checked access. Safe borrowing prevents passing an element of the mutably borrowed array as the source, so native self-alias calls remain outside this Rust method's direct domain.
