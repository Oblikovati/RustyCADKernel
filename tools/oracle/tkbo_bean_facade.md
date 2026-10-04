# BeanFace facade: native line/plane stage

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The 72 native rows exercise six geometric cases through both shape and adaptor
constructors, initial state, two Perform calls, Init after success, another
Perform, and context reset/recreation. Cases include transverse crossing,
contained and separated parallel lines, a 2.5e-7 separation, rejection outside
surface bounds, and oblique crossing. Rows compare completion, context presence,
minimum-distance sentinel, copied result count and all range endpoints.

Native shape Init adds Precision::Confusion to the two topology tolerances;
adaptor Init does not. The existing private line/plane helper now accepts that
criterion separately from the tolerances used to compute intersection width.
Native Init clears results but retains IsDone and the explicitly selected curve
range. Repeated Perform clears earlier ranges. These distinctions are preserved.

The public facade currently supports the line/plane Perform path only; other
curve/surface pairs return an explicit unported-stage failure. General localized,
extrema, boundary and coincidence stages and the long constructor/init overloads
remain pending. Shallow adaptor copies and allocator/exception edge cases have
not received full equivalence proof. No legacy declaration acceptance is added.

Compile the probe with the pinned native modeling include/libi paths, linking
TKBO, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKMath and TKernel.
Rust replay: `cargo test -p rustycad-tkbo --test int_tools_bean_facade`.
