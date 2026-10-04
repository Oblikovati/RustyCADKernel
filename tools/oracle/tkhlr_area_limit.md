# HLRBRep_AreaLimit native replay

Pinned source: `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`src/ModelingAlgorithms/TKHLR/HLRBRep/HLRBRep_AreaLimit.{hxx,cxx}`.

Compile the adjacent C++ probe with C++17, `-O0 -g0`, the native OCCT include
directory, TKHLR/TKMath/TKernel libraries and native library rpath. Its stdout
is recorded in `tkhlr_area_limit.expected`; the Rust integration test compares
the corresponding operations to that file.

The probe exercises every source-declared constructor/member: copied vertex,
both boundary/interference flags, all four state getters/setters, initially
null links, independently assigned neighbor/self links, handle identity, and
repeated Clear preserving the vertex and states. A mutex permits mutation
through shared Rust handles. As in the native implementation, link setters do
not automatically establish reciprocal links, and Clear releases only this
object's links.

This is supplemental API evidence. It maps no original GoogleTest declaration;
the full HLRBRep_Data and InternalAlgo legacy tests still need the exact HLR
pipeline. No concurrency equivalence is claimed from the Rust mutex.
