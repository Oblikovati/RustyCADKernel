# Nested Compound unification oracle

The pinned OCCT build records `nested_compound faces 1 edges 4 wires 1 vertices 8 valid 1`
for a Compound containing a planar face whose bottom boundary is split into two
collinear edges. `ShapeUpgrade_UnifySameDomain::UnifyEdges` maps all edges under
the input shape, repairs changed faces, and applies its reshape context through
the containing topology. This fixture isolates the nested-container behavior
without relying on Boolean operations.

Run `tools/oracle/tkshhealing_unify_nested_compound.cxx` with the OCCT include
and library paths documented in the adjacent UnifySameDomain oracle notes.
