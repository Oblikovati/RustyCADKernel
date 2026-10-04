# `Storage_BaseDriver` protected state setters

This probe derives from `FSD_File` to expose the native protected methods without
changing their access policy. It confirms `SetName` and `SetOpenMode` only mutate
base state: a manually selected read mode makes the next `Open` report
`Storage_VSAlreadyOpen`, while `Open` still replaces the stored name before that
check. `Close` clears the mode and preserves the name.

Reproduce against OCCT source revision `3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
g++ -std=c++17 tools/oracle/tkernel_storage_base_driver_setters.cxx \
  -I target/occt-modeling-full/include/opencascade \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel-storage-base-driver-setters
/tmp/tkernel-storage-base-driver-setters
```

The captured output is `tkernel_storage_base_driver_setters.expected`. The native
header SHA-256 is `a4a37f433d9d2b8ac913373c30486349020c7430116152b21460f99ad977109f`;
the linked `libTKernel.so` SHA-256 is
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.
The Rust equivalents are default `BaseDriver` methods that delegate to the
driver's `DriverBase` state, allowing concrete FSD drivers to use the inherited
protected operations without exposing the state field itself.

The captured declaration IDs are `e1762a6e9b51d45924b9114f40ccd82f9a9882f46bb99ed007b45d02abc85f62`
for `Storage_BaseDriver::SetName` and `a7bb31ba7c6b0ebb60fcc4b39595646a0e0cc633c122c7521d167f7cc0bfc486`
for `Storage_BaseDriver::SetOpenMode`; each was unique in the captured
`unclaimed_cpp_declarations` inventory before this mapping.
