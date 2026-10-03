# ShapeAnalysis bounding-box wire selector

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Fourteen rows compare acceptance, selected index, stop flag, and six status flags.
Both shared-vertex and geometric selection cover all four endpoint orientations.
The stateful sequence retains the native preference for an existing tail
candidate, exclusions, independent resets of boxes/tolerance, and shared-mode
failure retaining the previous selected index.

Additional Rust tests cover box rejection, excluded wrong-type entries, invalid
index precedence, and error propagation through UBTree::try_select. This new
fallible adapter uses the existing traversal and returns callback failures
without calling another selector hook. Null array input reports explicit Rust
failure where native dereferences null; this remains outside native defined
behavior. The API inventory must not interpret these additional oracle tests
as new legacy declaration mappings or whole-toolkit completion.
