# FSD persistent-object virtual dispatch

The probe derives from `FSD_File` and `FSD_CmpFile`, overrides each persistent
object framing virtual, increments a counter, and calls the qualified native
base implementation. It then exercises the methods through
`Storage_BaseDriver&`, which confirms that the real storage interface dispatches
to those overrides. The captured bytes and counters are in
`tkernel_fsd_object_hooks.expected`.

The Rust regression exercises the equivalent per-instance `TextDialect` hooks
through `DataSections`, delegates to explicit `*_object_*_base` methods, and
checks byte-for-byte output, hook counts, stream-write failure, and malformed
header exception identity. Cmp's two begin-read hooks are native no-ops; its
persistent end validates the line ending.

Build and run against OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
g++ -std=c++17 tools/oracle/tkernel_fsd_object_hooks.cxx \
  -I target/occt-modeling-full/include/opencascade \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel-fsd-object-hooks
/tmp/tkernel-fsd-object-hooks
```

Source SHA-256: `FSD_File.hxx`
`175ceb78d367166624bf87f96ffc6b577744da43e43ab4e95aca87339648ab69`,
`FSD_File.cxx`
`52a2a856d67b53ddf027af9fb73f4ba9c7e17538e019237397df345bbd8053a5`,
`FSD_CmpFile.hxx`
`a2d976ce0d4d32429aa3b03b2617c9986ed58e37c69a21dc2f36b046f2e95ee4`, and
`FSD_CmpFile.cxx`
`cff486f740d492e8f856d2c1b0dad6b46f8d5a822b94141704b2572e94f4332d`.
The probe SHA-256 is
`8bffe3c92929baaf88ea15b8c373afabf0e8e22299eb070ecf67c424669dc7a9`;
the linked `libTKernel.so` SHA-256 is
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.
