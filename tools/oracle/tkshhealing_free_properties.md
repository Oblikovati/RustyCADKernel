# ShapeAnalysis_FreeBoundsProperties oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Ten rows cover sampled contour measurements for a rectangle, circle and open
notch; empty/repeated lifecycle; retained tolerance; rejected notch outputs;
a sewing forecast; and a short intermediate edge included in a notch.
Native area and perimeter use 23 samples per edge, including cross-edge gaps,
and are deliberately not replaced by exact GProps measurements.

Behavior retained from source:

- Perform ORs all three phases, so an empty default object returns true.
- Dispatch appends rather than clearing; Init retains previous results, and the
  overload without tolerance retains the existing tolerance.
- Notch attempts replace the output wire even on invalid indices. Width changes
  only after the tangent-angle test accepts the notch.
- The public precision arguments to contour/notch checks are unused natively.
- Shared result sequence handles expose caller edits and record identity.

Additional Rust tests cover shared sequence mutation and index failures. Null
property handles return explicit Rust Failure where native dereferences null;
underlying sewing, wire analysis, and curve projection limitations remain.
These additional oracle cases do not add legacy declaration mappings or prove
whole-toolkit parity.
