# Periodic cone pcurve gap regression

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.
The fixture exactly reconstructs the interpolated conical ring, runs native
ShapeFix_Face::Perform, selects its constant-v ring, and runs FixGaps2d on that edge.
Output records return, input orientation, range, repaired endpoint coordinates,
endpoint identity and last-vertex tolerance.

Native closes the reversed ring at U=2*pi, V=0.5. The earlier supplemental Rust
assertion expected U=0 from a periodic-midpoint shortcut. The source-derived
conversion uses the oriented second endpoint when the second spline is null in
the same-edge branch; this fixture corrects that expectation. No native legacy
assertion or mapping was weakened. Full curved pcurve parity remains unproven.

The native direct result also retains distinct endpoint vertices: only the oriented
last endpoint is replaced, leaving the other occurrence at its original tolerance
(0.0017861609869531973). The old shortcut replaced both and incorrectly asserted
identical vertices at confusion tolerance. The direct result and later reshape
context application must not be conflated.

Applying the reshape context to the original wire creates a further edge copy;
it differs from both original and direct-result edges and reunifies its endpoint
vertices. The fixture and Rust assertions also record these three identities.
