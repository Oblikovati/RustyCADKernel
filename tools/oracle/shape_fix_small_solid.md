# ShapeFix_FixSmallSolid oracle

`shape_fix_small_solid.cxx` uses OCCT's public constructor, threshold setters, and
`Remove` operation on a compound of one unit box and one `10 x 10 x 10` box. The
rows cover disabled thresholds, volume-only mode, width-factor-only mode, and the
native invalid-mode reset to mode zero. The `merge` row uses two boxes whose contact
faces are made topologically shared with a reshape context before the small solid is
merged into the large one. `merge_chain` makes two small solids adjacent in sequence
to verify the repeated merge round; `merge_isolated` verifies an unconnected small
solid is retained. Output columns are solid, shell, and face counts followed by the
signed BRepGProp volume. `merge_other_shell` gives the small solid a second
disconnected shell and exercises the shell-addition branch. Its recorded shell
replacement captures native iterator reference behavior: a pointer saved to
`TopoDS_Iterator::Value()` observes the final child value after iteration. The probe
also covers mode zero's requirement that both criteria pass and negative/NaN threshold
clearing. It is built against the pinned OCCT checkout's installed `TKShHealing` and
geometry libraries.

`shape_fix_small_solid_trailing_child.cxx` adds a non-shell vertex after the shared
shell in a malformed solid. Native `Merge` then follows the aliased iterator value
past the list of shell children and segfaults (exit 139 in the pinned build). The
Rust port returns a `Failure` naming the offending vertex instead of silently
merging a different shell. This malformed topology is an explicit native failure
domain, not a successful parity case.
