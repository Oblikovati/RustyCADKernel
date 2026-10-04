# Boolean intersection pcurve installation

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
This translates `BOPTools_AlgoTools::MakePCurve` from `BOPTools_AlgoTools.cxx`.

The native `tkbo_make_section_pcurve.cxx` records 160 rows: five support fixtures,
four enable masks, supplied versus projected intersection pcurves, and all four
face orientations. The second face is a separate same-geometry support, with a
different orientation. Edge SameParameter and SameRange start false. Compare
with `cargo test -p rustycad-tkbo --test make_pcurve make_section_pcurve`.

Observations include edge tolerance, both parameter flags, pcurve presence,
ranges, and midpoint UV. This exercises unconditional SameParameter when both
installation flags are false as well as periodic and nonperiodic adjustment.
The edge tolerance used by each UpdateEdge is captured before either face's
projection, matching native sequencing.

Supplemental fixtures do not establish complete legacy or API parity. General
spline and distinct-geometry face intersection curves remain to be tested; the
mapping stays partial.
