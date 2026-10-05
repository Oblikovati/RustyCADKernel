# Owned `Standard_Type::Register` lifetime probe

This probe registers one type, wraps the returned raw pointer in two native handles,
releases both handles, then registers the same `type_info` again with new metadata.
It observes first-registration-wins while the descriptor is live, parent-handle
ownership, unbinding on destruction, and fresh metadata after re-registration. It
does not compare addresses after destruction because the allocator may reuse them.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe intentionally avoids racing lookup against destruction: native destructor
unbinding is not mutex-protected, so that race is outside the defined probe domain.

Build and replay:

```sh
g++ -std=c++17 -pthread \
  -I<occt-build>/include/opencascade \
  tools/oracle/tkernel_runtime_type_owned_probe.cxx \
  -L<occt-build>/lin64/gcc/lib -Wl,-rpath,<occt-build>/lin64/gcc/lib \
  -lTKernel -o tkernel_runtime_type_owned_probe
./tkernel_runtime_type_owned_probe
```

Expected output is `tkernel_runtime_type_owned.expected`. The raw return starts at
count zero; a native `occ::handle` raises it, and releasing the final owner removes
the registry entry. Rust's owned API follows that lifecycle. Its legacy borrowed
`register()` compatibility API still pins storage for process lifetime and therefore
does not yet match raw registration destruction.

Hashes for this replay:

- Source: `0bd325860d25bef42bbb3301afc4c1abb0ff7d7842d006df8d744436418b27ca`
- Native executable: `3aee0dab9dd086fc1f24a549cb17e51689355fdbd6e32f4cfb2ac7abbe6af8f0`
- Expected output: `f0247d8eb8ed5807419b8f9975bdbef1898c4f34aa0c90aa5468264c5332af19`
- `libTKernel.so`: `78221dc495f8316c2a28e4fd9202920efdd17e056549006646ffd81bd0ddcc87`
