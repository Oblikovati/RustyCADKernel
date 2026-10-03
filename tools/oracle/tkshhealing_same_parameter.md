# ShapeFix::SameParameter package oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Nine cases retain result, SameParameter/SameRange edge-occurrence counts,
summed edge tolerance, and warning count. Cases include empty and cancelled
shapes, an ordinary/forced/cancelled free edge, displaced planar geometry,
a box, a cylinder, and a missing 3D curve.

The native routine computes the caller's requested precision locally, but calls
ShapeFix_Edge with default zero tolerance. The Rust routine preserves this
observable behavior. Planar tolerance checks retain the native 23 samples,
parameter range replacement by CurveOnSurface, and 1.00005 growth coefficient.
Cancellation is checked at the native phase boundaries, including empty inputs.
Failures propagate as Rust Failure; native source has no catch despite its
historical header description. Child algorithm limitations remain; this does
not establish whole-package parity or add legacy declaration mappings.
