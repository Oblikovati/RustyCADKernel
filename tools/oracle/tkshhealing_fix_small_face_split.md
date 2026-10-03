# `ShapeFix_FixSmallFace::SplitOneFace` deterministic oracle

The native probe constructs a triangular planar face with a vertex offset by `1e-5`
from the opposite boundary edge, and assigns vertex tolerance `1e-4`. It sets the private
`ShapeAnalysis_CheckSmallFace::myPrecision` through the probe-only access macro before
calling `SplitOneFace`. That access provides a deterministic fixture state which the public
`ShapeFix_FixSmallFace` API cannot configure. The pinned direct result is a successful
split into two faces. A second run through `FixSplitFace` replaces the source face in the
input and reports two faces in a compound with `DONE3`.

This fixture validates recursive split construction under an explicitly initialized
analyzer. The Rust implementation matches its two-face, six-edge, twelve-vertex output and
the aggregate `DONE3` replacement. It does not resolve native ordinary-constructor behavior:
the public FixSmallFace path leaves `myPrecision` uninitialized before reading it, so that
domain remains explicitly partial.
