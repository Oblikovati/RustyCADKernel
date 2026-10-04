# `BRepOffset_MakeOffset::MakeThickSolid` without removed faces

This probe isolates the native no-cap branch: `MakeThickSolid` calls
`MakeOffsetShape`, and when no `AddFace` faces were supplied, reverses the
completed result only for a positive global offset. The zero-offset row
records the native precheck failure without reading the null result's faces.
Pinned outputs are in `tkoffset_make_thick_solid_no_caps.expected.txt`.

The Rust regression is `cargo test -p rustycad-tkoffset --test
make_offset_thick_solid`. Its two sign cases match result and face orientation;
the zero case matches `IsDone`, `Error`, and null result. The implementation
reports a named partial-port error for calls with closing faces because the
native cap-removal, open-shell offset, and quilting path is not implemented yet.

Build with the same include and library paths as
`tkoffset_get_analyse_unimplemented.md`, substituting
`tkoffset_make_thick_solid_no_caps.cxx` and output path
`/tmp/tkoffset-make-thick-solid-no-caps`.
