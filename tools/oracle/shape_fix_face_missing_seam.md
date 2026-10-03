# ShapeFix_Face::FixMissingSeam

This probe covers the no-op on an open plane, an open sphere boundary, opposing open cylinder boundaries, a
same-direction cylinder pair (which requires native wire reversal), a single
same-direction cylinder pair (which requires native wire reversal), a pair
whose second wire is topologically reversed, a single open boundary on a
spindle torus, and an open boundary on a periodic B-spline sphere. The
reversed-wire case also checks the source iterator semantics: the native
`TopoDS_Iterator(wire)` composes parent orientation into its edge, so the
open-direction test sees a negative U displacement. It compares the returned
`changed` flag and resulting face, edge, seam, and degenerated-edge counts.

Run it against the pinned OCCT build with:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_fix_face_missing_seam.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d \
  -lTKG2d -lTKMath -lTKernel -o /tmp/shape_fix_face_missing_seam
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_fix_face_missing_seam
```

Expected output is in `shape_fix_face_missing_seam.expected`. The Rust regression uses
`BRepBuilderAPIMakeEdge::from_curve` for the circular input because OCCT's API maker
creates the endpoint topology consumed later by `ShapeFix_ComposeShell`.
