# Reusable Cmp dialect defaults

`DerivedCmpFile` overrides all four protected Cmp text hooks, counts each call,
and delegates with a qualified `FSD_CmpFile::` call. Its `FlushEndOfLine`
override delegates to `FSD_File::FlushEndOfLine`, whose nested `ReadLine`
dispatch returns to the most-derived override. The fixture checks trimmed CR/LF,
decimal-unit Cmp read/write behavior, and nested virtual dispatch. Rust's
`TextDialect::USE_CMP_TEXT_DEFAULTS` selects reusable Cmp base bodies for custom
dialects; custom overrides can call the `read_cmp_*_base`/`write_cmp_*_base`
methods, and the base extended reader retains virtual flush dispatch. The FSD
oracle also checks qualified base calls and nested `FindTag` to `ReadString`
dispatch.

Build and run against OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
g++ -std=c++17 tools/oracle/tkernel_fsd_cmp_dialect_dispatch.cxx \
  -I target/occt-modeling-full/include/opencascade \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel-fsd-cmp-dialect-dispatch
/tmp/tkernel-fsd-cmp-dialect-dispatch
```

The captured output is `tkernel_fsd_cmp_dialect_dispatch.expected`. Source
hashes: `FSD_CmpFile.hxx` `a2d976ce0d4d32429aa3b03b2617c9986ed58e37c69a21dc2f36b046f2e95ee4`,
`FSD_CmpFile.cxx` `cff486f740d492e8f856d2c1b0dad6b46f8d5a822b94141704b2572e94f4332d`,
and `FSD_File.cxx` `52a2a856d67b53ddf027af9fb73f4ba9c7e17538e019237397df345bbd8053a5`.
Fixture SHA-256: `be6f4df814e18f403a16901fda5705ad6c005b007a60c00b86703982350c835e`.
The linked `libTKernel.so` SHA-256 is
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.
