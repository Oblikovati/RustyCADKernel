# Martensite mesh presentation helpers

The seven public `MeshVS_Tool` operations use the pinned OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871` expressions and lookup order.
The factories produce the existing real Graphic3d fill, line, marker and text
aspects. No renderer submission policy or production native dependency is added.
The unfinished MeshVS Drawer RTTI/Delete scope remains open independently.

Native integer appearance keys select existing typed values; a material value
channel does not substitute for a front/back material integer ID. Each lookup
uses the original borrowed drawer reference. Missing required values return null
before constructor validation. Text retains the bold default, arbitrary font
bytes and the first-NUL boundary of native `ToCString`; its expansion and spacing
arguments retain the existing native ignored behavior. Materials and returned
aspects are independent values, checked by mutation in both directions.

The integer-to-enum casts can retain unnamed values in the original C++ enum
ranges. Existing Rust enum types and numeric casts remain available; explicit
unnamed discriminants and checked conversions avoid invalid Rust enum values.
The only exhaustive consumers changed follow actual native post-switch behavior:
unknown line types keep their identity with a solid pattern, and unknown font
aspect diagnostic text is `invalid`. Values outside the C++ enum ranges and
material table indices are native undefined-input domains, not successful
fallbacks. The Rust boundary reports their offending values explicitly.

The no-material fill overload dereferences its drawer before checking null and
unconditionally dereferences the delegated fill. Thus null drawer, or required
fill keys missing after both material keys were supplied with defaults disabled,
are native null-dereference precondition violations. Rust reports these states;
no native successful-return parity is claimed for those undefined executions.
The explicit-material fill, line, marker and text overloads have safe null exits.

Polygon normals retain the fixed first edge, componentwise `Precision::Confusion`
tests, per-triangle normalization threshold and unnormalized averaging order.
Signed array bounds and the special zero-based count header remain significant.
`GetNormal` preserves output on non-planarity; `GetAverageNormal` writes the
average and returns false. Degenerate polygons of at least three points can
return a zero vector successfully; fewer than three preserve output and fail.
Bounds failures preserve output and their original typed cause.

A removable native capture generator lives under `tools/oracle`. Its attributed
fixture is permanent under `crates/tkmeshvs/tests/fixtures`; ordinary Rust tests
never execute OCCT. Exact floating-point bit rows cover every native enum-range
value, every safe required-key omission, defaults/custom values, failure order,
font bytes and planar/nonplanar/degenerate/signed-bound normal inputs. Independent
invariants additionally cover repaired input reuse and retained previous results.
