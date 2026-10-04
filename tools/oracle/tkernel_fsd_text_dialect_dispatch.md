# Stateful FSD virtual dispatch

`StatefulFile` derives from `FSD_File`, overrides the protected virtual text
primitives, counts calls, and delegates to the base implementations where the
base behavior must consume the stream. The fixture exercises direct calls and
inherited callers: `BeginReadInfoSection` dispatches `ReadChar` and `FindTag`
(which dispatches `ReadString`), `RootSectionSize` dispatches
`FlushEndOfLine` (which dispatches `ReadLine`), and `ReadRoot` dispatches
`ReadWord`. The captured output is in
`tkernel_fsd_text_dialect_dispatch.expected`.
The first output row checks `FSD_File::IsGoodFileType`: despite being invoked
through the derived type, this static method constructs a base `FSD_File`, so
the derived `ReadChar` counter stays unchanged.

Build/run against OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
g++ -std=c++17 tools/oracle/tkernel_fsd_text_dialect_dispatch.cxx \
  -I target/occt-modeling-full/include/opencascade \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel-fsd-text-dialect-dispatch
/tmp/tkernel-fsd-text-dialect-dispatch
```

Source hashes: `FSD_File.hxx` `175ceb78d367166624bf87f96ffc6b577744da43e43ab4e95aca87339648ab69`,
`FSD_File.cxx` `52a2a856d67b53ddf027af9fb73f4ba9c7e17538e019237397df345bbd8053a5`,
`FSD_CmpFile.hxx` `a2d976ce0d4d32429aa3b03b2617c9986ed58e37c69a21dc2f36b046f2e95ee4`,
and the fixture `d37c4f1d0b00a7bcbca24871330b43e00168e8f5782505b08f139b63f34405b5`.
The linked `libTKernel.so` hash is
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.

Rust `TextFile::with_dialect` stores an independent dialect value per driver.
`TextDialect` defaults now implement FSD behavior; `ReadChar`, `ReadWord`,
`FlushEndOfLine`, and `FindTag` are overridable and are used by both direct and
internal operations. Existing `File` and `CmpFile` use the stateless constructor
and retain their established formats. The generic dialect's magic number remains
a Rust format extension: native `MagicNumber()` is static and not virtual, so a
custom dialect must use the intended file magic explicitly when comparing to
native subclass behavior. Rust's static `is_good_file_type` similarly uses the
base `ReadChar` implementation and does not require installed custom dialect state.
