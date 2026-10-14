# Martensite mesh appearance drawer

`martensite-tkmeshvs` starts the TKMeshVS port with actual `MeshVS_Drawer` storage,
not a mesh renderer. OCCT names identify the pinned native declarations at
`3d097a0328e71b826377d4814ab05ec3c3d23871`. The upstream TKMeshVS GTest file
list is empty for this revision.

The drawer retains six independent TKernel `DataMap` instances. Keys remain
arbitrary signed integers and different types can coexist under the same key.
Setters replace one typed value, getters preserve the original output on misses,
and removal affects only one channel. Material and string values use their
existing genuine Martensite implementations. Tests mutate the source and outputs
and query the stored values afterward; `Clone` alone is not an isolation claim.

The native virtual `Assign` transport uses `MeshVSDrawerSource`, the actual
embedded base, and a borrowed original `HandleReference` argument. The base body
handles null and self assignment and copies fields in native assignment order:
integers, doubles, booleans, colors, materials, strings. Implicit copy construction
uses declaration order and resets the new object's intrusive count.

The native oracle records all six channels, absent outputs, material/string
mutation, assignment/removal, actual derived override dispatch, typed and foreign
failures after the base copy, and genuine intrusive counts. A separate native fixture records every exported
value: 40 drawer attributes, 16 display masks and 8 entity masks. The concrete
handle-to-source adapter forwards nullness and identity to the actual original
slot without projecting a pointee; a named metadata-only reader checks live and
null slot identity. This does not prove count-zero virtual Delete behavior.
Ordinary Rust tests
replay retained fixtures without compiling or running OCCT.

Full drawer class completion remains required and open: native RTTI
`get_type_descriptor` and virtual `DynamicType` return genuine `const Handle&`
fields. Checked main currently exposes descriptor pointees and owning conveniences.
This branch uses the existing descriptor for identity, not a duplicate cache, and
will adopt the original descriptor-field prerequisite only after it is checked
and integrated. These declarations and class completion are not credited by
pointee/descriptor-name tests. No whole toolkit or mesh rendering completion is
claimed. The native warning that text expansion, spacing, and display-type keys
have no rendering effect is retained; those integer declarations are preserved.
