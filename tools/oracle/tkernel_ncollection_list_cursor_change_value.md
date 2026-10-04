# `NCollection_List::Iterator::ChangeValue` cursor probe

Pinned OCCT source revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`NCollection_BaseList.hxx` SHA-256: `d88ee7d178fc00567026a9e1b1427cdb94df3adb90fc94df0971df87b508ee43`.
`NCollection_List.hxx` SHA-256: `8c4bef79ead7da3d5c5f4667f905aebb7262628d025530766c9c0e4cdff41d01`.
Oracle `libTKernel.so` SHA-256: `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b`.
Probe SHA-256: `5686bf3163318807ee28495a1198d0ff3a55dfe2d01f882bbf7a74021852aaa2`.

The source declaration is `NCollection_BaseList::BasicIterator::ChangeValue()` in the
protected iterator-template class, reachable through `NCollection_List::iterator`. The
inventory records the template declaration as `67b17825739975517785b17ad8b787b9cac9b1fcf59fe2656cf24beea6e615ed`;
its instantiated member record `1bff44bb84ef8d5ee3e72fd45d32b9f5fb73501dbbc93f62f45ec526e2d0e892`
is template-owned. Neither identity is rewritten as a synthetic `NCollection_List` method.

`ChangeValue()` returns a mutable reference to the current item; it does not advance the
cursor. The fixture checks mutation before and after `Next()`, then verifies `More()` is
false after advancing past the tail. Calling `ChangeValue()` on an end cursor would
dereference a null node in C++; that undefined domain is safely reported as `None` by Rust.

Reproduce with the pinned foundation profile:

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_ncollection_list_cursor_change_value.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-list-cursor-change-value
/tmp/tkernel-list-cursor-change-value
```
