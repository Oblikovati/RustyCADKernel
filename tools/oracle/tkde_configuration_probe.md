# TKDE configuration oracle

This supplemental probe targets OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It drives the native TKDE classes with a
shape-healing test node (`ProbeNode`) and prints `key|value` lines:

- `ctx.*`: the resource grammar of `DE_ConfigurationContext::LoadStr` (comments, missing
  colons, empty values, a trailing colon, `\` escapes, CR LF, a blank line that raises
  `Standard_OutOfRange`) and the iteration order of the resource map;
- `get.*`: the typed getters (`1e-3` as an integer, hexadecimal reals, string sequences);
- `fix.*`: the full text of `DE_ShapeFixConfigurationNode::Save` before and after `Load`;
- `wrap.*`, `load.*`, `copy.*`: the `DE_Wrapper` registry order, bind and unbind,
  priorities, lookups by extension, `Save` text, `Load` with priorities, `Copy` and the
  constructor from a handle;
- `content.*`, `valid.*`, `file.*`: `DE_ValidationUtils` and the resource files;
- `plugin.*`: the construction order of `DE_MultiPluginHolder` (libstdc++ builds the last
  tuple element first) and the unregistration of a shared format and vendor.

`crates/tkde/tests/de_native_oracle.rs` prints the same lines from the port and compares
them with `crates/tkde/tests/data/tkde_native_oracle.txt`, the output below.

```sh
B=target/occt-all-full
c++ -std=c++17 -O1 -I $B/include/opencascade tools/oracle/tkde_configuration_probe.cxx \
  -L $B/lin64/gcc/libi -Wl,-rpath,"$PWD/$B/lin64/gcc/libi" \
  -lTKDE -lTKBRep -lTKMath -lTKernel -o /tmp/tkde_configuration_probe
LC_ALL=C /tmp/tkde_configuration_probe > /tmp/tkde-native.actual
diff -u crates/tkde/tests/data/tkde_native_oracle.txt /tmp/tkde-native.actual
```

The probe and its Rust twin are supplemental. They do not add original OCCT test
declarations.
