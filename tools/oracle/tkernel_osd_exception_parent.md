# OSD exception parent replay

`DEFINE_STANDARD_EXCEPTION` builds `std::exception` descendants with virtual
`ExceptionType()` names. Those classes do not inherit `Standard_Transient` and do
not declare `DEFINE_STANDARD_RTTI*`; there is no `Standard_Type` descriptor or
static descriptor owner baseline to mirror. The Rust `Failure` value already kept
the names, but `Failure::is_kind` lacked the OSD intermediate parent links. This
probe checks representative Windows-style, Unix-signal, and OSD system-error catch
paths. The Rust table is compared against all 21 OSD exception macro declarations
plus the separately declared `OSD_OSDError`.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Pinned `libTKernel.so` SHA-256:
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.

```sh
c++ -std=c++17 -O2 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_osd_exception_parent_probe.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel_osd_exception_parent_probe
/tmp/tkernel_osd_exception_parent_probe > /tmp/tkernel_osd_exception_parent_probe.out
cmp tools/oracle/tkernel_osd_exception_parent.expected /tmp/tkernel_osd_exception_parent_probe.out
```

Rust regression command:
`cargo +1.98.1 test -p rustycad-tkernel --test standard_exceptions_behavior`.
