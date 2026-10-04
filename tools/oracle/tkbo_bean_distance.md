# BeanFace distance helpers

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The observer exposes private methods by changing header access control only;
the linked native algorithm implementation is unchanged. It directly calls
both Distance overloads and reads the cached surface projector afterward.

The 88 rows compare scalar distance, UV-overload distance and both UV outputs
for 11 line/conic–plane/cylinder/sphere cases, four curve parameters and original
or narrowed UV bounds. Native and Rust each observe **18 failed surface
projections**, so boundary fallback is exercised. The centre-of-sphere case
also reaches collapsed pole isolines. Repeated equal-distance candidates keep
the first native selection. UV clamping does not recompute the returned distance.

This is prerequisite evidence, not an implemented public C++ declaration or
legacy test mapping: the native methods are private. Remaining gaps include
located/freeform surfaces, invalid/infinite bounds, failure output side effects,
all floating-point exceptional inputs, and broader projection configurations.

Compile with the same native include/link settings as tkbo_bean_facade.cxx.
Stdout must match the expected fixture; stderr reports `fallback_count 18`.
Rust: `cargo test -p rustycad-tkbo --test int_tools_bean_facade`.
