# Curve-on-face tolerance wrapper

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_curve_tolerance.cxx` against installed OCCT headers and link TKBO,
TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Output: `tkbo_curve_tolerance.expected`.

Fifteen cases combine five supports with missing, projected and shifted pcurves.
They compare done/exception state, maximum discrepancy and its parameter within
1e-12. Status -1 in this fixture encodes native Standard_NullObject, not a public
API status. The wrapper propagates that exception and retains both caller outputs.
The revolution fixture preserves the recorded discrepancy of one unit; no
geometric correction is substituted for the native calculation.

The row remains partial for seams, locations, wider curve geometry and other
failure paths in BRepLib_CheckCurveOnSurface. These supplemental comparisons add
no legacy declaration mapping.
