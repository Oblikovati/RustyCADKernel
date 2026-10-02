# BRepGProp native reference fixtures

`occ49-cylinder-minus-box.brep` is the exact result of the pinned OCCT operation
`BRepAlgoAPI_Cut(BRepPrimAPI_MakeCylinder(10, 20), BRepPrimAPI_MakeBox(10, 10, 10))`.
It was captured with `BRepTools::Write` against OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`.

Pinned native reference from `BRepGProp::VolumeProperties`:

- `IsDone()`: true; `BRepCheck_Analyzer::IsValid()`: true
- volume: `5497.7871437821368`
- centre of mass: `(-0.60630454511198362, -0.60630454511198295, 10.714285714285715)`
- inertia matrix, row major:
  `315878.24146288086, 14521.015150373298, -19047.619047619028,`
  `14521.015150373298, 315878.24146287912, -19047.619047619057,`
  `-19047.619047619028, -19047.619047619057, 270847.32688836136`
- `HasSymmetryAxis()`: false
- topology counts: 44 vertices, 22 edges, 6 faces, 1 shell, 1 solid

The Rust support test reads this exact output through the existing `BRepTools::Read`
port and checks its volume properties. It does not exercise the missing Rust Boolean Cut
operation or OCCT's `IsDone()` precondition; `BRepGProp_Test.cxx` remains unmapped until
that behavior is available.
