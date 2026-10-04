# Offset interval and concavity foundations

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.
The fixture records all six ChFiDS_TypeOfConcavity values through the interval's
three-argument constructor; reversed bounds are retained. Independent setters
preserve the other values. Negative zero, infinity, and NaN are not normalized.

The default native constructor leaves all three scalar fields uninitialized.
The fixture initializes every field before reading it. Rust represents those
unset fields with Option and returns ConstructionError if read too early; this
is an explicit boundary for C++ undefined behavior, not a native parity claim.

TKFillet is introduced only for the shared concavity type. Its fillet algorithms
remain unported. No legacy declaration is mapped by these supplemental fixtures.
