# `Poly_CoherentTriPtr` nullable lifecycle

The fixture uses the protected `Poly_CoherentTriPtr(const Poly_CoherentTriangle*)` constructor with a null pointer, links valid members, resets the head pointer, and removes the null node without reading its triangle. It never calls `GetTriangle`, `Iterator::Value`, or `Iterator::ChangeValue` while the stored pointer is null because those operations dereference that pointer. The public `Append` and `Prepend` implementations dereference their input pointer when constructing the new member, so null inputs to those methods are outside the defined fixture domain. Rust's ordinary value iterator explicitly panics at a null member; native dereference behavior there is undefined. Nullable inspection, equality, and debug formatting continue across null entries without dereferencing them.

This is a bounded Rust lifecycle mirror. The public Rust value uses triangle indices and safe checked access; it does not expose OCCT triangle pointers or establish ABI-level pointer identity. The existing stable member token is Rust-side only.

- OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`
- Source: `src/Poly/Poly_CoherentTriPtr.hxx` SHA-256 `36c26272136fccf3f3e28a66536068d2b4859a855edf7cf01c1e5ffc91708387`; `src/Poly/Poly_CoherentTriPtr.cxx` SHA-256 `94c8114cd73f61b4df39ab335ac530b0a4f1bd00390d553c0f772894bd4f7eba`.
- Probe: `tools/oracle/tkmath_coherent_nullable_triptr.cxx`
- Expected output: `tools/oracle/tkmath_coherent_nullable_triptr.expected`
- Probe SHA-256: `65f50a3df56a3703a57b5eec341489b4a9102221f9402413fb282df03462c5d9`.
- Oracle profile: Linux x86_64, GCC C++17 `-O2`; `libTKMath.so` SHA-256 `4844f77b9a2177b458e7121443f8f7f1fad09e166003d56df7433180e1bb55e0`, `libTKernel.so` SHA-256 `230e4d112806e8d980bb2ec0ec7de56aea7639689790fde1eca18aa62c334ab5`.
- Rebuild command: `c++ -std=c++17 -O2 -I <occt-build>/include/opencascade tools/oracle/tkmath_coherent_nullable_triptr.cxx <occt-build>/lin64/gcc/libi/libTKMath.so <occt-build>/lin64/gcc/libi/libTKernel.so -Wl,-rpath,<occt-build>/lin64/gcc/libi -o /tmp/tkmath-coherent-null-native`.
- Relevant native behavior: the protected constructor and `SetTriangle` store nullable pointers (`Poly_CoherentTriPtr.hxx:113,167-172`); `Append`/`Prepend` dereference their pointer arguments (`Poly_CoherentTriPtr.cxx:34-64`).
- Rust coverage is supplemental; it does not alter legacy declaration counts.

Build with the pinned OCCT foundation include and libraries. The probe has no output dependency on triangle dereference while the member is null.
