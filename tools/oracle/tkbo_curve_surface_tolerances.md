# Curve-on-surface tolerance correction

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The native probe records 400 rows across plane, cylinder, torus, revolution and
Bezier supports; unchanged, shifted pcurve, shifted spatial curve and inconsistent
range/parameter flags; four avoidance sets; two maximum tolerances; and the
standalone CorrectCurveOnSurface or combined CorrectTolerances entry points.
Rows compare failure status and every unique edge/vertex tolerance.
Replay: `cargo test -p rustycad-tkbo --test make_pcurve curve_surface_tolerances`.

The source translation runs all wire corrections before all edge corrections.
It retains the native first-edge-only intersection length threshold, matches
representations by surface and relative location, validates both seam pcurves,
and projects onto planar supports when no stored pcurve matches. The combined
entry point runs point-on-curve correction first.

This is partial coverage: scheduling remains serial; arbitrary locations,
multiple spatial representations, seam, nonplanar pcurve-only reference,
intersection-segment and exceptional malformed topology cases require additional
native fixtures. A missing spatial adaptor in planar projection returns a Rust
Standard_NullObject error rather than reproducing a possible native null access.
These supplemental fixtures do not establish complete legacy test parity.
